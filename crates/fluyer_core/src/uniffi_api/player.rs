use super::FluyerAppEngine;
use crate::view_models::{PlayViewModel, PlayerBarViewModel};

#[uniffi::export]
impl FluyerAppEngine {
    pub fn player_toggle_play(&self) {
        self.inner.player_toggle_play();
    }

    pub fn player_next(&self) {
        self.inner.player_next();
    }

    pub fn player_previous(&self) {
        self.inner.player_previous();
    }

    pub fn player_seek(&self, position_ms: u64) {
        self.inner.player_seek(position_ms);
    }

    pub fn player_get_position(&self) -> u64 {
        self.inner.player_get_position()
    }

    pub fn player_set_volume(&self, volume: f32) {
        self.inner.player_set_volume(volume);
    }

    pub fn player_cycle_repeat(&self) {
        self.inner.player_cycle_repeat();
    }

    pub fn player_shuffle(&self) {
        self.inner.player_shuffle();
    }

    /// Bottom player bar: title, artist, progress, play state, volume.
    pub fn player_get_bar(&self) -> PlayerBarViewModel {
        self.inner.player_get_bar()
    }

    /// Full-screen play view: current track, lyrics, palette.
    pub fn player_get_play_view(&self) -> PlayViewModel {
        self.inner.player_get_play_view()
    }
}
