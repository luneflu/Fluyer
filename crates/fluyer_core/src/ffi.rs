use crate::audio::{MusicPlayerSync, RepeatMode};
use crate::events::EventSink;
use crate::metadata::MusicMetadata;
use crate::FluyerEngine;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::sync::Arc;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluyerRepeatMode {
    None = 0,
    All = 1,
    One = 2,
}

impl From<RepeatMode> for FluyerRepeatMode {
    fn from(m: RepeatMode) -> Self {
        match m {
            RepeatMode::None => FluyerRepeatMode::None,
            RepeatMode::All => FluyerRepeatMode::All,
            RepeatMode::One => FluyerRepeatMode::One,
        }
    }
}

impl From<FluyerRepeatMode> for RepeatMode {
    fn from(m: FluyerRepeatMode) -> Self {
        match m {
            FluyerRepeatMode::None => RepeatMode::None,
            FluyerRepeatMode::All => RepeatMode::All,
            FluyerRepeatMode::One => RepeatMode::One,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FluyerPlayerState {
    pub index: i64,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub is_playing: bool,
    pub repeat_mode: FluyerRepeatMode,
    pub is_shuffled: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FluyerCallbacks {
    pub user_data: *mut c_void,
    pub on_state_changed: Option<unsafe extern "C" fn(*mut c_void, FluyerPlayerState)>,
    pub on_track_changed: Option<unsafe extern "C" fn(*mut c_void, *const c_char, usize)>,
    pub on_scan_progress: Option<unsafe extern "C" fn(*mut c_void, usize, usize)>,
    pub on_toast: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
}

unsafe impl Send for FluyerCallbacks {}
unsafe impl Sync for FluyerCallbacks {}

struct FfiEventSink {
    callbacks: FluyerCallbacks,
}

impl EventSink for FfiEventSink {
    fn on_player_sync(&self, state: MusicPlayerSync) {
        if let Some(cb) = self.callbacks.on_state_changed {
            let ffi_state = FluyerPlayerState {
                index: state.index,
                position_ms: state.position_ms(),
                duration_ms: 0,
                is_playing: state.is_playing,
                repeat_mode: state.repeat_mode.into(),
                is_shuffled: state.is_shuffled,
            };
            unsafe { cb(self.callbacks.user_data, ffi_state) };
        }
    }

    fn on_track_changed(&self, track: Option<MusicMetadata>, index: usize) {
        if let Some(cb) = self.callbacks.on_track_changed {
            let json = serde_json::to_string(&track).unwrap_or_default();
            let c_json = CString::new(json).unwrap_or_default();
            unsafe { cb(self.callbacks.user_data, c_json.as_ptr(), index) };
        }
    }

    fn on_scan_progress(&self, current: usize, total: usize) {
        if let Some(cb) = self.callbacks.on_scan_progress {
            unsafe { cb(self.callbacks.user_data, current, total) };
        }
    }

    fn on_toast(&self, message: &str) {
        if let Some(cb) = self.callbacks.on_toast {
            let c_msg = CString::new(message).unwrap_or_default();
            unsafe { cb(self.callbacks.user_data, c_msg.as_ptr()) };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_init(
    data_dir: *const c_char,
    cache_dir: *const c_char,
    callbacks: FluyerCallbacks,
) -> *mut FluyerEngine {
    if data_dir.is_null() || cache_dir.is_null() {
        return std::ptr::null_mut();
    }

    let data_str = match CStr::from_ptr(data_dir).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let cache_str = match CStr::from_ptr(cache_dir).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let sink: Option<Arc<dyn EventSink>> = if callbacks.on_state_changed.is_some()
        || callbacks.on_track_changed.is_some()
        || callbacks.on_scan_progress.is_some()
        || callbacks.on_toast.is_some()
    {
        Some(Arc::new(FfiEventSink { callbacks }))
    } else {
        None
    };

    match FluyerEngine::new(Path::new(data_str), Path::new(cache_str), sink) {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_free(engine: *mut FluyerEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_play(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.play();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_pause(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.pause();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_toggle_play(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.toggle_play();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_next(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.next();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_previous(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.previous();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_seek(engine: *mut FluyerEngine, position_ms: u64) {
    if let Some(e) = engine.as_ref() {
        e.seek(position_ms);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_set_volume(engine: *mut FluyerEngine, volume: f32) {
    if let Some(e) = engine.as_ref() {
        e.set_volume(volume);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_set_repeat(
    engine: *mut FluyerEngine,
    repeat_mode: FluyerRepeatMode,
) {
    if let Some(e) = engine.as_ref() {
        e.set_repeat_mode(repeat_mode.into());
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_player_shuffle(engine: *mut FluyerEngine) {
    if let Some(e) = engine.as_ref() {
        e.shuffle();
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_get_count(engine: *mut FluyerEngine) -> usize {
    if let Some(e) = engine.as_ref() {
        e.library.read().unwrap().count()
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_get_track_json(
    engine: *mut FluyerEngine,
    index: usize,
) -> *mut c_char {
    if let Some(e) = engine.as_ref() {
        if let Some(track) = e.library.read().unwrap().get_by_index(index) {
            if let Ok(json) = serde_json::to_string(&track) {
                if let Ok(c_str) = CString::new(json) {
                    return c_str.into_raw();
                }
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_library_play_index(engine: *mut FluyerEngine, index: usize) {
    if let Some(e) = engine.as_ref() {
        e.play_all_from_library(index);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
