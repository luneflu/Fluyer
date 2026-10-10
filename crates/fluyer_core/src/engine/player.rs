//! Playback controls and the player bar / play view snapshots.

use super::FluyerEngine;
use crate::audio::RepeatMode;
use crate::{metadata, services, view_models};

impl FluyerEngine {
    pub fn player_toggle_play(&self) {
        self.player.toggle_play();
    }

    pub fn player_next(&self) {
        self.player.next();
    }

    pub fn player_previous(&self) {
        self.player.previous();
    }

    pub fn player_get_position(&self) -> u64 {
        self.player.get_sync_info(false).position_ms()
    }

    pub fn player_seek(&self, position_ms: u64) {
        self.player.set_pos(position_ms);
    }

    pub fn player_set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    /// None -> All -> One -> None.
    pub fn player_cycle_repeat(&self) {
        let next = match self.player.get_sync_info(false).repeat_mode {
            RepeatMode::None => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::None,
        };
        self.player.set_repeat_mode(next);
    }

    pub fn player_shuffle(&self) {
        self.player.shuffle_track();
    }

    pub fn player_get_bar(&self) -> view_models::PlayerBarViewModel {
        let sync = self.player.get_sync_info(false);
        let current_track = self.player.get_current_track();
        let (title, artist, album) = match current_track {
            Some(ref t) => (
                t.title
                    .clone()
                    .unwrap_or_else(|| metadata::DEFAULT_TITLE.to_string()),
                t.artist
                    .clone()
                    .unwrap_or_else(|| metadata::DEFAULT_ARTIST.to_string()),
                t.album.clone().unwrap_or_default(),
            ),
            None => ("No Track".to_string(), String::new(), String::new()),
        };

        let position_ms = sync.position_ms();
        let duration_ms = sync.duration_ms();
        let progress_pct = if duration_ms > 0 {
            (position_ms as f32 / duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let time_label = format!(
            "{} / {}",
            view_models::format_time(position_ms),
            view_models::format_time(duration_ms)
        );

        view_models::PlayerBarViewModel {
            track_index: sync.index,
            title,
            artist,
            album,
            position_ms,
            duration_ms,
            progress_pct,
            time_label,
            is_playing: sync.is_playing,
            repeat_mode: sync.repeat_mode.into(),
            is_shuffled: sync.is_shuffled,
            volume: self.player.get_volume(),
        }
    }

    pub fn player_get_play_view(&self) -> view_models::PlayViewModel {
        let current_track = self.player.get_current_track();
        let track_vm = current_track.as_ref().map(|t| {
            let sync = self.player.get_sync_info(false);
            let idx = if sync.index >= 0 {
                sync.index as usize
            } else {
                0
            };
            view_models::TrackItemViewModel::from_metadata(idx, t, true)
        });

        let lyrics = self.lyrics_get();
        let pos_ms = self.player.get_sync_info(false).position_ms();
        let current_lyric_index = view_models::find_active_lyric_index(&lyrics, pos_ms);

        let colors = match current_track.as_ref() {
            Some(t) => self.backdrop_palette(t),
            None => vec![services::background::balance_color([30, 30, 40], true)],
        };
        let palette = colors
            .into_iter()
            .map(|c| view_models::ColorRgb {
                r: c[0],
                g: c[1],
                b: c[2],
            })
            .collect();

        view_models::PlayViewModel {
            track: track_vm,
            lyrics,
            current_lyric_index,
            palette,
        }
    }
}
