//! Where provider secrets (OAuth device tokens, cookies) live.
//!
//! Order of preference: the OS credential store when the target has one, then an
//! encrypted file next to the settings. The fallback is obfuscation, not protection:
//! the key is derived from the machine id, which any local process can read.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const SERVICE: &str = "org.alvindimas05.fluyer";
const FALLBACK_FILE: &str = "secrets.enc";
const FALLBACK_KEY_FILE: &str = "secrets.key";
const FALLBACK_SALT: &[u8] = b"fluyer-secrets-v1";

pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str) -> bool;
    fn delete(&self, key: &str);
}

/// OS credential store, then encrypted file. Both are tried on read; writes prefer
/// the OS store and fall back only when it is unavailable (headless Linux, no keyring
/// daemon, or a platform without a backend such as Android).
pub struct Secrets {
    primary: Option<Box<dyn SecretStore>>,
    fallback: EncryptedFile,
}

impl Secrets {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            primary: platform_store(),
            fallback: EncryptedFile::new(data_dir),
        }
    }

    /// In-memory store, for tests.
    pub fn memory() -> Memory {
        Memory::default()
    }
}

impl SecretStore for Secrets {
    fn get(&self, key: &str) -> Option<String> {
        self.primary
            .as_ref()
            .and_then(|store| store.get(key))
            .or_else(|| self.fallback.get(key))
    }

    fn set(&self, key: &str, value: &str) -> bool {
        match self.primary.as_ref() {
            Some(store) if store.set(key, value) => true,
            _ => self.fallback.set(key, value),
        }
    }

    fn delete(&self, key: &str) {
        if let Some(store) = self.primary.as_ref() {
            store.delete(key);
        }
        self.fallback.delete(key);
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
fn platform_store() -> Option<Box<dyn SecretStore>> {
    Some(Box::new(KeyringStore))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_store() -> Option<Box<dyn SecretStore>> {
    None
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
struct KeyringStore;

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
impl KeyringStore {
    fn entry(key: &str) -> Option<keyring::Entry> {
        keyring::Entry::new(SERVICE, key).ok()
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
impl SecretStore for KeyringStore {
    fn get(&self, key: &str) -> Option<String> {
        Self::entry(key)?.get_password().ok()
    }

    fn set(&self, key: &str, value: &str) -> bool {
        Self::entry(key).map(|e| e.set_password(value).is_ok()).unwrap_or(false)
    }

    fn delete(&self, key: &str) {
        if let Some(entry) = Self::entry(key) {
            let _ = entry.delete_credential();
        }
    }
}

/// AES-256-GCM file keyed by `HKDF(machine id)`. Whole map is rewritten on every
/// change: providers store one or two tokens, not a keychain dump.
pub struct EncryptedFile {
    path: PathBuf,
    /// Where the fallback key material comes from when the OS has no machine id.
    data_dir: PathBuf,
    /// Cached map, so read/modify/write doesn't decrypt twice per call.
    cache: Mutex<Option<BTreeMap<String, String>>>,
}

impl EncryptedFile {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(FALLBACK_FILE),
            data_dir: data_dir.to_path_buf(),
            cache: Mutex::new(None),
        }
    }

    fn key(&self) -> Option<[u8; 32]> {
        let hk = Hkdf::<Sha256>::new(Some(FALLBACK_SALT), self.key_material()?.as_slice());
        let mut key = [0u8; 32];
        hk.expand(b"secrets", &mut key).ok()?;
        Some(key)
    }

    /// Machine id when the OS exposes one, else a random key file created on first use
    /// (Android, or any host without one). Both are readable by local processes.
    fn key_material(&self) -> Option<Vec<u8>> {
        #[cfg(not(target_os = "android"))]
        if let Ok(machine) = machine_uid::get() {
            return Some(machine.into_bytes());
        }
        let path = self.data_dir.join(FALLBACK_KEY_FILE);
        if let Ok(existing) = std::fs::read(&path) {
            if existing.len() >= 32 {
                return Some(existing);
            }
        }
        let material: [u8; 32] = rand::random();
        let _ = std::fs::create_dir_all(&self.data_dir);
        std::fs::write(&path, material).ok()?;
        Some(material.to_vec())
    }

    fn load(&self) -> BTreeMap<String, String> {
        let mut cached = self.cache.lock().expect("secrets cache");
        if let Some(map) = cached.as_ref() {
            return map.clone();
        }
        let map = self.read_from_disk().unwrap_or_default();
        *cached = Some(map.clone());
        map
    }

    fn read_from_disk(&self) -> Option<BTreeMap<String, String>> {
        let raw = std::fs::read(&self.path).ok()?;
        // Layout: 12-byte nonce | ciphertext
        if raw.len() < 12 {
            return None;
        }
        let (nonce, body) = raw.split_at(12);
        let cipher = Aes256Gcm::new_from_slice(&self.key()?).ok()?;
        let plain = cipher
            .decrypt(Nonce::from_slice(nonce), Payload { msg: body, aad: b"" })
            .ok()?;
        serde_json::from_slice(&plain).ok()
    }

    fn write_to_disk(&self, map: &BTreeMap<String, String>) -> bool {
        let Some(key) = self.key() else { return false };
        let Ok(cipher) = Aes256Gcm::new_from_slice(&key) else { return false };
        let nonce_bytes: [u8; 12] = rand::random();
        let Ok(plain) = serde_json::to_vec(map) else { return false };
        let Ok(body) = cipher.encrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: &plain, aad: b"" })
        else {
            return false;
        };
        let mut out = Vec::with_capacity(12 + body.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&body);
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.path, out).is_ok()
    }
}

impl SecretStore for EncryptedFile {
    fn get(&self, key: &str) -> Option<String> {
        self.load().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) -> bool {
        let mut map = self.load();
        map.insert(key.to_string(), value.to_string());
        let ok = self.write_to_disk(&map);
        *self.cache.lock().expect("secrets cache") = Some(map);
        ok
    }

