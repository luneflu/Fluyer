# Learning Progress

## macOS animated background: palette instead of artwork
Status: Implemented (macOS). Windows port not started.
Requirement (learner): Feed macOS Metal backdrop with extracted colors (as in Fluyer `AnimatedBackground.svelte`) instead of raw artwork, keep current rotation/blur/pinch animation.

Learner decisions:
- Generate a block-color image ("fake artwork") from the palette and pass it to the renderer instead of the real cover.
- Generate in Rust so every platform behaves the same.
- Fixed size: 500x500 image, 75px blocks; cropped edge blocks are fine because the result gets blurred.
- No pre-blur in Rust; each platform's renderer does the blur.
- Re-roll the block layout only on track change; window resize needs no re-roll.
- Fallback: hardcoded random grey blocks.

Accepted additions (via Implement confirmation):
- Animation off shows the same block image, statically blurred.
- A track with no cover uses grey blocks (`cover_palette` returns `GREY_PALETTE`).
- Old `generate_background_for_current` kept for the Windows C ABI.

Implemented: `generate_block_artwork` + `GREY_PALETTE` (background.rs), `generate_block_artwork_for_current` (lib.rs), `load_block_artwork` UniFFI export, AnimatedBackgroundView.swift uses it.
Verified: background tests pass (6), Swift bindings regenerated, `swift build` OK. Visual check not done yet.
Note: `cargo test` needs `DYLD_LIBRARY_PATH=libs/macos` (libbass rpath missing).
Next candidate: Windows port (C ABI export + texture swap in ArtworkBackdropRenderer.cs).
