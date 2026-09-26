use super::MusicMetadata;
use serde_json::Value;
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

pub fn probe_symphonia(path: &str) -> Result<MusicMetadata, String> {
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

    let mut metadata = MusicMetadata {
        path: path.to_string(),
        ..Default::default()
    };

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

pub async fn probe_ffmpeg(path: &str) -> Result<MusicMetadata, String> {
    let ffprobe = ffprobe_path().ok_or("ffprobe path not set")?;
    let output = create_command(ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
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

    parse_json_metadata(json, path.to_string())
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

    let mut metadata = MusicMetadata {
        path,
        ..Default::default()
    };

    if let Some(format) = json.get("format") {
        if let Some(tags) = format.get("tags") {
            metadata.title = extract_tag(tags, &["title", "TITLE", "Title"]);
            metadata.artist = extract_tag(tags, &["artist", "ARTIST", "Artist"]);
            metadata.album_artist =
                extract_tag(tags, &["album_artist", "ALBUM_ARTIST", "ALBUMARTIST"]);
            metadata.album = extract_tag(tags, &["album", "ALBUM", "Album"]);
            metadata.track_number = extract_tag(tags, &["track", "TRACK", "TRACKNUMBER"]);
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

pub fn extract_image_symphonia(path: &str) -> Result<Vec<u8>, String> {
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

pub fn extract_image_lofty(path: &str) -> Result<Vec<u8>, String> {
    use lofty::config::ParseOptions;
    use lofty::file::TaggedFileExt;
    use lofty::picture::PictureType;
    use lofty::probe::Probe;

    let tagged_file = Probe::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?
        .options(ParseOptions::new().read_properties(false))
        .read()
        .map_err(|e| format!("Lofty failed to read tags: {}", e))?;

    // Check primary tag first
    if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
        for pic in tag.pictures() {
            if pic.pic_type() == PictureType::CoverFront {
                return Ok(pic.data().to_vec());
            }
        }
        if let Some(pic) = tag.pictures().first() {
            return Ok(pic.data().to_vec());
        }
    }

    // Check other tags in file if primary didn't contain an image
    for tag in tagged_file.tags() {
        for pic in tag.pictures() {
            if pic.pic_type() == PictureType::CoverFront {
                return Ok(pic.data().to_vec());
            }
        }
        if let Some(pic) = tag.pictures().first() {
            return Ok(pic.data().to_vec());
        }
    }

    Err(format!("No cover art found in file: {}", path))
}

pub async fn extract_image_ffmpeg(path: &str) -> Result<Vec<u8>, String> {
    let ffmpeg = ffmpeg_path().ok_or("ffmpeg path not set")?;
    let ffprobe = ffprobe_path().ok_or("ffprobe path not set")?;

    let probe_output = create_command(ffprobe)
        .args([
            "-v",
            "quiet",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to probe for cover art {} : {}", path, e))?;

    if probe_output.stdout.is_empty() || !probe_output.status.success() {
        return Err(format!("No cover art in file {}", path));
    }

    let output = create_command(ffmpeg)
        .args([
            "-i",
            path,
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

pub async fn extract_image(path: &str) -> Result<Vec<u8>, String> {
    let path_clone = path.to_string();
    let result = tokio::task::spawn_blocking(move || extract_image_symphonia(&path_clone))
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

    match result {
        Ok(img) => Ok(img),
        Err(_) => {
            if ffmpeg_path().is_some() {
                extract_image_ffmpeg(path).await
            } else {
                Err("No cover art found and FFmpeg not configured".to_string())
            }
        }
    }
}

pub fn extract_lyrics_file(path: &str) -> Option<String> {
    let lyrics_path = Path::new(path).with_extension("lrc");
    std::fs::read_to_string(lyrics_path).ok()
}

pub fn extract_lyrics_embedded(path: &str) -> Option<String> {
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
