use super::FluyerAppEngine;

#[uniffi::export]
impl FluyerAppEngine {
    /// Index of the lyric line at `position_ms`, or -1. Runs on the UI's position tick.
    pub fn lyrics_get_active_index(&self, position_ms: u64) -> i32 {
        self.inner.lyrics_get_active_index(position_ms)
    }
}
