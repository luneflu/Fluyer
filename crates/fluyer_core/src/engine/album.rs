//! Albums: lookup and album-level playback.

use super::FluyerEngine;
use crate::view_models;

impl FluyerEngine {
    pub fn album_get_count(&self) -> usize {
        self.library.read().unwrap().album_count()
    }

    pub fn album_get_card(&self, index: usize) -> Option<view_models::AlbumCardViewModel> {
        self.library
            .read()
            .unwrap()
            .album_get_by_index(index)
            .map(|tracks| view_models::AlbumCardViewModel::from_tracks(index, &tracks))
    }

    pub fn album_get_detail(&self, index: usize) -> Option<view_models::AlbumDetailViewModel> {
        let current_path = self.player.get_current_track().map(|t| t.path);
        self.library
            .read()
            .unwrap()
            .album_get_by_index(index)
            .map(|tracks| {
                view_models::AlbumDetailViewModel::from_tracks(
                    index,
                    &tracks,
                    current_path.as_deref(),
                )
            })
    }

    pub fn album_play(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.clear();
            self.player.add_track_no_auto_play(album);
            self.player.goto_track(0);
        }
    }

    pub fn album_play_track(&self, album_index: usize, track_index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(album_index) {
            if track_index < album.len() {
                self.player.clear();
                self.player.add_track_no_auto_play(album);
                self.player.goto_track(track_index);
            }
        }
    }

    pub fn album_queue(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.add_track(album);
        }
    }

    pub fn album_shuffle(&self, index: usize) {
        if let Some(album) = self.library.read().unwrap().album_get_by_index(index) {
            if album.is_empty() {
                return;
            }
            self.player.clear();
            self.player.add_track_no_auto_play(album);
            self.player.shuffle_track();
            self.player.goto_track(0);
        }
    }
}
