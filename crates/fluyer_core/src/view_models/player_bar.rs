use super::common::NativeRepeatMode;
use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerBarViewModel {
    pub track_index: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub progress_pct: f32,
    pub time_label: String,
    pub is_playing: bool,
    pub repeat_mode: NativeRepeatMode,
    pub is_shuffled: bool,
    pub volume: f32,
}
