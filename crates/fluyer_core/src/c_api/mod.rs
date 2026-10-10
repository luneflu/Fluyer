//! C ABI surface for Windows (P/Invoke in `ui/windows/Fluyer.Core/Native/`).
//!
//! Same feature files and names as `uniffi_api/`, prefixed `fluyer_` because C
//! has no namespaces (`player_seek` -> `fluyer_player_seek` in `player.rs`).
//! Rich data crosses as JSON strings; free every returned string with
//! `fluyer_string_free` and every byte buffer with `fluyer_bytes_free`.
#![allow(clippy::missing_safety_doc)]

mod album;
mod artwork;
mod discord;
mod events;
mod library;
mod lyrics;
mod memory;
mod player;
mod queue;

pub use events::{FluyerCallbacks, FluyerPlayerState, FluyerRepeatMode};

use crate::events::EventSink;
use crate::FluyerEngine;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

/// Returns null when either path is null/invalid UTF-8 or the engine fails to start.
#[no_mangle]
pub unsafe extern "C" fn fluyer_init(
    data_dir: *const c_char,
    cache_dir: *const c_char,
    callbacks: FluyerCallbacks,
) -> *mut FluyerEngine {
    if data_dir.is_null() || cache_dir.is_null() {
        return std::ptr::null_mut();
    }
    let (Ok(data), Ok(cache)) = (CStr::from_ptr(data_dir).to_str(), CStr::from_ptr(cache_dir).to_str()) else {
        return std::ptr::null_mut();
    };
    let sink: Option<Arc<dyn EventSink>> = Some(Arc::new(events::CallbackSink { callbacks }));
    match FluyerEngine::new(Path::new(data), Path::new(cache), sink) {
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
