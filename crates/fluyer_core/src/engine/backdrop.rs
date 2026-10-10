//! Animated background input: cover palette and block artwork.

use super::FluyerEngine;
use crate::metadata::MusicMetadata;
use crate::services;
use std::sync::Arc;

impl FluyerEngine {
    /// Sharp block-colour square fed to the GPU backdrops instead of the real cover,
    /// so every palette colour gets equal share. Greys when nothing is playing.
    pub fn backdrop_block_artwork(&self) -> (Vec<u8>, u32, u32) {
        let colors = match self.player.get_current_track() {
            Some(ref t) => self.backdrop_palette(t),
            None => services::background::GREY_PALETTE.to_vec(),
        };
        let img = services::background::generate_block_artwork(
            &colors,
            services::background::BLOCK_ARTWORK_SIZE,
            services::background::BLOCK_ARTWORK_BLOCK,
        );
        let (w, h) = (img.width(), img.height());
        (img.into_raw(), w, h)
    }

    // ponytail: dominant-color extraction decodes the full cover, so it is
    // memoized per track path. PlayView polls this on every state change.
    pub(super) fn backdrop_palette(&self, track: &MusicMetadata) -> Vec<[u8; 3]> {
        if let Some((cached_path, colors)) = self.palette_cache.read().unwrap().as_ref() {
            if cached_path == &track.path {
                return colors.as_ref().clone();
            }
        }

        let colors = match self.artwork_resolve_track(track, None) {
            Some(ref bytes) => services::background::extract_prominent_from_bytes(bytes, 10, false),
            None => services::background::GREY_PALETTE.to_vec(),
        };

        *self.palette_cache.write().unwrap() = Some((track.path.clone(), Arc::new(colors.clone())));
        colors
    }
}
