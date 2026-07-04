use tauri::State;

use crate::{music::metadata::MusicMetadata, state::AppState};
use std::collections::{HashMap, HashSet};

#[tauri::command]
pub async fn music_queue_add(
    state: State<'_, AppState>,
    library: State<'_, crate::library::SharedLibraryState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let path_set: HashSet<&str> = paths.iter().map(|s| s.as_str()).collect();
    let mut lookup = HashMap::new();

    // Quick lookup from memory
    {
        let lib = library.0.read().unwrap();
        for m in &lib.music_list {
            if path_set.contains(m.path.as_str()) {
                lookup.insert(m.path.clone(), m.clone());
            }
        }
    }

    let mut playlist = Vec::with_capacity(paths.len());
    for path in paths {
        if let Some(m) = lookup.get(&path) {
            playlist.push(m.clone());
        } else {
            // Fallback for files not in library
            if let Ok(m) = MusicMetadata::get(path.clone()).await {
                playlist.push(m);
            }
        }
    }

    state.music_player.add_track(playlist);
    Ok(())
}

#[tauri::command]
pub fn music_queue_remove(state: State<AppState>, index: usize) {
    state.music_player.remove_track(index);
}

#[tauri::command]
pub fn music_queue_goto(state: State<AppState>, index: usize) {
    state.music_player.goto_track(index);
}

#[tauri::command]
pub fn music_queue_moveto(state: State<AppState>, from: usize, to: usize) {
    state.music_player.moveto_track(from, to);
}

#[tauri::command]
pub fn music_queue_shuffle(state: State<AppState>) {
    state.music_player.shuffle_track();
}
