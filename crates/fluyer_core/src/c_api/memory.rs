//! Ownership helpers: everything Rust hands to C must come back through the
//! matching `fluyer_*_free`, or it leaks.

use serde::Serialize;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Serializes `value` to a Rust-owned C string; null when absent or unserializable.
pub(super) fn json_into_c<T: Serialize>(value: Option<T>) -> *mut c_char {
    value
        .and_then(|v| serde_json::to_string(&v).ok())
        .and_then(|json| CString::new(json).ok())
        .map_or(std::ptr::null_mut(), CString::into_raw)
}

/// Hands a byte buffer to C; `out_len` receives its length (0 + null when absent).
pub(super) unsafe fn bytes_into_c(bytes: Option<Vec<u8>>, out_len: *mut usize) -> *mut u8 {
    let bytes = bytes.unwrap_or_default();
    if !out_len.is_null() {
        *out_len = bytes.len();
    }
    if bytes.is_empty() {
        return std::ptr::null_mut();
    }
    Box::into_raw(bytes.into_boxed_slice()) as *mut u8
}

/// Reads `count` UTF-8 C strings; null or invalid entries are skipped.
pub(super) unsafe fn strings_from_c(paths: *const *const c_char, count: usize) -> Vec<String> {
    if paths.is_null() {
        return Vec::new();
    }
    (0..count)
        .map(|i| *paths.add(i))
        .filter(|p| !p.is_null())
        .filter_map(|p| CStr::from_ptr(p).to_str().ok().map(str::to_string))
        .collect()
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_bytes_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_and_bytes_round_trip() {
        unsafe {
            let s = json_into_c(Some(vec![1, 2]));
            assert_eq!(CStr::from_ptr(s).to_str().unwrap(), "[1,2]");
            fluyer_string_free(s);
            assert!(json_into_c::<u8>(None).is_null());

            let mut len = 99;
            let p = bytes_into_c(Some(vec![7, 8, 9]), &mut len);
            assert_eq!((len, *p.add(2)), (3, 9));
            fluyer_bytes_free(p, len);
            assert!(bytes_into_c(None, &mut len).is_null());
            assert_eq!(len, 0);

            let a = CString::new("/a").unwrap();
            let arr = [a.as_ptr(), std::ptr::null()];
            assert_eq!(strings_from_c(arr.as_ptr(), 2), vec!["/a".to_string()]);
        }
    }
}
