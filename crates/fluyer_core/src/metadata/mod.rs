pub mod parser;
pub mod probe;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MusicMetadata {
    pub id: i64,
    pub path: String,
    pub duration: Option<u128>,
    pub filename: Option<String>,

    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub bits_per_sample: Option<u32>,
    pub sample_rate: Option<u32>,
    pub image: Option<String>,

    pub extra_tags: Option<HashMap<String, Option<String>>>,
}

pub const DEFAULT_TITLE: &str = "Unknown Title";
pub const DEFAULT_ARTIST: &str = "Unknown Artist";

impl MusicMetadata {
    pub fn default_title() -> &'static str {
        DEFAULT_TITLE
    }

    pub fn default_artist() -> &'static str {
        DEFAULT_ARTIST
    }

    pub fn artist_separator() -> &'static str {
        ";"
    }

    pub fn separator() -> &'static str {
        " • "
    }

    pub fn set_ffmpeg_paths(ffmpeg: PathBuf, ffprobe: PathBuf) {
        probe::set_ffmpeg_paths(ffmpeg, ffprobe);
    }

    pub fn ffmpeg_path() -> Option<&'static PathBuf> {
        probe::ffmpeg_path()
    }

    pub fn ffprobe_path() -> Option<&'static PathBuf> {
        probe::ffprobe_path()
    }

    pub async fn get(path: String) -> Result<Self, String> {
        let path_clone = path.clone();

        let sym_res = tokio::task::spawn_blocking(move || {
            probe::probe_symphonia(&path_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        let mut metadata = match sym_res {
            Ok(meta) => meta,
            Err(err) => {
                log::warn!("Symphonia failed for {}: {}", path, err);
                if Self::ffprobe_path().is_some() {
                    probe::probe_ffmpeg(&path).await?
                } else {
                    return Err(err);
                }
            }
        };

        let title_missing = metadata
            .title
            .as_ref()
            .is_none_or(|t| t.trim().is_empty());
        let artist_missing = metadata
            .artist
            .as_ref()
            .is_none_or(|a| a.trim().is_empty());

        if title_missing || artist_missing {
            if let Some(file_name) = Path::new(&metadata.path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                let (parsed_artist, parsed_title) =
                    parser::get_artist_title_from_file_name(file_name);

                if title_missing {
                    metadata.title = Some(parsed_title);
                }
                if artist_missing {
                    if let Some(artist) = parsed_artist {
                        metadata.artist = Some(artist);
                    }
                }
            }
        }

        Ok(metadata)
    }

    pub fn get_with_symphonia(path: &str) -> Result<Self, String> {
        probe::probe_symphonia(path)
    }

    pub fn get_image_with_symphonia(path: &str) -> Result<Vec<u8>, String> {
        probe::extract_image_symphonia(path)
    }

    pub fn get_image_with_lofty(path: &str) -> Result<Vec<u8>, String> {
        probe::extract_image_lofty(path)
    }

    pub async fn get_image_from_path(path: String) -> Result<Vec<u8>, String> {
        probe::extract_image(&path).await
    }

    pub fn get_artist_title_from_file_name(file_name: &str) -> (Option<String>, String) {
        parser::get_artist_title_from_file_name(file_name)
    }

    pub fn get_lyrics_from_path(path: &str) -> Option<String> {
        probe::extract_lyrics_file(path)
    }

    pub fn get_embedded_lyrics_from_path(path: &str) -> Option<String> {
        probe::extract_lyrics_embedded(path)
    }
}
