use super::memory::json_into_c;
use crate::view_models::SettingsViewModel;
use crate::FluyerEngine;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Settings as camelCase JSON (`SettingsViewModel`). Null only when `engine` is null.
/// Free with `fluyer_string_free`.
#[no_mangle]
pub unsafe extern "C" fn fluyer_settings_get(engine: *mut FluyerEngine) -> *mut c_char {
    json_into_c(engine.as_ref().map(FluyerEngine::settings_get))
}

/// Reads the same JSON shape, normalizes it, persists it and returns what was
/// stored. Null when `engine`/`json` is null or the payload doesn't parse.
#[no_mangle]
pub unsafe extern "C" fn fluyer_settings_set(
    engine: *mut FluyerEngine,
    json: *const c_char,
) -> *mut c_char {
    let Some(engine) = engine.as_ref() else {
        return std::ptr::null_mut();
    };
    if json.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(text) = CStr::from_ptr(json).to_str() else {
        return std::ptr::null_mut();
    };
    let Ok(settings) = serde_json::from_str::<SettingsViewModel>(text) else {
        return std::ptr::null_mut();
    };
    json_into_c(Some(engine.settings_set(&settings)))
}
