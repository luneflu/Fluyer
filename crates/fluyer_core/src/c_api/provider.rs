//! Windows surface. Same features as `uniffi_api/provider.rs`, JSON over the C ABI and
//! blocking (P/Invoke cannot await).

use super::memory::json_into_c;
use crate::FluyerEngine;
use serde_json::json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe fn read_str(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok().map(str::to_string)
}

fn error_json(reason: String) -> *mut c_char {
    json_into_c(Some(json!({ "error": reason })))
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_search(
    engine: *mut FluyerEngine,
    query: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    let Some(query) = read_str(query) else { return std::ptr::null_mut() };
    match engine.provider_search_blocking(query) {
        Ok(items) => json_into_c(Some(items)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_album(
    engine: *mut FluyerEngine,
    browse_id: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    let Some(browse_id) = read_str(browse_id) else { return std::ptr::null_mut() };
    match engine.provider_album_blocking(browse_id) {
        Ok(album) => json_into_c(Some(album)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_lyrics(
    engine: *mut FluyerEngine,
    video_id: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    let Some(video_id) = read_str(video_id) else { return std::ptr::null_mut() };
    match engine.provider_lyrics_blocking(video_id) {
        Ok(text) => json_into_c(Some(text)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_stream_formats(
    engine: *mut FluyerEngine,
    video_id: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    let Some(video_id) = read_str(video_id) else { return std::ptr::null_mut() };
    match engine.provider_stream_blocking(video_id) {
        Ok(info) => json_into_c(Some(info)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_begin_login(engine: *mut FluyerEngine) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    match engine.provider_begin_login_blocking() {
        Ok(login) => json_into_c(Some(login)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_poll_login(
    engine: *mut FluyerEngine,
    device_code: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    let Some(device_code) = read_str(device_code) else { return std::ptr::null_mut() };
    match engine.provider_poll_login_blocking(device_code) {
        Ok(status) => json_into_c(Some(status)),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_logout(engine: *mut FluyerEngine) {
    if let Some(engine) = engine.as_ref() {
        engine.provider_logout();
    }
}

/// Returns the pending profile as JSON (null when there is nothing newer) or `{"error":…}`.
#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_check_profile(engine: *mut FluyerEngine) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    match engine.provider_check_profile_blocking() {
        Ok(profile) => json_into_c(profile),
        Err(reason) => error_json(reason),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_apply_profile(engine: *mut FluyerEngine) -> bool {
    engine.as_ref().map(|e| e.provider_apply_profile()).unwrap_or(false)
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_dismiss_profile(engine: *mut FluyerEngine) -> bool {
    engine.as_ref().map(|e| e.provider_dismiss_profile()).unwrap_or(false)
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_profile_version(engine: *mut FluyerEngine) -> u32 {
    engine.as_ref().map(|e| e.provider_profile_version()).unwrap_or(0)
}

/// Login state as JSON `{"loggedIn":bool}` so the C ABI does not need a bool-out pointer.
#[no_mangle]
pub unsafe extern "C" fn fluyer_provider_login_state(engine: *mut FluyerEngine) -> *mut c_char {
    let Some(engine) = engine.as_ref() else { return std::ptr::null_mut() };
    json_into_c(Some(json!({ "loggedIn": engine.provider_is_logged_in() })))
}

#[allow(dead_code)]
fn c_json(text: String) -> *mut c_char {
    CString::new(text).map_or(std::ptr::null_mut(), CString::into_raw)
}
