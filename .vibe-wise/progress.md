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

## Readability refactor (Rust core + macOS Swift)
Status: R1 (FFI layer) implemented. Next: R2 engine split, then Swift.
Requirement (learner): code findable by humans; e.g. couldn't find where to change a view's padding. Scope: Rust core + Swift only.
Learner decisions:
- Lookup by UI keyword (searches "Now Playing" to find queue view); names should follow UI words.
- Layout constants in one shared file; accepts that one change can affect many places.
- Rust: readability goal; FFI may be modified entirely.
- Order: md doc first, Rust (FFI first), then Swift.
- Finds code by file name too; queue is main focus.
- Rewrite both FFI layers (UniFFI + Windows C ABI); Windows untestable now; Android (UniFFI Kotlin) must keep working; FFI renames must be easy.
- Rust: many small properly named files over few big ones.
- "Now Playing" stays with the queue; full-screen player stays PlayView (alt name: lyrics page).
- FFI naming: `<feature>_<verb>` (queue_get, player_seek).
- Windows keeps hand-written C ABI (P/Invoke + JSON), renamed to the same convention (after explanation that all 3 platforms use FFI).
- UniFFI wrapper stays a separate layer, split into small properly named files (reason: separate files open in tabs and are searchable; avoids scrolling).
- Delete unused FFI functions; rename engine methods too in R1.
- Learner asked to write R1 directly and will verify other platforms later.
Implemented R1: `uniffi_api/{mod,error,events,player,queue,library,album,artwork,lyrics,backdrop,discord}.rs`, `c_api/{mod,memory,events,player,queue,library,album,artwork,lyrics,discord}.rs`; engine methods renamed in lib.rs; Swift/Kotlin callers + Windows FluyerNative.cs/FluyerEngine.cs/FakeEngine.cs updated; README paths fixed.
Verified: cargo test 26 pass (new memory.rs round-trip test); swift build + swift test 66 pass; Android bindings + gradle compile/unit tests OK; nm check: all C# imports exist. Not verified: dotnet build, Windows runtime.
Proposed (not confirmed): Rust `engine/*.rs` impl-block split (R2); Swift `Features/<UI name>/`, one type per file, `Theme/Layout.swift` with role-named constants, per-file `// UI:` keyword comment.
