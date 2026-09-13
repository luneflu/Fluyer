use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricQuery {
    pub title: String,
    pub artist: String,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LrcLibResult {
    pub id: Option<i64>,
    pub name: String,
    #[serde(rename = "trackName")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "albumName")]
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
}

pub struct LyricService {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl LyricService {
    pub fn new(cache_dir: &Path) -> Self {
        let lyric_cache = cache_dir.join("lyrics");
        let _ = std::fs::create_dir_all(&lyric_cache);
        Self {
            cache_dir: lyric_cache,
            client: reqwest::Client::new(),
        }
    }

    pub fn get_cached(&self, query: &LyricQuery) -> Option<String> {
        let key = self.generate_cache_key(query);
        let path = self.cache_dir.join(key);
        std::fs::read_to_string(path).ok()
    }

    pub async fn fetch(&self, query: LyricQuery) -> Option<String> {
        if let Some(cached) = self.get_cached(&query) {
            return Some(cached);
        }

        let primary_artist = query.artist.split(" • ").next().unwrap_or(&query.artist);
        let search_queries = vec![
            format!("{} {}", query.title, primary_artist),
            query.title.clone(),
        ];

        let mut all_results: Vec<LrcLibResult> = Vec::new();

        for search_query in search_queries {
            let url = format!(
                "https://lrclib.net/api/search?q={}",
                urlencoding::encode(&search_query)
            );

            if let Ok(response) = self.client.get(&url).send().await {
                if let Ok(results) = response.json::<Vec<LrcLibResult>>().await {
                    all_results.extend(results);
                    if all_results.len() >= 3 {
                        break;
                    }
                }
            }
        }

        if all_results.is_empty() {
            return None;
        }

        let mut scored_results: Vec<(LrcLibResult, f64)> = all_results
            .into_iter()
            .filter(|r| r.synced_lyrics.is_some())
            .map(|r| {
                let score = self.score_result(&r, &query);
                (r, score)
            })
            .filter(|(_, score)| *score > 0.4)
            .collect();

        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_result = scored_results.into_iter().next()?;
        let lyrics = best_result.0.synced_lyrics?;

        let key = self.generate_cache_key(&query);
        let path = self.cache_dir.join(key);
        let _ = std::fs::write(path, &lyrics);

        Some(lyrics)
    }

    fn generate_cache_key(&self, query: &LyricQuery) -> String {
        let primary_artist = query.artist.split(" • ").next().unwrap_or(&query.artist);
        let key = format!("{} - {}.lrc", primary_artist, query.title);
        key.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn score_result(&self, result: &LrcLibResult, query: &LyricQuery) -> f64 {
        if let (Some(res_dur), Some(q_dur)) = (result.duration, query.duration) {
            let diff = (res_dur - (q_dur as f64 / 1000.0)).abs();
            if diff > 3.0 {
                return 0.0;
            }
        }

        let mut score = 0.0;
        let title_sim = calculate_similarity(&normalize(&result.name), &normalize(&query.title));
        score += title_sim * 0.4;

        let primary_artist = query.artist.split(" • ").next().unwrap_or(&query.artist);
        let artist_sim = calculate_similarity(
            &normalize(&result.artist_name),
            &normalize(primary_artist),
        );
        score += artist_sim * 0.3;

        if let (Some(res_dur), Some(q_dur)) = (result.duration, query.duration) {
            let diff = (res_dur - (q_dur as f64 / 1000.0)).abs();
            let dur_sim = (1.0 - diff / 3.0).max(0.0);
            score += dur_sim * 0.3;
        }

        score
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn levenshtein(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let m = s1_chars.len();
    let n = s2_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut matrix = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..=m {
        matrix[0][i] = i;
    }
    for j in 0..=n {
        matrix[j][0] = j;
    }

    for j in 1..=n {
        for i in 1..=m {
            let indicator = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[j][i] = (matrix[j][i - 1] + 1)
                .min(matrix[j - 1][i] + 1)
                .min(matrix[j - 1][i - 1] + indicator);
        }
    }
    matrix[n][m]
}

fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    let longer = if s1.len() > s2.len() { s1 } else { s2 };
    let shorter = if s1.len() > s2.len() { s2 } else { s1 };

    if longer.is_empty() {
        return 1.0;
    }

    let distance = levenshtein(longer, shorter);
    (longer.len() - distance) as f64 / longer.len() as f64
}
