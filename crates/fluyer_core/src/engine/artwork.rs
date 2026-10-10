//! Cover art: resolution (cache, embedded, online) and memoized thumbnails.

use super::FluyerEngine;
use crate::metadata::MusicMetadata;
use crate::view_models;
use std::sync::Arc;

impl FluyerEngine {
    pub fn artwork_track_thumbnail(&self, index: usize, max_size: u32) -> Option<Arc<Vec<u8>>> {
        self.artwork_cached_thumbnail(
            format!("track:{}", index),
            || {
                self.library
                    .read()
                    .unwrap()
                    .get_by_index(index)
                    .and_then(|t| self.artwork_resolve_track(&t, Some(index)))
            },
            max_size,
        )
    }

    pub fn artwork_album_thumbnail(&self, index: usize, max_size: u32) -> Option<Arc<Vec<u8>>> {
        self.artwork_cached_thumbnail(
            format!("album:{}", index),
            || {
                self.library
                    .read()
                    .unwrap()
                    .album_get_by_index(index)
                    .and_then(|a| self.artwork_resolve_album(&a, Some(index)))
            },
            max_size,
        )
    }

    pub fn artwork_current_thumbnail(&self, max_size: u32) -> Option<Arc<Vec<u8>>> {
        let track = self.player.get_current_track()?;
        self.artwork_cached_thumbnail(
            format!("path:{}", track.path),
            || self.artwork_resolve_track(&track, None),
            max_size,
        )
    }

    // ponytail: memoized down-scaled JPEG cover. `key` identifies the source
    // cover so repeated scroll renders of the same row are allocation-free.
    pub fn artwork_cached_thumbnail(
        &self,
        key: String,
        source: impl FnOnce() -> Option<Vec<u8>>,
        max_size: u32,
    ) -> Option<Arc<Vec<u8>>> {
        let cache_key = (key, max_size);
        if let Some(hit) = self.thumbnail_cache.read().unwrap().get(&cache_key) {
            return Some(hit);
        }
        let bytes = source()?;
        let encoded = view_models::thumbnail_jpeg(&bytes, max_size)?;
        let arc = Arc::new(encoded);
        self.thumbnail_cache
            .write()
            .unwrap()
            .put(cache_key, Arc::clone(&arc));
        Some(arc)
    }

    pub fn artwork_resolve_track(
        &self,
        track: &MusicMetadata,
        notify_index: Option<usize>,
    ) -> Option<Vec<u8>> {
        let artist = track.artist.as_deref().unwrap_or("");
        let album = track.album.as_deref();
        let title = track.title.as_deref();

        // 1. Check disk cache first (fast, hits SSD/OS cache, 0 HDD reads)
        if let Some(cached) =
            self.cover_art
                .get_cached_with_fallback(artist, album, title, Some(&track.path))
        {
            return Some(cached);
        }

        // 2. Extract embedded cover art with Lofty (only parses header, avoids decoding audio)
        if let Ok(bytes) = MusicMetadata::get_image_with_lofty(&track.path) {
            let _ = self
                .cover_art
                .save_to_cache(artist, album, title, Some(&track.path), &bytes);
            return Some(bytes);
        }

        // 3. Fallback: async fetch online from MusicBrainz / Cover Art Archive
        let cover_service = Arc::clone(&self.cover_art);
        let sink = self.event_sink.clone();
        let artist_s = artist.to_string();
        let album_s = album.map(|s| s.to_string());
        let title_s = title.map(|s| s.to_string());
        self.runtime.spawn(async move {
            if let Ok(Some(_)) = cover_service
                .fetch_and_cache(&artist_s, album_s.as_deref(), title_s.as_deref())
                .await
            {
                if let (Some(s), Some(idx)) = (sink, notify_index) {
                    s.on_track_cover_loaded(idx);
                }
            }
        });
        None
    }

    pub fn artwork_resolve_album(
        &self,
        album: &[MusicMetadata],
        notify_index: Option<usize>,
    ) -> Option<Vec<u8>> {
        let first_track = album.first()?;
        let artist = first_track
            .album_artist
            .as_deref()
            .or(first_track.artist.as_deref())
            .unwrap_or("");
        let album_name = first_track.album.as_deref();

        // 1. Check disk cache first
        if let Some(cached) = self.cover_art.get_cached_with_fallback(
            artist,
            album_name,
            None,
            Some(&first_track.path),
        ) {
            return Some(cached);
        }

        // 2. Extract embedded cover art from first track with Lofty
        if let Ok(bytes) = MusicMetadata::get_image_with_lofty(&first_track.path) {
            let _ = self.cover_art.save_to_cache(
                artist,
                album_name,
                None,
                Some(&first_track.path),
                &bytes,
            );
            return Some(bytes);
        }

        // 3. Fallback: async fetch online
        let cover_service = Arc::clone(&self.cover_art);
        let sink = self.event_sink.clone();
        let artist_s = artist.to_string();
        let album_s = album_name.map(|s| s.to_string());
        self.runtime.spawn(async move {
            if let Ok(Some(_)) = cover_service
                .fetch_and_cache(&artist_s, album_s.as_deref(), None)
                .await
            {
                if let (Some(s), Some(idx)) = (sink, notify_index) {
                    s.on_album_cover_loaded(idx);
                }
            }
        });
        None
    }
}
