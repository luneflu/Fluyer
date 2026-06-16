pub mod commands;

use crate::music::metadata::MusicMetadata;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct LibraryState {
    pub music_list: Vec<MusicMetadata>,
    pub albums: Vec<Vec<MusicMetadata>>,
}

impl LibraryState {
    fn sort_key(m: &MusicMetadata) -> (String, u32, String) {
        let album = m.album.clone().unwrap_or_default();
        let track = m
            .track_number
            .as_deref()
            .and_then(|t| t.split('/').next())
            .and_then(|t| t.parse::<u32>().ok())
            .unwrap_or(u32::MAX);
        let filename = m.filename.clone().unwrap_or_default();
        (album, track, filename)
    }

    pub fn rebuild(&mut self, mut raw: Vec<MusicMetadata>) {
        // Sort flat list
        raw.sort_by(|a, b| Self::sort_key(a).cmp(&Self::sort_key(b)));
        self.music_list = raw;

        // Group into albums (only tracks with a non-empty album tag)
        use std::collections::BTreeMap;
        let mut map: BTreeMap<String, Vec<MusicMetadata>> = BTreeMap::new();
        for m in &self.music_list {
            if let Some(album) = m.album.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
                map.entry(album.to_string()).or_default().push(m.clone());
            }
        }
        self.albums = map.into_values().collect();
    }

    pub fn filtered_music<'a>(
        &'a self,
        search: &str,
        album_name: Option<&str>,
        folder_path: Option<&str>,
        playlist_paths: Option<&[String]>,
    ) -> Vec<&'a MusicMetadata> {
        let search_lc = search.to_lowercase();
        let path_set: Option<std::collections::HashSet<&str>> =
            playlist_paths.map(|paths| paths.iter().map(String::as_str).collect());

        self.music_list
            .iter()
            .filter(|m| {
                // Playlist filter
                if let Some(ref set) = path_set {
                    return set.contains(m.path.as_str());
                }
                // Folder filter
                if let Some(fp) = folder_path {
                    let fp_path = std::path::Path::new(fp);
                    let m_path = std::path::Path::new(&m.path);
                    if m_path.parent() != Some(fp_path) {
                        return false;
                    }
                }
                // Album filter
                if let Some(name) = album_name {
                    if m.album.as_deref() != Some(name) {
                        return false;
                    }
                }
                // Search filter
                if !search_lc.is_empty() {
                    let matches =
                        [&m.title, &m.artist, &m.album, &m.album_artist]
                            .iter()
                            .any(|f| {
                                f.as_deref()
                                    .map(|v| v.to_lowercase().contains(&search_lc))
                                    .unwrap_or(false)
                            });
                    return matches;
                }
                true
            })
            .collect()
    }

    pub fn filtered_albums<'a>(&'a self, search: &str) -> Vec<&'a Vec<MusicMetadata>> {
        if search.is_empty() {
            return self.albums.iter().collect();
        }
        let search_lc = search.to_lowercase();
        self.albums
            .iter()
            .filter(|tracks| {
                tracks.first().map_or(false, |m| {
                    m.album
                        .as_deref()
                        .map(|a| a.to_lowercase().contains(&search_lc))
                        .unwrap_or(false)
                        || m.album_artist
                            .as_deref()
                            .map(|a| a.to_lowercase().contains(&search_lc))
                            .unwrap_or(false)
                })
            })
            .collect()
    }
}

/// Thread-safe wrapper stored in Tauri's managed state.
pub struct SharedLibraryState(pub Arc<RwLock<LibraryState>>);

impl Default for SharedLibraryState {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(LibraryState::default())))
    }
}
