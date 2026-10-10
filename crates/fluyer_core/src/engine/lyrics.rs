//! Lyrics: resolution (file, embedded, online) and the parsed per-track cache.

use super::FluyerEngine;
use crate::metadata::{self, MusicMetadata};
use crate::{services, view_models};
use std::sync::Arc;

impl FluyerEngine {
    /// Active lyric index without cloning the parsed lyric vector; this runs on
    /// the UI's 250ms position tick, so the Arc path matters.
    pub fn lyrics_get_active_index(&self, position_ms: u64) -> i32 {
        match self.lyrics_get_arc() {
            Some(lines) => view_models::find_active_lyric_index(&lines, position_ms),
            None => -1,
        }
    }

    pub fn lyrics_get(&self) -> Vec<view_models::LyricLine> {
        self.lyrics_get_arc()
            .as_ref()
            .map(|lines| lines.as_ref().clone())
            .unwrap_or_default()
    }

    pub fn lyrics_get_arc(&self) -> Option<Arc<Vec<view_models::LyricLine>>> {
        let track = self.player.get_current_track()?;

        if let Some((cached_path, lines)) = self.lyrics_cache.read().unwrap().as_ref() {
            if cached_path == &track.path {
                return Some(Arc::clone(lines));
            }
        }

        let lrc = self.lyrics_resolve(&track)?;
        let lines = Arc::new(view_models::parse_lrc(&lrc));
        *self.lyrics_cache.write().unwrap() = Some((track.path.clone(), Arc::clone(&lines)));
        Some(lines)
    }

    pub fn lyrics_resolve(&self, track: &MusicMetadata) -> Option<String> {
        if let Some(lyrics) = metadata::probe::extract_lyrics_file(&track.path) {
            return Some(lyrics);
        }
        if let Some(lyrics) = metadata::probe::extract_lyrics_embedded(&track.path) {
            return Some(lyrics);
        }
        let title = track.title.as_deref().unwrap_or("");
        let artist = track.artist.as_deref().unwrap_or("");
        let duration = track.duration.map(|d| d as u64);
        let query = services::lyric::LyricQuery {
            title: title.to_string(),
            artist: artist.to_string(),
            duration,
        };
        if let Some(cached) = self.lyrics.get_cached(&query) {
            return Some(cached);
        }
        let lyric_service = Arc::clone(&self.lyrics);
        let sink = self.event_sink.clone();
        self.runtime.spawn(async move {
            if let Some(fetched) = lyric_service.fetch(query).await {
                if let Some(s) = sink {
                    s.on_lyrics_loaded(&fetched);
                }
            }
        });
        None
    }
}
