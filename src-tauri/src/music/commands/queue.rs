use tauri::State;

use crate::{music::metadata::MusicMetadata, state::AppState};
use std::collections::{HashMap, HashSet};

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
