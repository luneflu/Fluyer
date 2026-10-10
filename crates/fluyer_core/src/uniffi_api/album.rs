use super::FluyerAppEngine;
use crate::view_models::{AlbumCardViewModel, AlbumDetailViewModel};

#[uniffi::export]
impl FluyerAppEngine {
    pub fn album_get_count(&self) -> u64 {
        self.inner.album_get_count() as u64
    }

    pub fn album_get_card(&self, index: u64) -> Option<AlbumCardViewModel> {
        self.inner.album_get_card(index as usize)
    }

    pub fn album_get_detail(&self, index: u64) -> Option<AlbumDetailViewModel> {
        self.inner.album_get_detail(index as usize)
    }

    pub fn album_play(&self, index: u64) {
        self.inner.album_play(index as usize);
    }

    pub fn album_queue(&self, index: u64) {
        self.inner.album_queue(index as usize);
    }

    pub fn album_shuffle(&self, index: u64) {
        self.inner.album_shuffle(index as usize);
    }
}
