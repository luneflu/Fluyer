use super::album_card::AlbumCardViewModel;
use super::formatters::{format_album_label, format_duration};
use super::track_item::TrackItemViewModel;
use crate::metadata::MusicMetadata;
use serde::{Deserialize, Serialize};

#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumDetailViewModel {
    pub header: AlbumCardViewModel,
    pub duration_ms: u64,
    pub total_duration_formatted: String,
    pub subtitle: String,
    pub tracks: Vec<TrackItemViewModel>,
}

impl AlbumDetailViewModel {
    pub fn from_tracks(
        index: usize,
        tracks: &[MusicMetadata],
        current_track_path: Option<&str>,
    ) -> Self {
        let header = AlbumCardViewModel::from_tracks(index, tracks);

        let mut duration_ms = 0u64;
        let mut track_vms = Vec::with_capacity(tracks.len());

        for (i, t) in tracks.iter().enumerate() {
            let dur = t.duration.unwrap_or(0) as u64;
            duration_ms += dur;
            let is_cur = current_track_path
                .map(|p| p == t.path)
                .unwrap_or(false);
            track_vms.push(TrackItemViewModel::from_metadata(i, t, is_cur));
        }

        let total_duration_formatted = format_duration(duration_ms);
        let subtitle = format_album_label(
            &header.name,
            &header.artist,
            &header.year,
            duration_ms,
        );

        Self {
            header,
            duration_ms,
            total_duration_formatted,
            subtitle,
            tracks: track_vms,
        }
    }
}
