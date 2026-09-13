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
use tokio::process::Command;

static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();
static FFPROBE_PATH: OnceLock<PathBuf> = OnceLock::new();

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
        let _ = FFMPEG_PATH.set(ffmpeg);
        let _ = FFPROBE_PATH.set(ffprobe);
    }

    pub fn ffmpeg_path() -> Option<&'static PathBuf> {
        FFMPEG_PATH.get()
    }

    pub fn ffprobe_path() -> Option<&'static PathBuf> {
        FFPROBE_PATH.get()
    }

    fn create_command(program: &Path) -> Command {
        #[allow(unused_mut)]
        let mut cmd = Command::new(program);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd
    }

    /// Extract metadata using Symphonia first, falling back to FFmpeg if configured
    pub async fn get(path: String) -> Result<Self, String> {
        let path_clone = path.clone();

        let sym_res = tokio::task::spawn_blocking(move || {
            Self::get_with_symphonia(&path_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        let mut metadata = match sym_res {
            Ok(meta) => meta,
            Err(err) => {
                log::warn!("Symphonia failed for {}: {}", path, err);
                if Self::ffprobe_path().is_some() {
                    Self::get_with_ffmpeg(path.clone()).await?
                } else {
                    return Err(err);
                }
            }
        };

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

    pub fn get_with_symphonia(path: &str) -> Result<Self, String> {
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
            .map_err(|e| format!("Symphonia probe failed {} : {}", path, e))?;

        let mut metadata = MusicMetadata::default();
        metadata.path = path.to_string();

        let track = format
            .first_track(TrackType::Audio)
            .ok_or_else(|| "No audio track found".to_string())?;

        if let Some(dur) = track.duration {
            if let Some(tb) = track.time_base {
                let duration_secs =
                    dur.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
                metadata.duration = Some((duration_secs * 1000.0) as u128);
            }
        }

        if let Some(CodecParameters::Audio(audio_params)) = &track.codec_params {
            metadata.sample_rate = audio_params.sample_rate;
            metadata.bits_per_sample = audio_params.bits_per_sample;
        }

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

        let mut metadatas = format.metadata();
        let mut best_score = -1;
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

        metadata.title = best_temp_meta.title;
        metadata.artist = best_temp_meta.artist;
        metadata.album = best_temp_meta.album;
        metadata.album_artist = best_temp_meta.album_artist;
        metadata.track_number = best_temp_meta.track_number;
        metadata.genre = best_temp_meta.genre;
        metadata.date = best_temp_meta.date;

        Ok(metadata)
    }

    async fn get_with_ffmpeg(path: String) -> Result<Self, String> {
        let ffprobe = Self::ffprobe_path().ok_or("ffprobe path not set")?;
        let output = Self::create_command(ffprobe)
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

        if let Some(format) = json.get("format") {
            if let Some(tags) = format.get("tags") {
                metadata.title = Self::extract_tag(tags, &["title", "TITLE", "Title"]);
                metadata.artist = Self::extract_tag(tags, &["artist", "ARTIST", "Artist"]);
                metadata.album_artist =
                    Self::extract_tag(tags, &["album_artist", "ALBUM_ARTIST", "ALBUMARTIST"]);
                metadata.album = Self::extract_tag(tags, &["album", "ALBUM", "Album"]);
                metadata.track_number = Self::extract_tag(tags, &["track", "TRACK", "TRACKNUMBER"]);
            }

            if let Some(duration_str) = format.get("duration").and_then(|v| v.as_str()) {
                if let Ok(duration_secs) = duration_str.parse::<f64>() {
                    metadata.duration = Some((duration_secs * 1000.0) as u128);
                }
            }
        }

        if let Some(streams) = json.get("streams").and_then(|v| v.as_array()) {
            for stream in streams {
                if stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio") {
                    if let Some(sample_rate) = stream.get("sample_rate").and_then(|v| v.as_str()) {
                        if let Ok(rate) = sample_rate.parse::<f64>() {
                            metadata.sample_rate = Some(rate as u32);
                        }
                    }

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
                    break;
                }
            }
        }

        Ok(metadata)
    }

    fn extract_tag(tags: &Value, keys: &[&str]) -> Option<String> {
        for key in keys {
            if let Some(value) = tags.get(key).and_then(|v| v.as_str()) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

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

        let extract_cover = |meta: &symphonia::core::meta::MetadataRevision| -> Option<Vec<u8>> {
            for visual in &meta.media.visuals {
                if visual.usage == Some(StandardVisualKey::FrontCover) || visual.usage.is_none() {
                    return Some(visual.data.to_vec());
                }
            }
            meta.media.visuals.first().map(|v| v.data.to_vec())
        };

        if let Some(rev) = format.metadata().current() {
            if let Some(cover) = extract_cover(rev) {
                return Ok(cover);
            }
        }

        Err(format!("No cover art found in file: {}", path))
    }

    pub async fn get_image_from_path(path: String) -> Result<Vec<u8>, String> {
        let path_clone = path.clone();
        let result = tokio::task::spawn_blocking(move || {
            Self::get_image_with_symphonia(&path_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok(img) => Ok(img),
            Err(_) => {
                if Self::ffmpeg_path().is_some() {
                    Self::get_image_with_ffmpeg(path).await
                } else {
                    Err("No cover art found and FFmpeg not configured".to_string())
                }
            }
        }
    }

    async fn get_image_with_ffmpeg(path: String) -> Result<Vec<u8>, String> {
        let ffmpeg = Self::ffmpeg_path().ok_or("ffmpeg path not set")?;
        let ffprobe = Self::ffprobe_path().ok_or("ffprobe path not set")?;

        let probe_output = Self::create_command(ffprobe)
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

        if probe_output.stdout.is_empty() || !probe_output.status.success() {
            return Err(format!("No cover art in file {}", path));
        }

        let output = Self::create_command(ffmpeg)
            .args(&[
                "-i",
                &path,
                "-an",
                "-c:v",
                "copy",
                "-vframes",
                "1",
                "-f",
                "image2pipe",
                "-",
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute ffmpeg {} : {}", path, e))?;

        if output.status.success() && !output.stdout.is_empty() {
            return Ok(output.stdout);
        }

        Err(format!("Failed to extract cover art from: {}", path))
    }

    pub fn get_artist_title_from_file_name(file_name: &str) -> (Option<String>, String) {
        let without_extension = file_name
            .rsplit_once('.')
            .map(|(name, _)| name)
            .unwrap_or(file_name);

        let cleanup_suffixes =
            Regex::new(r"(?i)\s*[\[\(](?:official\s*(?:video|audio|music\s*video|lyric\s*video)?|lyrics?|hd|hq|4k|1080p|720p|audio|video|mv|m/v)[\]\)]\s*$").unwrap();
        let youtube_id_re = Regex::new(r"\s*[\[\(]?[a-zA-Z0-9_-]{11}[\]\)]?\s*$").unwrap();

        let mut cleaned = cleanup_suffixes
            .replace_all(without_extension, "")
            .to_string();
        cleaned = youtube_id_re.replace_all(&cleaned, "").trim().to_string();

        let patterns: Vec<(Regex, bool)> = vec![
            (
                Regex::new(r"^\d+[\.\-\s]+(.+?)\s+[-–—]\s+(.+)$").unwrap(),
                false,
            ),
            (
                Regex::new(r"^(.+?)\s+[-–—]\s+(.+?)(?:\s+[\(\[].*[\)\]])*$").unwrap(),
                false,
            ),
            (Regex::new(r"^(.+?)\s+[–—]\s+(.+)$").unwrap(), false),
            (Regex::new(r"^(.+?)\s+-\s+(.+)$").unwrap(), false),
            (Regex::new(r"(?i)^(.+?)\s+by\s+(.+)$").unwrap(), true),
            (
                Regex::new(r"(?i)^(.+?)\s+(?:ft\.?|feat\.?|featuring)\s+(.+)$").unwrap(),
                true,
            ),
            (Regex::new(r"^([^_]+)_([^_]+)$").unwrap(), false),
            (Regex::new(r"^(.+?)\s*~\s*(.+)$").unwrap(), false),
        ];

        let metadata_cleanup =
            Regex::new(r"(?i)(?:\s*[\[\(](?:official|lyrics?|hd|hq|audio|video|mv|m/v|remaster(?:ed)?|remix|live|acoustic|cover|instrumental|extended|edit|version|ver\.?|mix)(?:\s+\w+)*[\]\)])+\s*$").unwrap();
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

                artist = metadata_cleanup.replace_all(&artist, "").trim().to_string();
                title = metadata_cleanup.replace_all(&title, "").trim().to_string();
                title = leading_track_re.replace(&title, "").trim().to_string();

                if !artist.is_empty() && !title.is_empty() {
                    return (Some(artist), title);
                }
            }
        }

        let fallback_title = if cleaned.is_empty() {
            without_extension.trim().to_string()
        } else {
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

    pub fn get_lyrics_from_path(path: &str) -> Option<String> {
        let lyrics_path = Path::new(path).with_extension("lrc");
        std::fs::read_to_string(lyrics_path).ok()
    }

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

        let extract_lyrics = |meta: &symphonia::core::meta::MetadataRevision| -> Option<String> {
            for tag in &meta.media.tags {
                if let Some(StandardTag::Lyrics(v)) = &tag.std {
                    if !v.is_empty() {
                        return Some(v.as_ref().clone());
                    }
                }
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

        if let Some(rev) = format.metadata().current() {
            if let Some(lyrics) = extract_lyrics(rev) {
                return Some(lyrics);
            }
        }

        None
    }
}
