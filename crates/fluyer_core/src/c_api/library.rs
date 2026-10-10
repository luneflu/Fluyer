use super::memory::{json_into_c, strings_from_c};
use crate::FluyerEngine;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_scan(
    engine: *mut FluyerEngine,
    paths: *const *const c_char,
    count: usize,
) {
    if let Some(e) = engine.as_ref() {
        if count > 0 {
            e.library_scan(&strings_from_c(paths, count));
        }
    }
}

/// Drops every library row under `path` (a removed music folder).
#[no_mangle]
pub unsafe extern "C" fn fluyer_library_remove_folder(engine: *mut FluyerEngine, path: *const c_char) {
    if let (Some(e), false) = (engine.as_ref(), path.is_null()) {
        if let Ok(s) = CStr::from_ptr(path).to_str() {
            e.library_remove_folder(s);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_get_track_count(engine: *mut FluyerEngine) -> usize {
    engine.as_ref().map_or(0, |e| e.library_get_track_count())
}

/// JSON `TrackItemViewModel`, or null for an unknown index.
#[no_mangle]
pub unsafe extern "C" fn fluyer_library_get_track(engine: *mut FluyerEngine, index: usize) -> *mut c_char {
    json_into_c(engine.as_ref().and_then(|e| e.library_get_track(index)))
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_play_all(engine: *mut FluyerEngine, start_index: usize) {
    if let Some(e) = engine.as_ref() {
        e.library_play_all(start_index);
    }
}
