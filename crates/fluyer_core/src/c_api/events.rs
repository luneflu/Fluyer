use crate::audio::{MusicPlayerSync, RepeatMode};
use crate::events::EventSink;
use crate::metadata::MusicMetadata;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

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
    pub on_track_cover_loaded: Option<unsafe extern "C" fn(*mut c_void, usize)>,
    pub on_album_cover_loaded: Option<unsafe extern "C" fn(*mut c_void, usize)>,
    pub on_lyrics_loaded: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
}

unsafe impl Send for FluyerCallbacks {}
unsafe impl Sync for FluyerCallbacks {}

/// Forwards core events to the C function pointers; unset callbacks are skipped.
pub(super) struct CallbackSink {
    pub(super) callbacks: FluyerCallbacks,
}

impl EventSink for CallbackSink {
    fn on_player_sync(&self, state: MusicPlayerSync) {
        if let Some(cb) = self.callbacks.on_state_changed {
            let duration_ms = state.duration_ms();
            let ffi_state = FluyerPlayerState {
                index: state.index,
                position_ms: state.position_ms(),
                duration_ms,
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

    fn on_track_cover_loaded(&self, index: usize) {
        if let Some(cb) = self.callbacks.on_track_cover_loaded {
            unsafe { cb(self.callbacks.user_data, index) };
        }
    }

    fn on_album_cover_loaded(&self, index: usize) {
        if let Some(cb) = self.callbacks.on_album_cover_loaded {
            unsafe { cb(self.callbacks.user_data, index) };
        }
    }

    fn on_lyrics_loaded(&self, lyrics: &str) {
        if let Some(cb) = self.callbacks.on_lyrics_loaded {
            let c_lyrics = CString::new(lyrics).unwrap_or_default();
            unsafe { cb(self.callbacks.user_data, c_lyrics.as_ptr()) };
        }
    }
}
