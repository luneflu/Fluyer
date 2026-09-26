use crate::metadata::MusicMetadata;
use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumCardViewModel {
    pub index: u64,
    pub name: String,
    pub artist: String,
    pub year: String,
    pub track_count: u64,
    pub track_count_label: String,
}

impl AlbumCardViewModel {
    pub fn from_tracks(index: usize, tracks: &[MusicMetadata]) -> Self {
        if tracks.is_empty() {
            return Self {
                index: index as u64,
                name: "Unknown Album".to_string(),
                artist: MusicMetadata::default_artist().to_string(),
                year: String::new(),
                track_count: 0,
                track_count_label: "0 tracks".to_string(),
            };
        }

        let first = &tracks[0];
        let name = first
            .album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string());
        let artist = first
            .album_artist
            .clone()
            .or_else(|| first.artist.clone())
            .unwrap_or_else(|| MusicMetadata::default_artist().to_string());
        let year = first
            .date
            .as_ref()
            .map(|d| {
                if d.len() >= 4 {
                    d[0..4].to_string()
                } else {
                    d.clone()
                }
            })
            .unwrap_or_default();

        let track_count = tracks.len() as u64;
        let track_count_label = if track_count == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", track_count)
        };

        Self {
            index: index as u64,
            name,
            artist,
            year,
            track_count,
            track_count_label,
        }
    }
}
