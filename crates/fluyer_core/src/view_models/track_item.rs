use super::formatters::format_duration;
use crate::metadata::MusicMetadata;
use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackItemViewModel {
    pub index: u64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub duration_formatted: String,
    pub is_current: bool,
}

impl TrackItemViewModel {
    pub fn from_metadata(index: usize, meta: &MusicMetadata, is_current: bool) -> Self {
        let duration_ms = meta.duration.unwrap_or(0) as u64;
        Self {
            index: index as u64,
            path: meta.path.clone(),
            title: meta
                .title
                .clone()
                .unwrap_or_else(|| MusicMetadata::default_title().to_string()),
            artist: meta
                .artist
                .clone()
                .unwrap_or_else(|| MusicMetadata::default_artist().to_string()),
            album: meta.album.clone().unwrap_or_default(),
            duration_ms,
            duration_formatted: format_duration(duration_ms),
            is_current,
        }
    }
}
