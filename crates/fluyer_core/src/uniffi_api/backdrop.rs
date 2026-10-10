use super::FluyerAppEngine;
use crate::view_models::AnimatedBackgroundFrame;

/// Animated background behind the whole window.
#[uniffi::export]
impl FluyerAppEngine {
    /// Unblurred palette-block square that replaces the cover as backdrop input.
    /// Random per call: the UI calls it once per track.
    pub async fn backdrop_load_block_artwork(&self) -> AnimatedBackgroundFrame {
        let (rgba, width, height) = self.inner.backdrop_block_artwork();
        AnimatedBackgroundFrame { rgba, width, height }
    }
}
