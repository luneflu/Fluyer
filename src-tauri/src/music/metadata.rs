use dotenvy_macro::dotenv;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag, StandardVisualKey};
use tauri::Manager;
use tokio::process::Command;

#[cfg(target_os = "android")]
use tauri_plugin_fluyer::FluyerExt;

use crate::state::app_handle;

static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();
static FFPROBE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MusicMetadata {
    pub id: i16,
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

const DEFAULT_TITLE: &str = dotenv!("VITE_DEFAULT_MUSIC_TITLE");
const DEFAULT_ARTIST: &str = dotenv!("VITE_DEFAULT_MUSIC_ARTIST");

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

    fn create_command(program: &Path) -> Command {
        let mut cmd = Command::new(program);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd
    }

    pub fn ffmpeg_path() -> &'static PathBuf {
        FFMPEG_PATH
            .get()
            .expect("initialize_ffmpeg_paths not called")
    }

    pub fn ffprobe_path() -> &'static PathBuf {
        FFPROBE_PATH
            .get()
            .expect("initialize_ffmpeg_paths not called")
    }

    pub fn initialize_ffmpeg_paths() {
        #[cfg(target_os = "linux")]
        let (ffmpeg_path, ffprobe_path) = {
            // /usr/lib/fluyer is root-owned and may be read-only, so copy to a
            // user-writable dir and chmod there or exec fails with os error 13.
            let src_dir = PathBuf::from("/usr/lib/fluyer");
            let bin_dir = app_handle()
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory")
                .join("bin");

            let _ = std::fs::create_dir_all(&bin_dir);

            use std::os::unix::fs::PermissionsExt;
            for src in ["ffmpeg", "ffprobe"] {
                let src_path = src_dir.join(src);
                if !src_path.exists() {
                    continue;
                }
                let dst_path = bin_dir.join(src);
                let copied = match std::fs::copy(&src_path, &dst_path) {
                    Ok(_) => {
                        let mut perms = std::fs::metadata(&dst_path)
                            .map(|m| m.permissions())
                            .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o644));
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&dst_path, perms).is_ok()
                    }
                    Err(e) => {
                        crate::warn!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e);
                        false
                    }
                };
                if copied {
                    crate::info!("Bundled {} prepared at {:?}", src, dst_path);
                }
            }
            (bin_dir.join("ffmpeg"), bin_dir.join("ffprobe"))
        };

        #[cfg(not(target_os = "linux"))]
        let (ffmpeg_path, ffprobe_path) = {
            let dir = app_handle()
                .path()
                .resource_dir()
                .expect("Failed to get resource directory")
                .join("libs/ffmpeg/bin");
            (dir.join("ffmpeg"), dir.join("ffprobe"))
        };

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in &[&ffmpeg_path, &ffprobe_path] {
                if let Ok(metadata) = std::fs::metadata(path) {
                    let mut perms = metadata.permissions();
                    if perms.mode() & 0o111 == 0 {
                        perms.set_mode(0o755);
                        if let Err(e) = std::fs::set_permissions(path, perms) {
                            crate::warn!("Failed to set permissions for {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        FFMPEG_PATH.set(ffmpeg_path).unwrap();
        FFPROBE_PATH.set(ffprobe_path).unwrap();
    }

    /// Get metadata, trying Symphonia first for performance, falling back to FFmpeg

    pub async fn get(path: String) -> Result<Self, String> {
        let path_clone = path.clone();

        let mut metadata = tokio::task::spawn_blocking(move || {
            // Try Symphonia first (fast, pure Rust)
            match Self::get_with_symphonia(&path_clone).map_err(|e| (e, path_clone.clone())) {
                Ok(metadata) => Ok(metadata),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        // Handle fallback if Symphonia failed
        if let Err((error, path)) = metadata {
            crate::warn!("{}", error);
            metadata = Self::get_with_ffmpeg(path.clone())
                .await
                .map_err(|e| (e, path));
        }

        let mut metadata = metadata.map_err(|(e, _)| e)?;

        // Check if artist or title is missing
        let title_missing = metadata
            .title
            .as_ref()
            .map_or(true, |t| t.trim().is_empty());
        let artist_missing = metadata
            .artist
            .as_ref()
            .map_or(true, |a| a.trim().is_empty());

        if title_missing || artist_missing {
            if let Some(file_name) = Path::new(&metadata.path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                let (parsed_artist, parsed_title) =
                    Self::get_artist_title_from_file_name(file_name);

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

    /// Extract metadata using Symphonia (pure Rust, fast)
    fn get_with_symphonia(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint to help the probe
        let mut hint = Hint::new();
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, format_opts, metadata_opts)
            .map_err(|e| format!("Symphonia probe failed {} : {}", path, e))?;

        let mut metadata = MusicMetadata::default();
        metadata.path = path.to_string();

        // Find first audio track and extract codec parameters
        let track = format
            .first_track(TrackType::Audio)
            .ok_or_else(|| "No audio track found".to_string())?;

        // Extract duration from track.duration + time_base
        // duration in seconds = ticks * (numer / denom)
        if let Some(dur) = track.duration {
            if let Some(tb) = track.time_base {
                let duration_secs =
                    dur.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
                metadata.duration = Some((duration_secs * 1000.0) as u128);
            }
        }

        // Extract sample rate and bits per sample from AudioCodecParameters
        if let Some(CodecParameters::Audio(audio_params)) = &track.codec_params {
            metadata.sample_rate = audio_params.sample_rate;
            metadata.bits_per_sample = audio_params.bits_per_sample;
        }

        // Extract metadata tags from format metadata
        let extract_tags = |meta: &symphonia::core::meta::MetadataRevision,
                            metadata: &mut MusicMetadata| {
            for tag in &meta.media.tags {
                if let Some(std_tag) = &tag.std {
                    match std_tag {
                        StandardTag::TrackTitle(v) => metadata.title = Some(v.as_ref().clone()),
                        StandardTag::Artist(v) => metadata.artist = Some(v.as_ref().clone()),
                        StandardTag::Album(v) => metadata.album = Some(v.as_ref().clone()),
                        StandardTag::AlbumArtist(v) => {
                            metadata.album_artist = Some(v.as_ref().clone())
                        }
                        StandardTag::TrackNumber(n) => metadata.track_number = Some(n.to_string()),
                        StandardTag::Genre(v) => metadata.genre = Some(v.as_ref().clone()),
                        StandardTag::RecordingDate(v) => metadata.date = Some(v.as_ref().clone()),
                        StandardTag::ReleaseDate(v) if metadata.date.is_none() => {
                            metadata.date = Some(v.as_ref().clone())
                        }
                        _ => {}
                    }
                }
            }
        };

        // Prefer ID3v2 tags over ID3v1 to avoid encoding issues with Cyrillic characters
        // ID3v1 is strictly ISO-8859-1 (or system local), which symphonia decodes as ISO-8859-1 lossy.
        // It replaces non-ISO-8859-1 characters (like Cyrillic Windows-1251) with '?' or replacement chars.
        let mut metadatas = format.metadata();
        let mut best_score = -1;
        
        // We will accumulate the best metadata found across all revisions
        let mut best_temp_meta = MusicMetadata::default();
        
        while let Some(rev) = metadatas.current() {
            let score = match rev.info.short_name {
                "id3v2" => 10,
                "vorbis" | "mp4" | "flac" => 8,
                "ape" | "apev2" => 5,
                "id3v1" => 1,
                _ => 0,
            };
            
            if score > best_score {
                best_score = score;
                best_temp_meta = MusicMetadata::default();
                extract_tags(rev, &mut best_temp_meta);
            }
            
            if metadatas.pop().is_none() {
                break;
            }
        }
        
        // Apply the best metadata tags found
        metadata.title = best_temp_meta.title;
        metadata.artist = best_temp_meta.artist;
        metadata.album = best_temp_meta.album;
        metadata.album_artist = best_temp_meta.album_artist;
        metadata.track_number = best_temp_meta.track_number;
        metadata.genre = best_temp_meta.genre;
        metadata.date = best_temp_meta.date;

        Ok(metadata)
    }

    /// Fallback metadata extraction using FFmpeg
    async fn get_with_ffmpeg(path: String) -> Result<Self, String> {
        let output = Self::create_command(Self::ffprobe_path())
            .args(&[
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                &path,
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

        if !output.status.success() {
            return Err(format!("ffprobe failed with status: {}", output.status));
        }
        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: Value =
            serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Self::parse_json_metadata(json, path)
    }

    fn parse_json_metadata(json: Value, path: String) -> Result<MusicMetadata, String> {
        // Verify this is actually an audio file with at least one audio stream
        let has_audio_stream = json
            .get("streams")
            .and_then(|v| v.as_array())
            .map(|streams| {
                streams
                    .iter()
                    .any(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio"))
            })
            .unwrap_or(false);

        if !has_audio_stream {
            return Err("No audio stream found in file".to_string());
        }

        let mut metadata = MusicMetadata::default();
        metadata.path = path;

        // Extract format tags
        if let Some(format) = json.get("format") {
            if let Some(tags) = format.get("tags") {
                metadata.title = Self::extract_tag(tags, &["title", "TITLE", "Title"]);
                metadata.artist = Self::extract_tag(tags, &["artist", "ARTIST", "Artist"]);
                metadata.album_artist =
                    Self::extract_tag(tags, &["album_artist", "ALBUM_ARTIST", "ALBUMARTIST"]);
                metadata.album = Self::extract_tag(tags, &["album", "ALBUM", "Album"]);
                metadata.track_number = Self::extract_tag(tags, &["track", "TRACK", "TRACKNUMBER"]);
            }

            // Extract duration
            if let Some(duration_str) = format.get("duration").and_then(|v| v.as_str()) {
                if let Ok(duration_secs) = duration_str.parse::<f64>() {
                    metadata.duration = Some((duration_secs * 1000.0) as u128);
                }
            }
        }

        // Extract stream information (bit depth, sample rate)
        if let Some(streams) = json.get("streams").and_then(|v| v.as_array()) {
            for stream in streams {
                if stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio") {
                    // Sample rate
                    if let Some(sample_rate) = stream.get("sample_rate").and_then(|v| v.as_str()) {
                        if let Ok(rate) = sample_rate.parse::<f64>() {
                            metadata.sample_rate = Some(rate as u32);
                        }
                    }

                    // Bit depth
                    if let Some(bits_per_sample) =
                        stream.get("bits_per_raw_sample").and_then(|v| v.as_str())
                    {
                        if let Ok(bits) = bits_per_sample.parse::<i32>() {
                            if bits > 0 {
                                metadata.bits_per_sample = Some(bits as u32);
                            }
                        }
                    } else if let Some(bits_per_sample) =
                        stream.get("bits_per_sample").and_then(|v| v.as_i64())
                    {
                        if bits_per_sample > 0 {
                            metadata.bits_per_sample = Some(bits_per_sample as u32);
                        }
                    }

                    break; // Use first audio stream
                }
            }
        }

        Ok(metadata)
    }

    /// Extract tag value from multiple possible keys (case-insensitive)
    fn extract_tag(tags: &Value, keys: &[&str]) -> Option<String> {
        // First try exact case-sensitive match
        for key in keys {
            if let Some(value) = tags.get(key).and_then(|v| v.as_str()) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        // Then try case-insensitive match across all tag keys
        if let Some(obj) = tags.as_object() {
            for key in keys {
                let key_lower = key.to_lowercase();
                for (tag_key, tag_value) in obj {
                    if tag_key.to_lowercase() == key_lower {
                        if let Some(value) = tag_value.as_str() {
                            let trimmed = value.trim();
                            if !trimmed.is_empty() {
                                return Some(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    // pub fn get_default_cover_art_path() -> String {
    //     let resource_dir = app_handle()
    //         .path()
    //         .resource_dir()
    //         .expect("Failed to get resource directory");
    //     let default_cover_art_path = resource_dir.join("resources/images/default-cover-art.png");
    //     default_cover_art_path.to_str().unwrap().to_string()
    // }

    // pub fn get_default_cover_art() -> Result<Vec<u8>, std::io::Error> {
    //     use tauri_plugin_fs::FsExt;
    //     app_handle()
    //         .fs()
    //         .read(PathBuf::from(Self::get_default_cover_art_path()))
    // }

    /// Extract cover art, trying Symphonia first for performance, falling back to FFmpeg
    pub async fn get_image_from_path(path: String) -> Result<Vec<u8>, String> {
        let path_clone = path.clone();

        let result = tokio::task::spawn_blocking(move || {
            // Try Symphonia first (fast, pure Rust)
            Self::get_image_with_symphonia(&path_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok(image) => Ok(image),
            Err(_) => {
                // Fallback to FFmpeg for unsupported formats (e.g., MP4 container artwork)
                Self::get_image_with_ffmpeg(path).await
            }
        }
    }

    /// Extract cover art using Symphonia (pure Rust, fast)
    pub fn get_image_with_symphonia(path: &str) -> Result<Vec<u8>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, format_opts, metadata_opts)
            .map_err(|e| format!("Symphonia probe failed: {}", e))?;

        // Helper to extract front cover from metadata revision
        let extract_cover = |meta: &symphonia::core::meta::MetadataRevision| -> Option<Vec<u8>> {
            for visual in &meta.media.visuals {
                // Prefer front cover, but accept any visual
                if visual.usage == Some(StandardVisualKey::FrontCover) || visual.usage.is_none() {
                    return Some(visual.data.to_vec());
                }
            }
            // If no front cover found, return first visual
            meta.media.visuals.first().map(|v| v.data.to_vec())
        };

        // Try format metadata
        if let Some(rev) = format.metadata().current() {
            if let Some(cover) = extract_cover(rev) {
                return Ok(cover);
            }
        }

        Err(format!("No cover art found in file: {}", path))
    }

    /// Fallback cover art extraction using FFmpeg
    async fn get_image_with_ffmpeg(path: String) -> Result<Vec<u8>, String> {
        // First check if the file has any video stream (cover art)
        let probe_output = Self::create_command(Self::ffprobe_path())
            .args(&[
                "-v",
                "quiet",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_type",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                &path,
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to probe for cover art {} : {}", path, e))?;

        // If no video stream found, return error early
        if probe_output.stdout.is_empty() || !probe_output.status.success() {
            return Err(format!("No cover art in file {}", path));
        }

        // Try to copy the embedded image without re-encoding (fastest)
        let output = Self::create_command(Self::ffmpeg_path())
            .args(&[
                "-i",
                &path,
                "-an", // Disable audio
                "-c:v",
                "copy", // Copy video codec (don't re-encode)
                "-vframes",
                "1", // Extract only first frame
                "-f",
                "image2pipe", // Output to pipe
                "-",
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute ffmpeg {} : {}", path, e))?;

        if output.status.success() && !output.stdout.is_empty() {
            return Ok(output.stdout);
        }

        // If copy fails, try BMP encoder as fallback
        let output = Self::create_command(Self::ffmpeg_path())
            .args(&[
                "-i",
                &path,
                "-an", // Disable audio
                "-c:v",
                "bmp", // Use BMP encoder (built-in, no external deps)
                "-vframes",
                "1", // Extract only first frame
                "-f",
                "image2pipe", // Output to pipe
                "-",
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute ffmpeg {} : {}", path, e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract cover art {} : {}",
                path,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        if output.stdout.is_empty() {
            return Err(format!("No cover art in file: {}", path));
        }

        Ok(output.stdout)
    }

    /// Get metadata using FFprobeKit on Android
    #[cfg(target_os = "android")]
    pub async fn get_android(path: String) -> Result<Self, String> {
        let path_clone = path.clone();
        tokio::task::spawn_blocking(move || {
            let result = app_handle()
                .fluyer()
                .metadata_get(path_clone.clone())
                .map_err(|e| format!("Failed to get metadata: {}", e))?;

            match result.value {
                Some(json_str) => {
                    let json: Value = serde_json::from_str(&json_str)
                        .map_err(|e| format!("Failed to parse JSON from Android: {}", e))?;
                    Self::parse_json_metadata(json, path_clone)
                }
                None => Err("No metadata returned from Android".to_string()),
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    /// Extract cover art using FFmpegKit on Android
    #[cfg(target_os = "android")]
    pub async fn get_image_from_path_android(path: String) -> Result<Vec<u8>, String> {
        // Try Symphonia first (fast, pure Rust)
        if let Ok(image) = Self::get_image_with_symphonia(&path) {
            return Ok(image);
        }

        tokio::task::spawn_blocking(move || {
            let result = app_handle()
                .fluyer()
                .metadata_get_image(path.clone())
                .map_err(|e| format!("Failed to get image: {}", e))?;

            match result.path {
                Some(temp_path) => {
                    let bytes = std::fs::read(&temp_path)
                        .map_err(|e| format!("Failed to read image file: {}", e))?;
                    // Clean up temp file
                    let _ = std::fs::remove_file(&temp_path);
                    Ok(bytes)
                }
                None => Err(format!("No cover art in file: {}", path)),
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    /// Parses artist and title from a filename.
    /// Returns (Option<artist>, title) - artist may be None if only title could be extracted.
    fn get_artist_title_from_file_name(file_name: &str) -> (Option<String>, String) {
        let without_extension = file_name
            .rsplit_once('.')
            .map(|(name, _)| name)
            .unwrap_or(file_name);

        // Pre-cleanup: Remove common suffixes that don't affect parsing
        let cleanup_suffixes =
            Regex::new(r"(?i)\s*[\[\(](?:official\s*(?:video|audio|music\s*video|lyric\s*video)?|lyrics?|hd|hq|4k|1080p|720p|audio|video|mv|m/v)[\]\)]\s*$").unwrap();
        let youtube_id_re = Regex::new(r"\s*[\[\(]?[a-zA-Z0-9_-]{11}[\]\)]?\s*$").unwrap();

        let mut cleaned = cleanup_suffixes
            .replace_all(without_extension, "")
            .to_string();
        // Remove YouTube video IDs (11 character alphanumeric at end)
        cleaned = youtube_id_re.replace_all(&cleaned, "").trim().to_string();

        // Patterns ordered from most specific to least specific
        let patterns: Vec<(Regex, bool)> = vec![
            // Track number prefixed: "01 - Artist - Title" or "01. Artist - Title"
            (
                Regex::new(r"^\d+[\.\-\s]+(.+?)\s+[-–—]\s+(.+)$").unwrap(),
                false,
            ),
            // "Artist - Title (feat. Someone)" or "Artist - Title (Remix)"
            (
                Regex::new(r"^(.+?)\s+[-–—]\s+(.+?)(?:\s+[\(\[].*[\)\]])*$").unwrap(),
                false,
            ),
            // "Artist – Title" (en-dash) or "Artist — Title" (em-dash)
            (Regex::new(r"^(.+?)\s+[–—]\s+(.+)$").unwrap(), false),
            // "Artist - Title"
            (Regex::new(r"^(.+?)\s+-\s+(.+)$").unwrap(), false),
            // "Title by Artist" (reverse order)
            (Regex::new(r"(?i)^(.+?)\s+by\s+(.+)$").unwrap(), true),
            // "Title ft. Artist" or "Title feat. Artist" or "Title featuring Artist"
            (
                Regex::new(r"(?i)^(.+?)\s+(?:ft\.?|feat\.?|featuring)\s+(.+)$").unwrap(),
                true,
            ),
            // "Artist_Title" (underscore separator)
            (Regex::new(r"^([^_]+)_([^_]+)$").unwrap(), false),
            // "Artist ~ Title" (tilde separator, sometimes used)
            (Regex::new(r"^(.+?)\s*~\s*(.+)$").unwrap(), false),
        ];

        // Regex to clean up trailing metadata from both artist and title
        let metadata_cleanup =
            Regex::new(r"(?i)(?:\s*[\[\(](?:official|lyrics?|hd|hq|audio|video|mv|m/v|remaster(?:ed)?|remix|live|acoustic|cover|instrumental|extended|edit|version|ver\.?|mix)(?:\s+\w+)*[\]\)])+\s*$").unwrap();
        // Clean up leading track numbers from title
        let leading_track_re = Regex::new(r"^\d+[\.\-\s]+").unwrap();

        for (pattern, is_reversed) in patterns {
            if let Some(captures) = pattern.captures(&cleaned) {
                let (first, second) = (
                    captures.get(1).map_or("", |m| m.as_str()).trim(),
                    captures.get(2).map_or("", |m| m.as_str()).trim(),
                );

                if first.is_empty() || second.is_empty() {
                    continue;
                }

                let (mut artist, mut title) = if is_reversed {
                    (second.to_string(), first.to_string())
                } else {
                    (first.to_string(), second.to_string())
                };

                // Clean up metadata from both
                artist = metadata_cleanup.replace_all(&artist, "").trim().to_string();
                title = metadata_cleanup.replace_all(&title, "").trim().to_string();
                title = leading_track_re.replace(&title, "").trim().to_string();

                // Validate we still have meaningful content
                if !artist.is_empty() && !title.is_empty() {
                    return (Some(artist), title);
                }
            }
        }

        // Fallback: just use the cleaned filename as title
        let fallback_title = if cleaned.is_empty() {
            without_extension.trim().to_string()
        } else {
            // Final cleanup for standalone title
            let title = metadata_cleanup
                .replace_all(&cleaned, "")
                .trim()
                .to_string();
            let title = leading_track_re.replace(&title, "").trim().to_string();
            if title.is_empty() {
                without_extension.trim().to_string()
            } else {
                title
            }
        };

        (None, fallback_title)
    }

    pub fn get_lyrics_from_path(path: String) -> Option<String> {
        let lyrics_path = Path::new(&path).with_extension("lrc");
        if let Ok(lyrics) = std::fs::read_to_string(&lyrics_path) {
            return Some(lyrics);
        }
        None
    }

    /// Extract embedded lyrics from audio file metadata using Symphonia
    pub fn get_embedded_lyrics_from_path(path: &str) -> Option<String> {
        let file = File::open(path).ok()?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, format_opts, metadata_opts)
            .ok()?;

        // Helper to extract lyrics from metadata revision
        let extract_lyrics = |meta: &symphonia::core::meta::MetadataRevision| -> Option<String> {
            for tag in &meta.media.tags {
                // Check for standard lyrics key
                if let Some(StandardTag::Lyrics(v)) = &tag.std {
                    if !v.is_empty() {
                        return Some(v.as_ref().clone());
                    }
                }
                // Also check by tag name for various lyrics tag formats
                let tag_key_lower = tag.raw.key.to_lowercase();
                if tag_key_lower.contains("lyrics") || tag_key_lower.contains("unsyncedlyrics") {
                    let value = tag.raw.value.to_string();
                    if !value.is_empty() {
                        return Some(value);
                    }
                }
            }
            None
        };

        // Try format metadata
        if let Some(rev) = format.metadata().current() {
            if let Some(lyrics) = extract_lyrics(rev) {
                return Some(lyrics);
            }
        }

        None
    }
}
