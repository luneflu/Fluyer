use super::musicbrainz::MusicBrainz;
use std::path::{Path, PathBuf};

pub struct CoverArtService {
    cache_dir: PathBuf,
}

impl CoverArtService {
    pub fn new(cache_dir: &Path) -> Self {
        let art_cache = cache_dir.join("covers");
        let _ = std::fs::create_dir_all(&art_cache);
        Self {
            cache_dir: art_cache,
        }
    }

    pub fn get_cached(
        &self,
        artist: &str,
        album: Option<&str>,
        title: Option<&str>,
    ) -> Option<Vec<u8>> {
        let path = self.cache_path(artist, album, title);
        std::fs::read(path).ok()
    }

    pub async fn fetch_and_cache(
        &self,
        artist: &str,
        album: Option<&str>,
        title: Option<&str>,
    ) -> Result<Option<Vec<u8>>, String> {
        if let Some(cached) = self.get_cached(artist, album, title) {
            return Ok(Some(cached));
        }

        let url = MusicBrainz::get_cover_art_url(artist, album, title).await?;
        let Some(image_url) = url else {
            return Ok(None);
        };

        let res = reqwest::get(image_url).await.map_err(|e| e.to_string())?;
        let bytes = res.bytes().await.map_err(|e| e.to_string())?.to_vec();

        let path = self.cache_path(artist, album, title);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &bytes);

        Ok(Some(bytes))
    }

    fn cache_path(&self, artist: &str, album: Option<&str>, title: Option<&str>) -> PathBuf {
        let (folder, name) = if let Some(alb) = album {
            ("album", format!("{} - {}", artist, alb))
        } else if let Some(tit) = title {
            ("track", format!("{} - {}", artist, tit))
        } else {
            ("misc", artist.to_string())
        };

        let sanitized: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        self.cache_dir.join(folder).join(format!("{}.jpg", sanitized))
    }
}
