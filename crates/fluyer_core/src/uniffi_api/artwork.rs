use super::FluyerAppEngine;

// ponytail: async so the blocking cover decode runs on the tokio pool, never the
// UI thread. Returns a pre-downscaled JPEG so scroll views never pay a
// full-resolution decode or a multi-hundred-KB FFI transfer per cell.
#[uniffi::export]
impl FluyerAppEngine {
    pub async fn artwork_load_track_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .artwork_track_thumbnail(index as usize, max_size)
            .map(|b| b.as_ref().clone())
    }

    pub async fn artwork_load_album_thumbnail(&self, index: u64, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .artwork_album_thumbnail(index as usize, max_size)
            .map(|b| b.as_ref().clone())
    }

    pub async fn artwork_load_current_thumbnail(&self, max_size: u32) -> Option<Vec<u8>> {
        self.inner
            .artwork_current_thumbnail(max_size)
            .map(|b| b.as_ref().clone())
    }
}
