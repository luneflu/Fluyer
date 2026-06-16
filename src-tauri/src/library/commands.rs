use serde::{Deserialize, Serialize};
use tauri::State;

use crate::library::SharedLibraryState;
use crate::music::metadata::MusicMetadata;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCounts {
    pub music_count: usize,
    pub album_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    pub track_count: usize,
    pub total_duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CollectionContext {
    Album {
        name: String,
    },
    AlbumIndex {
        index: usize,
        search: String,
        #[serde(rename = "sortAsc")]
        sort_asc: bool,
    },
    Folder {
        path: String,
    },
    Playlist {
        paths: Vec<String>,
    },
}

#[tauri::command]
pub async fn library_load(lib: State<'_, SharedLibraryState>) -> Result<LibraryCounts, String> {
    let raw = crate::folder::database::get_tracks();
    let mut guard = lib.0.write().map_err(|e| e.to_string())?;
    guard.rebuild(raw);
    Ok(LibraryCounts {
        music_count: guard.music_list.len(),
        album_count: guard.albums.len(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFilter {
    pub search: String,
    #[serde(default)]
    pub sort_asc: bool,
    pub album_name: Option<String>,
    pub folder_path: Option<String>,
    pub playlist_paths: Option<Vec<String>>,
}

#[tauri::command]
pub fn library_music_count_get(lib: State<'_, SharedLibraryState>, filter: MusicFilter) -> usize {
    let guard = lib.0.read().unwrap();
    guard
        .filtered_music(
            &filter.search,
            filter.album_name.as_deref(),
            filter.folder_path.as_deref(),
            filter.playlist_paths.as_deref(),
        )
        .len()
}

#[tauri::command]
pub fn library_music_get_by_index(
    lib: State<'_, SharedLibraryState>,
    index: usize,
    filter: MusicFilter,
) -> Option<MusicMetadata> {
    let guard = lib.0.read().unwrap();
    let mut items = guard.filtered_music(
        &filter.search,
        filter.album_name.as_deref(),
        filter.folder_path.as_deref(),
        filter.playlist_paths.as_deref(),
    );
    if !filter.sort_asc {
        items.reverse();
    }
    items.get(index).map(|m| (*m).clone())
}

#[tauri::command]
pub fn library_music_get_by_path(
    lib: State<'_, SharedLibraryState>,
    path: String,
) -> Option<MusicMetadata> {
    let guard = lib.0.read().unwrap();
    guard.music_list.iter().find(|m| m.path == path).cloned()
}

#[tauri::command]
pub fn library_album_count_get(
    lib: State<'_, SharedLibraryState>,
    search: String,
    sort_asc: bool,
) -> usize {
    let guard = lib.0.read().unwrap();
    guard.filtered_albums(&search).len()
}

#[tauri::command]
pub fn library_album_get_by_index(
    lib: State<'_, SharedLibraryState>,
    index: usize,
    search: String,
    sort_asc: bool,
) -> Option<Vec<MusicMetadata>> {
    let guard = lib.0.read().unwrap();
    let mut albums = guard.filtered_albums(&search);
    if !sort_asc {
        albums.reverse();
    }
    albums.get(index).map(|tracks| (*tracks).clone())
}

#[tauri::command]
pub fn library_album_get_first_by_index(
    lib: State<'_, SharedLibraryState>,
    index: usize,
    search: String,
    sort_asc: bool,
) -> Option<MusicMetadata> {
    let guard = lib.0.read().unwrap();
    let mut albums = guard.filtered_albums(&search);
    if !sort_asc {
        albums.reverse();
    }
    albums.get(index).and_then(|tracks| tracks.first().cloned())
}

#[tauri::command]
pub fn music_queue_count_get(state: State<'_, AppState>) -> usize {
    state.music_player.queue_count()
}

#[tauri::command]
pub fn music_queue_get_by_index(state: State<'_, AppState>, index: usize) -> Option<MusicMetadata> {
    state.music_player.queue_get_by_index(index)
}

#[tauri::command]
pub fn library_folder_info_get(lib: State<'_, SharedLibraryState>, path: String) -> FolderInfo {
    let guard = lib.0.read().unwrap();
    let mut track_count = 0;
    let mut total_duration = 0.0;

    for m in &guard.music_list {
        if std::path::Path::new(&m.path).starts_with(&path) {
            track_count += 1;
            if let Some(d) = m.duration {
                total_duration += d as f64;
            }
        }
    }

    FolderInfo {
        track_count,
        total_duration,
    }
}

#[tauri::command]
pub fn library_folders_filter_has_music(
    lib: State<'_, SharedLibraryState>,
    paths: Vec<String>,
) -> Vec<String> {
    let guard = lib.0.read().unwrap();
    paths
        .into_iter()
        .filter(|p| {
            let p_path = std::path::Path::new(p);
            guard
                .music_list
                .iter()
                .any(|m| std::path::Path::new(&m.path).starts_with(p_path))
        })
        .collect()
}

fn resolve_tracks(
    lib: &crate::library::LibraryState,
    context: &CollectionContext,
) -> Vec<MusicMetadata> {
    match context {
        CollectionContext::Album { name } => lib
            .music_list
            .iter()
            .filter(|m| m.album.as_deref() == Some(name.as_str()))
            .cloned()
            .collect(),
        CollectionContext::AlbumIndex {
            index,
            search,
            sort_asc,
        } => {
            let mut albums = lib.filtered_albums(search);
            if !sort_asc {
                albums.reverse();
            }
            albums.get(*index).map(|v| (*v).clone()).unwrap_or_default()
        }
        CollectionContext::Folder { path } => {
            let p_path = std::path::Path::new(path);
            lib.music_list
                .iter()
                .filter(|m| std::path::Path::new(&m.path).starts_with(p_path))
                .cloned()
                .collect()
        }
        CollectionContext::Playlist { paths } => {
            let set: std::collections::HashSet<&str> = paths.iter().map(String::as_str).collect();
            lib.music_list
                .iter()
                .filter(|m| set.contains(m.path.as_str()))
                .cloned()
                .collect()
        }
    }
}

#[tauri::command]
pub fn library_collection_add_and_play(
    lib: State<'_, SharedLibraryState>,
    state: State<'_, AppState>,
    context: CollectionContext,
) {
    let tracks = {
        let guard = lib.0.read().unwrap();
        resolve_tracks(&guard, &context)
    };
    state.music_player.clear();
    state.music_player.add_track(tracks);
    state.music_player.play();
}

#[tauri::command]
pub fn library_collection_add_to_queue(
    lib: State<'_, SharedLibraryState>,
    state: State<'_, AppState>,
    context: CollectionContext,
) {
    let tracks = {
        let guard = lib.0.read().unwrap();
        resolve_tracks(&guard, &context)
    };
    state.music_player.add_track(tracks);
}

#[tauri::command]
pub fn library_collection_shuffle_and_play(
    lib: State<'_, SharedLibraryState>,
    state: State<'_, AppState>,
    context: CollectionContext,
) {
    let mut tracks = {
        let guard = lib.0.read().unwrap();
        resolve_tracks(&guard, &context)
    };

    // Fisher-Yates shuffle
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);

    let mut rng = seed as usize;
    for i in (1..tracks.len()).rev() {
        let mut h = DefaultHasher::new();
        (rng ^ i).hash(&mut h);
        rng = h.finish() as usize;
        let j = rng % (i + 1);
        tracks.swap(i, j);
    }

    state.music_player.clear();
    state.music_player.add_track(tracks);
    state.music_player.play();
}

#[tauri::command]
pub async fn library_sync() {
    #[cfg(target_os = "android")]
    if !crate::commands::mobile::audio_permission_read_check() {
        return;
    }

    let mut search_dirs: Vec<String> = vec![];

    // Get store music paths
    let dir = match crate::state::app_store()
        .get(crate::music::commands::directory::MUSIC_STORE_PATH_NAME)
    {
        Some(d) => d.to_string(),
        None => return,
    };
    let dir_paths = dir.split(crate::folder::types::MUSIC_PATH_SEPARATOR);

    for d in dir_paths {
        let trimmed = d.trim().trim_matches('"');
        if !trimmed.is_empty() {
            search_dirs.push(trimmed.to_string());
        }
    }

    if crate::platform::is_ios() {
        search_dirs.push(crate::folder::scanner::get_home_dir())
    }

    let now = std::time::Instant::now();
    // Scan directories for music files
    let paths = crate::folder::scanner::scan_directories(search_dirs);
    crate::info!("Scan directories took {}s", now.elapsed().as_secs_f64());

    let now = std::time::Instant::now();
    // Process files and update database
    crate::folder::scanner::process_supported_files(&paths).await;
    crate::info!("Process files took {}s", now.elapsed().as_secs_f64());

    let now = std::time::Instant::now();
    crate::folder::database::delete_non_existing_paths(paths);
    crate::info!(
        "Delete non existing paths took {}s",
        now.elapsed().as_secs_f64()
    );
}
