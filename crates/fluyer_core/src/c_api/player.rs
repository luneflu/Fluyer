use super::memory::json_into_c;
use crate::FluyerEngine;
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_toggle_play(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.player_toggle_play();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_next(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.player_next();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_previous(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.player_previous();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_seek(engine: *mut FluyerEngine, position_ms: u64) {
    if let Some(e) = engine.as_ref() {
        e.player_seek(position_ms);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_get_position(engine: *mut FluyerEngine) -> u64 {
    engine.as_ref().map_or(0, |e| e.player_get_position())
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_set_volume(engine: *mut FluyerEngine, volume: f32) {
    if let Some(e) = engine.as_ref() {
        e.player_set_volume(volume);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_cycle_repeat(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.player_cycle_repeat();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_shuffle(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.player_shuffle();
    }
}

/// JSON `PlayerBarViewModel`.
#[no_mangle]
pub unsafe extern "C" fn fluyer_player_get_bar(engine: *mut FluyerEngine) -> *mut c_char {
    json_into_c(engine.as_ref().map(|e| e.player_get_bar()))
}

/// JSON `PlayViewModel`.
#[no_mangle]
pub unsafe extern "C" fn fluyer_player_get_play_view(engine: *mut FluyerEngine) -> *mut c_char {
    json_into_c(engine.as_ref().map(|e| e.player_get_play_view()))
}
