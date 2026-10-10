use super::memory::bytes_into_c;
use crate::FluyerEngine;

// ponytail: pre-downscaled JPEG thumbnails so list views never pay a
// full-resolution decode or transfer per cell. Free with `fluyer_bytes_free`.

#[no_mangle]
pub unsafe extern "C" fn fluyer_artwork_get_track_thumbnail(
    engine: *mut FluyerEngine,
    index: usize,
    max_size: u32,
    out_len: *mut usize,
) -> *mut u8 {
    let bytes = engine.as_ref().and_then(|e| e.artwork_track_thumbnail(index, max_size));
    bytes_into_c(bytes.map(|b| b.as_ref().clone()), out_len)
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_artwork_get_album_thumbnail(
    engine: *mut FluyerEngine,
    index: usize,
    max_size: u32,
    out_len: *mut usize,
) -> *mut u8 {
    let bytes = engine.as_ref().and_then(|e| e.artwork_album_thumbnail(index, max_size));
    bytes_into_c(bytes.map(|b| b.as_ref().clone()), out_len)
}

#[no_mangle]
pub unsafe extern "C" fn fluyer_artwork_get_current_thumbnail(
    engine: *mut FluyerEngine,
    max_size: u32,
    out_len: *mut usize,
) -> *mut u8 {
    let bytes = engine.as_ref().and_then(|e| e.artwork_current_thumbnail(max_size));
    bytes_into_c(bytes.map(|b| b.as_ref().clone()), out_len)
}
