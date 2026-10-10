use super::memory::json_into_c;
use crate::FluyerEngine;
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn fluyer_album_get_count(engine: *mut FluyerEngine) -> usize {
    engine.as_ref().map_or(0, |e| e.album_get_count())
}

/// JSON `AlbumCardViewModel`, or null for an unknown index.
#[no_mangle]
pub unsafe extern "C" fn fluyer_album_get_card(engine: *mut FluyerEngine, index: usize) -> *mut c_char {
    json_into_c(engine.as_ref().and_then(|e| e.album_get_card(index)))
}

/// JSON `AlbumDetailViewModel`, or null for an unknown index.
#[no_mangle]
pub unsafe extern "C" fn fluyer_album_get_detail(engine: *mut FluyerEngine, index: usize) -> *mut c_char {
    json_into_c(engine.as_ref().and_then(|e| e.album_get_detail(index)))
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_album_play(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.album_play(index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_album_play_track(
    engine: *mut FluyerEngine,
    album_index: usize,
    track_index: usize,
) {
    if let Some(e) = engine.as_ref() {
        e.album_play_track(album_index, track_index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_album_queue(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.album_queue(index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_album_shuffle(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.album_shuffle(index);
    }
}
