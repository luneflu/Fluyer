use super::common::{ColorRgb, LyricLine};
use super::track_item::TrackItemViewModel;
use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayViewModel {
    pub track: Option<TrackItemViewModel>,
    pub lyrics: Vec<LyricLine>,
    pub current_lyric_index: i32,
    pub palette: Vec<ColorRgb>,
}
