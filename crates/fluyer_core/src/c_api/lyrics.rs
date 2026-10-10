use crate::FluyerEngine;

/// Index of the lyric line at `position_ms`, or -1.
#[no_mangle]
pub unsafe extern "C" fn fluyer_lyrics_get_active_index(engine: *mut FluyerEngine, position_ms: u64) -> i32 {
    engine.as_ref().map_or(-1, |e| e.lyrics_get_active_index(position_ms))
}
