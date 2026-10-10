use super::FluyerAppEngine;
use crate::view_models::TrackItemViewModel;

/// Queue panel ("Now Playing" in the UI).
#[uniffi::export]
impl FluyerAppEngine {
    /// Queue in play order; `index` on each row is the queue position.
    pub fn queue_get(&self) -> Vec<TrackItemViewModel> {
        self.inner.queue_get()
    }

    pub fn queue_goto(&self, index: u64) {
        self.inner.queue_goto(index as usize);
    }

    pub fn queue_remove(&self, index: u64) {
        self.inner.queue_remove(index as usize);
    }

    pub fn queue_move(&self, from: u64, to: u64) {
        self.inner.queue_move(from as usize, to as usize);
    }

    pub fn queue_clear(&self) {
        self.inner.queue_clear();
    }
}
