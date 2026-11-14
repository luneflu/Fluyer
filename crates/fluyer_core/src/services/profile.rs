//! Provider API profile: the request envelope (client version, keys, endpoint
//! paths) that unofficial APIs rotate faster than an app release cycle.
//!
//! Rust owns the file. Bundled snapshot is the floor; a newer signed profile can be
//! downloaded, but only a confirmed one is applied (`check` -> UI dialog -> `apply`).
//! Parsers stay in Rust: nothing downloaded here is executed.

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const BUNDLED: &str = include_str!("../../assets/profile.json");
const CACHED_FILE: &str = "profile.json";
const STATE_FILE: &str = "profile-state.json";

/// Compile-time verification key (hex, 32 bytes). Empty until a release key exists:
/// updates are then downloaded but never applied, so a build without a key cannot be
/// talked into running a foreign profile.
pub const PROFILE_PUBKEY_HEX: &str = "";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub version: u32,
    pub client_version: String,
    pub api_key: String,
    /// Named request paths ("search", "player", "browse") so a rename is data, not code.
    pub endpoints: BTreeMap<String, String>,
    /// Device-code OAuth client. Rotates with Google's device clients, so it is data.
    pub auth_client_id: String,
    pub auth_client_secret: String,
}

impl Profile {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let profile: Profile = serde_json::from_slice(bytes).ok()?;
        // A profile without an envelope is a bundled placeholder, not an update.
        if profile.version == 0 || profile.client_version.is_empty() {
            return None;
        }
        Some(profile)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct State {
    /// Version the user declined; that one is not offered again.
    dismissed: Option<u32>,
}

pub struct ProfileService {
    data_dir: PathBuf,
    current: Mutex<Profile>,
    pending: Mutex<Option<Profile>>,
    /// URL of the release asset holding `profile.json`; None disables checks.
    source: Option<String>,
}

impl ProfileService {
    pub fn new(data_dir: &Path, source: Option<String>) -> Self {
        let current = Self::load_best(data_dir);
        Self {
            data_dir: data_dir.to_path_buf(),
            current: Mutex::new(current),
            pending: Mutex::new(None),
            source,
        }
    }

    /// Bundled snapshot when nothing better exists; the cached file wins when it
    /// parses, so a previously applied update survives restarts offline.
    fn load_best(data_dir: &Path) -> Profile {
        std::fs::read(data_dir.join(CACHED_FILE))
            .ok()
            .and_then(|bytes| Profile::parse(&bytes))
            .or_else(|| Profile::parse(BUNDLED.as_bytes()))
            .unwrap_or_default()
    }

    pub fn current(&self) -> Profile {
        self.current.lock().expect("profile").clone()
    }

    pub fn pending(&self) -> Option<Profile> {
        self.pending.lock().expect("profile").clone()
    }

    /// Downloads the profile and records it as pending. Never applies it: the caller
    /// shows a confirmation and then calls [`Self::apply`].
    pub async fn check(&self) -> Result<Option<Profile>, String> {
        let Some(url) = self.source.clone() else { return Ok(None) };
        let sig_url = format!("{}.sig", url);
        let client = reqwest::Client::new();
        let bytes = get_bytes(&client, &url).await?;
        let Some(profile) = Profile::parse(&bytes) else {
            return Err("profile payload is not a usable profile".to_string());
        };
        if profile.version <= self.current().version {
            return Ok(None);
        }
        if self.dismissed() == Some(profile.version) {
            return Ok(None);
        }
        let signature = get_bytes(&client, &sig_url).await?;
        if !verify(&bytes, &signature) {
            return Err("profile signature rejected".to_string());
        }
        *self.pending.lock().expect("profile") = Some(profile.clone());
        Ok(Some(profile))
    }

    /// Writes the confirmed profile and makes it current.
    pub fn apply(&self) -> bool {
        let Some(profile) = self.pending.lock().expect("profile").take() else {
            return false;
        };
        let Ok(json) = serde_json::to_vec_pretty(&profile) else { return false };
        if std::fs::write(self.data_dir.join(CACHED_FILE), json).is_err() {
            return false;
        }
        *self.current.lock().expect("profile") = profile;
        true
    }

    pub fn dismiss(&self) -> bool {
        let Some(pending) = self.pending.lock().expect("profile").take() else {
            return false;
        };
        let state = State { dismissed: Some(pending.version) };
        let json = serde_json::to_vec(&state).unwrap_or_default();
        std::fs::write(self.data_dir.join(STATE_FILE), json).is_ok()
    }