    fn delete(&self, key: &str) {
        let mut map = self.load();
        map.remove(key);
        let _ = self.write_to_disk(&map);
        *self.cache.lock().expect("secrets cache") = Some(map);
    }
}

#[derive(Default)]
pub struct Memory(Mutex<BTreeMap<String, String>>);

impl SecretStore for Memory {
    fn get(&self, key: &str) -> Option<String> {
        self.0.lock().expect("memory secrets").get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) -> bool {
        self.0
            .lock()
            .expect("memory secrets")
            .insert(key.to_string(), value.to_string());
        true
    }

    fn delete(&self, key: &str) {
        self.0.lock().expect("memory secrets").remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fluyer-secrets-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn encrypted_file_round_trips_and_reloads_from_disk() {
        let dir = temp_dir();
        let store = EncryptedFile::new(&dir);
        assert!(store.set("ytmusic.token", "abc123"));
        assert_eq!(store.get("ytmusic.token").as_deref(), Some("abc123"));

        let reopened = EncryptedFile::new(&dir);
        assert_eq!(reopened.get("ytmusic.token").as_deref(), Some("abc123"));
        reopened.delete("ytmusic.token");
        assert_eq!(reopened.get("ytmusic.token"), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn file_is_not_plaintext_and_survives_garbage() {
        let dir = temp_dir();
        let path = dir.join(FALLBACK_FILE);
        let store = EncryptedFile::new(&dir);
        assert!(store.set("k", "super-secret-value"));
        let raw = std::fs::read(&path).unwrap();
        assert!(!String::from_utf8_lossy(&raw).contains("super-secret-value"));

        std::fs::write(&path, b"garbage").unwrap();
        let reopened = EncryptedFile::new(&dir);
        assert_eq!(reopened.get("k"), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn memory_backend_matches_the_trait() {
        let store = Secrets::memory();
        assert_eq!(store.get("a"), None);
        assert!(store.set("a", "1"));
        assert_eq!(store.get("a").as_deref(), Some("1"));
        store.delete("a");
        assert_eq!(store.get("a"), None);
    }
}
