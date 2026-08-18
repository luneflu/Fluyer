use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread::available_parallelism;
use tauri::Manager;
use tokio::sync::{watch, Semaphore};

use crate::state::{app_handle, app_store};

const STORE_KEY_DPI: &str = "music-cover-art-dpi";

static RESIZE_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
static BASE_COVER_SIZE: OnceLock<u32> = OnceLock::new();

lazy_static::lazy_static! {
    static ref IMAGE_CACHE_QUEUE: Mutex<HashMap<String, CacheEntry>> =
        Mutex::new(HashMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CacheStatus {
    Pending,
    Loaded,
    Failed,
}

struct CacheEntry {
    status: CacheStatus,
    sender: watch::Sender<CacheStatus>,
    receiver: watch::Receiver<CacheStatus>,
}

pub struct ImageCache;

impl ImageCache {
    pub fn init_base_cover_size(scale_factor: f64) {
        let size = (300.0 * scale_factor).round() as u32;

        let store = app_store();
        let prior: Option<u32> = store
            .get(STORE_KEY_DPI)
            .and_then(|v| serde_json::from_value(v).ok());

        if let Some(prior_size) = prior {
            if prior_size != size {
                crate::info!(
                    "DPI changed ({} → {}). Invalidating music image cache.",
                    prior_size,
                    size
                );
                Self::invalidate_cache();
            }
        }

        store.set(STORE_KEY_DPI.to_string(), serde_json::json!(size));
        let _ = store.save();

        let _ = BASE_COVER_SIZE.set(size);

        let permits = available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .saturating_sub(2)
            .max(1);
        let _ = RESIZE_SEMAPHORE.set(Semaphore::new(permits));
    }

    pub fn base_cover_size() -> u32 {
        *BASE_COVER_SIZE.get().unwrap_or(&300)
    }

    pub fn resize_semaphore() -> &'static Semaphore {
        RESIZE_SEMAPHORE.get_or_init(|| Semaphore::new(1))
    }

    pub fn get_cache_dir() -> PathBuf {
        let dir = app_handle()
            .path()
            .app_cache_dir()
            .expect("Failed to get app cache dir")
            .join("coverarts")
            .join("local");
        std::fs::create_dir_all(&dir).expect("Failed to create music image cache dir");
        dir
    }

    pub fn invalidate_cache() {
        let dir = Self::get_cache_dir();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    pub fn get_cache_key(artist: Option<&str>, album: Option<&str>, path: &str) -> String {
        if let (Some(a), Some(b)) = (artist, album) {
            let raw = format!("{} {}", a, b);
            Self::sanitize_filename(&raw)
        } else {
            let mut hash: u64 = 5381;
            for byte in path.bytes() {
                hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
            }
            format!("{:016x}", hash)
        }
    }

    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c => c,
            })
            .collect()
    }

    pub fn read_cache(key: &str) -> Option<Vec<u8>> {
        let path = Self::get_cache_dir().join(key);
        std::fs::read(path).ok()
    }

    pub fn write_cache(key: &str, bytes: &[u8]) {
        let path = Self::get_cache_dir().join(key);
        if let Err(e) = std::fs::write(&path, bytes) {
            crate::warn!("Failed to write music image cache {}: {}", key, e);
        }
    }

    pub fn queue_get(key: &str) -> Option<CacheStatus> {
        let queue = IMAGE_CACHE_QUEUE.lock().unwrap();
        queue.get(key).map(|e| e.status)
    }

    pub fn queue_get_receiver(key: &str) -> Option<watch::Receiver<CacheStatus>> {
        let queue = IMAGE_CACHE_QUEUE.lock().unwrap();
        queue.get(key).map(|e| e.receiver.clone())
    }

    pub fn queue_set(key: &str, status: CacheStatus) {
        let mut queue = IMAGE_CACHE_QUEUE.lock().unwrap();
        if let Some(entry) = queue.get_mut(key) {
            entry.status = status;
            let _ = entry.sender.send(status);
        } else {
            let (sender, receiver) = watch::channel(status);
            queue.insert(
                key.to_string(),
                CacheEntry {
                    status,
                    sender,
                    receiver,
                },
            );
        }
    }

    pub async fn queue_wait(key: &str) -> CacheStatus {
        let rx = {
            let queue = IMAGE_CACHE_QUEUE.lock().unwrap();
            queue.get(key).map(|e| e.receiver.clone())
        };

        if let Some(mut rx) = rx {
            loop {
                let status = *rx.borrow();
                if status != CacheStatus::Pending {
                    return status;
                }
                if rx.changed().await.is_err() {
                    return CacheStatus::Failed;
                }
            }
        }

        CacheStatus::Failed
    }
}
