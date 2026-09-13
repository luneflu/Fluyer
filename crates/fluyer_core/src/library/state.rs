use crate::metadata::MusicMetadata;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Default, Clone, Debug)]
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
        raw.sort_by(|a, b| Self::sort_key(a).cmp(&Self::sort_key(b)));
        self.music_list = raw;

        let mut map: BTreeMap<String, Vec<MusicMetadata>> = BTreeMap::new();
        for m in &self.music_list {
            if let Some(album) = m.album.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
                map.entry(album.to_string()).or_default().push(m.clone());
            }
        }
        self.albums = map.into_values().collect();
    }

    pub fn count(&self) -> usize {
        self.music_list.len()
    }

    pub fn get_by_index(&self, index: usize) -> Option<MusicMetadata> {
        self.music_list.get(index).cloned()
    }

    pub fn get_by_path(&self, path: &str) -> Option<MusicMetadata> {
        self.music_list.iter().find(|m| m.path == path).cloned()
    }

    pub fn filtered_music(
        &self,
        search: &str,
        album_name: Option<&str>,
        folder_path: Option<&str>,
        playlist_paths: Option<&[String]>,
    ) -> Vec<MusicMetadata> {
        let search_lc = search.to_lowercase();
        let path_set: Option<HashSet<&str>> =
            playlist_paths.map(|paths| paths.iter().map(String::as_str).collect());

        self.music_list
            .iter()
            .filter(|m| {
                if let Some(ref set) = path_set {
                    return set.contains(m.path.as_str());
                }
                if let Some(fp) = folder_path {
                    let fp_path = Path::new(fp);
                    let m_path = Path::new(&m.path);
                    if m_path.parent() != Some(fp_path) {
                        return false;
                    }
                }
                if let Some(name) = album_name {
                    if m.album.as_deref() != Some(name) {
                        return false;
                    }
                }
                if !search_lc.is_empty() {
                    let matches = [&m.title, &m.artist, &m.album, &m.album_artist]
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
            .cloned()
            .collect()
    }

    pub fn album_count(&self) -> usize {
        self.albums.len()
    }

    pub fn album_get_by_index(&self, index: usize) -> Option<Vec<MusicMetadata>> {
        self.albums.get(index).cloned()
    }
}
