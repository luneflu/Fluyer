use tauri::State;

use crate::state::AppState;

use crate::music::player::RepeatMode;

#[tauri::command]
pub fn music_play(state: State<AppState>) {
    state.music_player.play();
}

#[tauri::command]
pub fn music_pause(state: State<AppState>) {
    state.music_player.pause();
}

#[tauri::command]
pub fn music_next(state: State<AppState>) {
    state.music_player.next();
}

#[tauri::command]
pub fn music_previous(state: State<AppState>) {
    state.music_player.previous();
}

#[tauri::command]
pub fn music_clear(state: State<AppState>) {
    state.music_player.clear();
}

#[tauri::command]
pub fn music_repeat_mode_set(state: State<AppState>, mode: RepeatMode) {
    state.music_player.set_repeat_mode(mode);
}

#[tauri::command]
pub fn music_position_set(state: State<AppState>, position: u64) {
    state.music_player.set_pos(position);
}

#[tauri::command]
pub fn music_volume_set(state: State<AppState>, volume: f32) {
    state.music_player.set_volume(volume);
}

#[tauri::command]
pub fn music_duration_get(state: State<AppState>) -> Option<f64> {
    Some(state.music_player.get_current_duration())
}

#[tauri::command]
pub fn music_player_request_sync(state: State<AppState>) {
    state.music_player.request_sync();
}

#[tauri::command]
pub fn music_bit_perfect_toggle(state: State<AppState>, enable: bool) {
    state.music_player.toggle_bit_perfect(enable);
}

#[cfg(desktop)]
#[tauri::command]
pub fn music_equalizer(state: State<AppState>, values: Vec<f32>) {
    state.music_player.equalizer(values);
}

#[cfg(desktop)]
#[tauri::command]
pub fn music_equalizer_reset(state: State<AppState>) {
    state.music_player.reset_equalizer();
}

#[cfg(desktop)]
#[tauri::command]
pub fn music_discord_rpc_set(enabled: bool) {
    crate::music::discord_rpc::DiscordRpc::set_enabled(enabled);
}
