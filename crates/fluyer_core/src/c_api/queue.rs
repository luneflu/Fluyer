use super::memory::json_into_c;
use crate::FluyerEngine;
use std::os::raw::c_char;

/// JSON array of `TrackItemViewModel` in play order; `index` = queue position.
#[no_mangle]
pub unsafe extern "C" fn fluyer_queue_get(engine: *mut FluyerEngine) -> *mut c_char {
    json_into_c(engine.as_ref().map(|e| e.queue_get()))
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_queue_goto(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.queue_goto(index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_queue_remove(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.queue_remove(index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_queue_move(engine: *mut FluyerEngine, from: usize, to: usize) {
    if let Some(e) = engine.as_ref() {
        e.queue_move(from, to);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_queue_clear(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.queue_clear();
    }
}