    fn dismissed(&self) -> Option<u32> {
        let bytes = std::fs::read(self.data_dir.join(STATE_FILE)).ok()?;
        serde_json::from_slice::<State>(&bytes).ok()?.dismissed
    }
}

async fn get_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("profile request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("profile request failed: {}", response.status()));
    }
    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("profile body failed: {e}"))
}

/// Ed25519 over the exact bytes of `profile.json`; the `.sig` asset is base64.
/// False while [`PROFILE_PUBKEY_HEX`] is empty: an unkeyed build applies nothing.
pub fn verify(payload: &[u8], signature_b64: &[u8]) -> bool {
    verify_with(payload, signature_b64, PROFILE_PUBKEY_HEX)
}

fn verify_with(payload: &[u8], signature_b64: &[u8], pubkey_hex: &str) -> bool {
    if pubkey_hex.is_empty() {
        return false;
    }
    let Some(key_bytes) = hex_decode(pubkey_hex).filter(|b| b.len() == 32) else {
        return false;
    };
    let Ok(key) = VerifyingKey::from_bytes(&key_bytes.try_into().expect("32 bytes")) else {
        return false;
    };
    let Ok(sig_text) = std::str::from_utf8(signature_b64) else { return false };
    let Ok(sig_bytes) = base64::engine::general_purpose::STANDARD.decode(sig_text.trim()) else {
        return false;
    };
    let Ok(sig_bytes) = <[u8; 64]>::try_from(sig_bytes) else { return false };
    key.verify(payload, &Signature::from_bytes(&sig_bytes)).is_ok()
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fluyer-profile-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample(version: u32) -> Profile {
        Profile {
            version,
            client_version: "1.20250101".into(),
            api_key: "AIza".into(),
            endpoints: [("search".to_string(), "/youtubei/v1/search".to_string())].into(),
            auth_client_id: "id".into(),
            auth_client_secret: "secret".into(),
        }
    }

    #[test]
    fn bundled_profile_is_usable_and_can_be_updated() {
        let dir = temp_dir();
        let service = ProfileService::new(&dir, None);
        // The checked-in snapshot must work anonymously: browse needs a version + key.
        let bundled = service.current();
        assert!(bundled.version > 0);
        assert!(!bundled.client_version.is_empty());
        assert!(!bundled.api_key.is_empty());
        assert_eq!(service.pending(), None);
        assert!(Profile::parse(BUNDLED.as_bytes()).is_some());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn applied_profile_is_loaded_again_on_restart() {
        let dir = temp_dir();
        let service = ProfileService::new(&dir, None);
        *service.pending.lock().unwrap() = Some(sample(7));
        assert!(service.apply());
        assert_eq!(service.current().version, 7);

        let reopened = ProfileService::new(&dir, None);
        assert_eq!(reopened.current().version, 7);
        assert_eq!(reopened.current().client_version, "1.20250101");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dismissing_a_version_records_it_and_clears_the_pending_one() {
        let dir = temp_dir();
        let service = ProfileService::new(&dir, None);
        *service.pending.lock().unwrap() = Some(sample(9));
        assert!(service.dismiss());
        assert_eq!(service.dismissed(), Some(9));
        assert_eq!(service.pending(), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unkeyed_build_applies_nothing() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let payload = br#"{"version":4,"clientVersion":"x"}"#;
        let sig = base64::engine::general_purpose::STANDARD.encode(signing.sign(payload).to_bytes());
        // PROFILE_PUBKEY_HEX is empty, so even a valid signature is refused.
        assert!(!verify(payload, sig.as_bytes()));
    }

    #[test]
    fn ed25519_verification_accepts_the_payload_and_rejects_tampering() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let pubkey_hex: String = signing.verifying_key().as_bytes().iter().map(|b| format!("{b:02x}")).collect();
        let payload = br#"{"version":4,"clientVersion":"x"}"#;
        let sig = base64::engine::general_purpose::STANDARD.encode(signing.sign(payload).to_bytes());
        assert!(verify_with(payload, sig.as_bytes(), &pubkey_hex));
        assert!(!verify_with(b"{}", sig.as_bytes(), &pubkey_hex));
        assert!(!verify_with(payload, b"not-base64", &pubkey_hex));
    }
}
