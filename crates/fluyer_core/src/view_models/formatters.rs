use super::common::LyricLine;

pub fn format_time(ms: u64) -> String {
    let total_sec = ms / 1000;
    let min = total_sec / 60;
    let sec = total_sec % 60;
    format!("{}:{:02}", min, sec)
}

pub fn format_duration(ms: u64) -> String {
    format_time(ms)
}

pub fn format_album_label(name: &str, artist: &str, year: &str, duration_ms: u64) -> String {
    let mut parts = Vec::new();
    if !name.is_empty() {
        parts.push(name);
    }
    if !artist.is_empty() {
        parts.push(artist);
    }
    if !year.is_empty() {
        parts.push(year);
    }
    let dur_str;
    if duration_ms > 0 {
        dur_str = format_duration(duration_ms);
        parts.push(&dur_str);
    }
    parts.join(" • ")
}

// ponytail: parse basic LRC format with [mm:ss.xx] timestamps; upgrade to full LRC parser if syllable sync needed
pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
    if lrc_text.trim().is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();

    for raw_line in lrc_text.lines() {
        let line = raw_line.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }

        let Some(open_bracket) = line.find('[') else { continue };
        let Some(close_bracket) = line.find(']') else { continue };
        if close_bracket <= open_bracket {
            continue;
        }

        let tag = &line[open_bracket + 1..close_bracket];
        let Some(colon) = tag.find(':') else { continue };

        let Ok(minutes) = tag[..colon].parse::<f64>() else { continue };
        let Ok(seconds) = tag[colon + 1..].parse::<f64>() else { continue };
        let timestamp_ms = ((minutes * 60.0 + seconds) * 1000.0).round() as u64;

        let text = line[close_bracket + 1..].trim().to_string();

        if lines.is_empty() && timestamp_ms > 5000 {
            lines.push(LyricLine {
                timestamp_ms: 0,
                text: String::new(),
            });
        }

        lines.push(LyricLine { timestamp_ms, text });
    }

    lines.sort_by_key(|l| l.timestamp_ms);
    lines
}

pub fn find_active_lyric_index(lyrics: &[LyricLine], position_ms: u64) -> i32 {
    if lyrics.is_empty() {
        return -1;
    }
    if position_ms < lyrics[0].timestamp_ms {
        return 0;
    }
    // Binary search first line past position_ms
    let idx = lyrics.partition_point(|line| line.timestamp_ms <= position_ms);
    (idx - 1) as i32
}

// ponytail: fast triangle resize into raw RGBA8 bytes; switch to Lanczos3 if downscale aliasing noticed
pub fn resize_image_rgba(
    bytes: &[u8],
    target_w: u32,
    target_h: u32,
) -> Option<(Vec<u8>, u32, u32)> {
    if bytes.is_empty() || target_w == 0 || target_h == 0 {
        return None;
    }
    let img = image::load_from_memory(bytes).ok()?;
    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = resized.into_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc_and_active_index() {
        let lrc = r#"
[00:10.50]First line
[00:20.00]Second line
[01:05.12]Third line
"#;
        let lines = parse_lrc(lrc);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].timestamp_ms, 0);
        assert_eq!(lines[0].text, "");
        assert_eq!(lines[1].timestamp_ms, 10500);
        assert_eq!(lines[1].text, "First line");
        assert_eq!(lines[2].timestamp_ms, 20000);
        assert_eq!(lines[3].timestamp_ms, 65120);

        assert_eq!(find_active_lyric_index(&lines, 0), 0);
        assert_eq!(find_active_lyric_index(&lines, 5000), 0);
        assert_eq!(find_active_lyric_index(&lines, 10500), 1);
        assert_eq!(find_active_lyric_index(&lines, 15000), 1);
        assert_eq!(find_active_lyric_index(&lines, 20000), 2);
        assert_eq!(find_active_lyric_index(&lines, 65119), 2);
        assert_eq!(find_active_lyric_index(&lines, 70000), 3);
    }

    #[test]
    fn test_formatters() {
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(65_000), "1:05");
        assert_eq!(format_time(3665_000), "61:05");

        let label = format_album_label("Kind of Blue", "Miles Davis", "1959", 125_000);
        assert_eq!(label, "Kind of Blue • Miles Davis • 1959 • 2:05");
    }
}
