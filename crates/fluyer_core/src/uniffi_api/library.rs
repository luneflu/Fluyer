use super::FluyerAppEngine;
use crate::view_models::{ScanStatusViewModel, TrackItemViewModel};

/// Song list and music folders.
#[uniffi::export]
impl FluyerAppEngine {
    pub fn library_scan(&self, directories: Vec<String>) {
        self.inner.library_scan(&directories);
    }

    /// Drops every row under `directory` and rebuilds the library synchronously.
    pub fn library_remove_folder(&self, directory: String) {
        self.inner.library_remove_folder(&directory);
    }

    // ponytail: always idle; live progress arrives via `FluyerEvent::ScanProgress`.
    pub fn library_get_scan_status(&self) -> ScanStatusViewModel {
        ScanStatusViewModel::idle()
    }

    pub fn library_get_track_count(&self) -> u64 {
        self.inner.library_get_track_count() as u64
    }

    pub fn library_get_track(&self, index: u64) -> Option<TrackItemViewModel> {
        self.inner.library_get_track(index as usize)
    }

    pub fn library_play_all(&self, start_index: u64) {
        self.inner.library_play_all(start_index as usize);
    }

    /// Plays library tracks in the caller's (sorted) order.
    pub fn library_play_tracks(&self, indices: Vec<u64>, start_index: u64) {
        let indices: Vec<usize> = indices.into_iter().map(|i| i as usize).collect();
        self.inner.library_play_tracks(&indices, start_index as usize);
    }
}
