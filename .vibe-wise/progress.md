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

## Backdrop color tuning
Status: Build checkpoint open (where color tuning should live).
Explained: stacked pipeline: Rust balance, then shader saturation x1.3 and x2, dark scrim, clamp.
Learner action: commented out `balance_color` in `extract_prominent_colors` as a temporary step. Shader stages untouched.
Pending decision: which layer owns the look; keep or drop shader stages for block input; target look.

## Backdrop source setting (artwork vs blocks)
Status: Implemented (macOS).
Requirement (learner): User-selectable option in Settings: direct artwork or custom palette blocks as backdrop input. Motivation: block look "not the best" yet.
Learner decisions:
- Setting lives in the macOS UI layer (SettingsView + persisted like the other toggles), not in Rust.
- A choice (enum/picker), not a Bool.
- No track playing: use the hardcoded grey blocks in both modes.
- Animation-off path is broken; out of scope for now.
- Default: direct artwork. Scope: macOS only.
Accepted additions (via Implement confirmation): `BackdropSource` enum in SettingsState.swift; source in the task key so switching reloads; coverless track in artwork mode falls back to grey blocks; unknown saved value decodes to `.artwork`; default `.artwork`.
Implemented: SettingsState.swift (enum + persistence), SettingsView.swift (Picker under Appearance), AnimatedBackgroundView.swift (branch on source).
Verified: `swift build` OK; SettingsStateTests 7/7 pass (round-trip, default, unknown value). Visual check not done yet.
