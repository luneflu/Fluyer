use crate::lyric::{cache, queue, request, types::*};
use crate::music::metadata::MusicMetadata;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Check if lyrics contain synced timestamps (LRC format: [MM:SS.xx] or [MM:SS])
fn has_synced_timestamps(lyrics: &str) -> bool {
    // Match patterns like [00:12.34] or [00:12] at the start of lines
    let timestamp_regex = Regex::new(r"^\[\d{1,2}:\d{2}(?:\.\d{1,3})?\]").unwrap();

    // Check if any line starts with a timestamp (more than just metadata like [ar:Artist])
    lyrics
        .lines()
        .any(|line| timestamp_regex.is_match(line.trim()))
}

/// Get lyrics for a track - returns synced lyrics text
/// Priority: .lrc file (synced) → embedded metadata (synced) → cached → LrcLib API
#[tauri::command]
pub async fn lyric_get(query: LyricQuery) -> Result<String, String> {
    if query.title.is_empty() {
        let err = "No title provided for lyric search".to_string();
        crate::warn!("{}", err);
        return Err(err);
    }

    // 1. Try .lrc file first (highest priority) - only if synced
    let lrc_path = Path::new(&query.path).with_extension("lrc");
    if let Ok(lyrics) = fs::read_to_string(&lrc_path) {
        if has_synced_timestamps(&lyrics) {
            crate::info!("Loaded synced lyrics from .lrc file: {:?}", lrc_path);
            return Ok(lyrics);
        }
        crate::info!("Skipping .lrc file (no timestamps): {:?}", lrc_path);
    }

    // 2. Try embedded lyrics from audio file metadata - only if synced
    if let Some(lyrics) = MusicMetadata::get_embedded_lyrics_from_path(&query.path) {
        if has_synced_timestamps(&lyrics) {
            crate::info!("Loaded synced embedded lyrics from: {}", query.path);
            return Ok(lyrics);
        }
        crate::info!("Skipping embedded lyrics (no timestamps): {}", query.path);
    }

    // 3. Try cache/API with queue system
    let cache_key = request::generate_cache_key(&query);
    let cache_path = format!("{}/{}", cache::get_cache_directory(), cache_key);

    // Check if there's an existing queue entry
    if let Some(queue_item) = queue::get_queue(&cache_key) {
        match queue_item.status {
            LyricRequestStatus::Loaded => {
                // Already fetched, return from cache
                if let Ok(lyrics) = fs::read_to_string(&cache_path) {
                    return Ok(lyrics);
                }
                let err = format!("Lyrics cache file not found: {}", cache_key);
                crate::warn!("{}", err);
                return Err(err);
            }
            LyricRequestStatus::Failed => {
                // Previous attempt failed
                let err = format!("Lyrics fetch previously failed: {}", cache_key);
                crate::warn!("{}", err);
                return Err(err);
            }
            LyricRequestStatus::Pending => {
                // Another request is in progress, wait for it
                let status = queue::wait_for_result(&cache_key).await;
                match status {
                    LyricRequestStatus::Loaded => {
                        if let Ok(lyrics) = fs::read_to_string(&cache_path) {
                            return Ok(lyrics);
                        }
                        let err = format!("Lyrics cache file not found: {}", cache_key);
                        crate::warn!("{}", err);
                        return Err(err);
                    }
                    _ => {
                        let err = format!("Lyrics fetch failed: {}", cache_key);
                        crate::warn!("{}", err);
                        return Err(err);
                    }
                }
            }
        }
    }

    // No queue entry - we're the first request
    // Try cache first
    if let Ok(lyrics) = fs::read_to_string(&cache_path) {
        queue::set_status(cache_key.clone(), LyricRequestStatus::Loaded);
        return Ok(lyrics);
    }

    // Mark as pending before starting the fetch
    queue::set_status(cache_key.clone(), LyricRequestStatus::Pending);

    // Request from API
    let lyrics = request::request_lyrics(query).await;

    if let Some(lyrics) = lyrics {
        queue::set_status(cache_key.clone(), LyricRequestStatus::Loaded);
        Ok(lyrics)
    } else {
        queue::set_status(cache_key.clone(), LyricRequestStatus::Failed);
        let err = format!("Failed to get lyrics for: {}", cache_key);
        crate::warn!("{}", err);
        Err(err)
    }
}
