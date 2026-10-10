//! Play queue (the "Now Playing" panel in the UI).

use super::FluyerEngine;
use crate::view_models;

impl FluyerEngine {
    /// Queue in play order; `index` is the queue position, not a library index.
    pub fn queue_get(&self) -> Vec<view_models::TrackItemViewModel> {
        let (tracks, current) = self.player.queue_snapshot();
        tracks
            .iter()
            .enumerate()
            .map(|(i, t)| view_models::TrackItemViewModel::from_metadata(i, t, current == Some(i)))
            .collect()
    }

    pub fn queue_goto(&self, index: usize) {
        if index < self.player.queue_count() {
            self.player.goto_track(index);
        }
    }

    pub fn queue_remove(&self, index: usize) {
        self.player.remove_track(index);
    }

    pub fn queue_move(&self, from: usize, to: usize) {
        self.player.move_track(from, to);
    }

    /// Stops playback and empties the queue; sync tells the UI the bar is empty.
    pub fn queue_clear(&self) {
        self.player.clear();
        self.player.emit_sync(false);
    }
}
