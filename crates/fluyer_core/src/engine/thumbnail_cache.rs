//! Bounded FIFO cache of encoded thumbnails, keyed by (source key, max size).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

const THUMBNAIL_CACHE_CAPACITY: usize = 256;

pub(super) struct ThumbnailCache {
    map: HashMap<(String, u32), Arc<Vec<u8>>>,
    order: VecDeque<(String, u32)>,
}

impl ThumbnailCache {
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub(super) fn get(&self, key: &(String, u32)) -> Option<Arc<Vec<u8>>> {
        self.map.get(key).cloned()
    }

    pub(super) fn put(&mut self, key: (String, u32), value: Arc<Vec<u8>>) {
        if self.map.contains_key(&key) {
            return;
        }
        if self.order.len() >= THUMBNAIL_CACHE_CAPACITY {
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, value);
    }
}
