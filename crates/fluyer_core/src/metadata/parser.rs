use regex::Regex;
use std::sync::LazyLock;

static CLEANUP_SUFFIXES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s*[\[\(](?:official\s*(?:video|audio|music\s*video|lyric\s*video)?|lyrics?|hd|hq|4k|1080p|720p|audio|video|mv|m/v)[\]\)]\s*$").unwrap()
});

static YOUTUBE_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\s+[a-zA-Z0-9_-]{11}|[\[\(][a-zA-Z0-9_-]{11}[\]\)])\s*$").unwrap()
});

static METADATA_CLEANUP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:\s*[\[\(](?:official|lyrics?|hd|hq|audio|video|mv|m/v|remaster(?:ed)?|remix|live|acoustic|cover|instrumental|extended|edit|version|ver\.?|mix)(?:\s+\w+)*[\]\)])+\s*$").unwrap()
});

static LEADING_TRACK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d+[\.\-\s]+").unwrap()
});

struct PatternPair {
    regex: Regex,
    is_reversed: bool,
}

static PATTERNS: LazyLock<Vec<PatternPair>> = LazyLock::new(|| {
    vec![
        PatternPair {
            regex: Regex::new(r"^\d+[\.\-\s]+(.+?)\s+[-–—]\s+(.+)$").unwrap(),
            is_reversed: false,
        },
        PatternPair {
            regex: Regex::new(r"^(.+?)\s+[-–—]\s+(.+?)(?:\s+[\(\[].*[\)\]])*$").unwrap(),
            is_reversed: false,
        },
        PatternPair {
            regex: Regex::new(r"^(.+?)\s+[–—]\s+(.+)$").unwrap(),
            is_reversed: false,
        },
        PatternPair {
            regex: Regex::new(r"^(.+?)\s+-\s+(.+)$").unwrap(),
            is_reversed: false,
        },
        PatternPair {
            regex: Regex::new(r"(?i)^(.+?)\s+by\s+(.+)$").unwrap(),
            is_reversed: true,
        },
        PatternPair {
            regex: Regex::new(r"(?i)^(.+?)\s+(?:ft\.?|feat\.?|featuring)\s+(.+)$").unwrap(),
            is_reversed: true,
        },
        PatternPair {
            regex: Regex::new(r"^([^_]+)_([^_]+)$").unwrap(),
            is_reversed: false,
        },
        PatternPair {
            regex: Regex::new(r"^(.+?)\s*~\s*(.+)$").unwrap(),
            is_reversed: false,
        },
    ]
});

pub fn get_artist_title_from_file_name(file_name: &str) -> (Option<String>, String) {
    let without_extension = file_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(file_name);

    let cleaned = CLEANUP_SUFFIXES.replace_all(without_extension, "");
    let cleaned = YOUTUBE_ID_RE.replace_all(&cleaned, "").trim().to_string();

    for pair in PATTERNS.iter() {
        if let Some(captures) = pair.regex.captures(&cleaned) {
            let (first, second) = (
                captures.get(1).map_or("", |m| m.as_str()).trim(),
                captures.get(2).map_or("", |m| m.as_str()).trim(),
            );

            if first.is_empty() || second.is_empty() {
                continue;
            }

            let (mut artist, mut title) = if pair.is_reversed {
                (second.to_string(), first.to_string())
            } else {
                (first.to_string(), second.to_string())
            };

            artist = METADATA_CLEANUP.replace_all(&artist, "").trim().to_string();
            title = METADATA_CLEANUP.replace_all(&title, "").trim().to_string();
            title = LEADING_TRACK_RE.replace(&title, "").trim().to_string();

            if !artist.is_empty() && !title.is_empty() {
                return (Some(artist), title);
            }
        }
    }

    let fallback_title = if cleaned.is_empty() {
        without_extension.trim().to_string()
    } else {
        let title = METADATA_CLEANUP.replace_all(&cleaned, "").trim().to_string();
        let title = LEADING_TRACK_RE.replace(&title, "").trim().to_string();
        if title.is_empty() {
            without_extension.trim().to_string()
        } else {
            title
        }
    };

    (None, fallback_title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artist_and_title() {
        let (artist, title) = get_artist_title_from_file_name("Queen - Bohemian Rhapsody (Official Video).mp3");
        assert_eq!(artist.as_deref(), Some("Queen"));
        assert_eq!(title, "Bohemian Rhapsody");

        let (artist, title) = get_artist_title_from_file_name("01. Daft Punk - Get Lucky.flac");
        assert_eq!(artist.as_deref(), Some("Daft Punk"));
        assert_eq!(title, "Get Lucky");

        let (artist, title) = get_artist_title_from_file_name("Yellow by Coldplay.m4a");
        assert_eq!(artist.as_deref(), Some("Coldplay"));
        assert_eq!(title, "Yellow");

        let (artist, title) = get_artist_title_from_file_name("Coldplay - Clocks [dQw4w9WgXcQ].mp3");
        assert_eq!(artist.as_deref(), Some("Coldplay"));
        assert_eq!(title, "Clocks");

        let (artist, title) = get_artist_title_from_file_name("JustASongTitle.mp3");
        assert_eq!(artist, None);
        assert_eq!(title, "JustASongTitle");
    }
}
