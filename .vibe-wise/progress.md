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

## Windows UI architecture port
Status: W1-W3 implemented. Learner skipped the Build checkpoint ("execute"); Windows-specific choices are AI-made, not learner decisions.
Explained: two-project split (Fluyer.Core net8.0 testable state vs Fluyer WinUI views); XAML layout constants via ResourceDictionary + StaticResource.
AI choices (unreviewed): same Screens/Components/Shared tree mirrored in both projects; namespaces by top folder; MenuView under Screens/Home/Menu; MediaTransportCoordinator in Shared/Support; CoverState in WinUI project; SettingsView + ToastView split out of MainWindow; Layout.xaml keys `<Feature><Role>` + Layout.cs for code-behind.
Verified: dotnet build OK, dotnet test 112 pass, app launch screenshot shows library. Sidebars/play screen/settings not visually checked. Recorded in ARCHITECTURE.md (Windows target).
Note: stale target/debug/fluyer_core.dll caused 2 smoke-test failures; fixed by cargo build -p fluyer_core.

## Readability refactor (Rust core + macOS Swift)
Status: R1 (FFI) + R2 (engine split) implemented. Swift S1-S3 implemented; awaiting learner visual check.
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
Implemented R2 (learner said "execute"): `engine/{mod,player,queue,library,album,artwork,lyrics,backdrop,thumbnail_cache}.rs`, lib.rs now 16 lines; same feature names as FFI. Verified: cargo test 26, swift test 66, Android compile, nm. Logic unchanged (moved only).
Swift decisions (learner):
- Separate logic and view (MVVM-like).
- Views never call the engine directly; everything goes through a state object.
- State per feature, grouped under screens (e.g. Play > PlayerBar, Play > Lyrics, Home > PlayerBar, Home > MusicGrid).
- Naming: keep `XState` + `XView` (no "ViewModel" suffix in Swift).
- Folders: Screen > Feature > state + view.
- Shared state in a shared folder; shared views in `Components/`; one shared layout file.
- PlayerBar state shared between Home and Play; learner noted the two bars differ (kept as two views).
Design confirmed (Confirm and continue) incl. accepted additions: CoverState, views get only needed state, @State for hover/idle, SettingsState in Shared, role-named Layout grouped by feature, tests renamed. Recorded in ARCHITECTURE.md.
Implemented S1 (Implement this step): files moved to App/, Screens/{Home,Play,Settings}/<feature>/, Shared/{State,Support}/, Components/; renames ContentView->HomeView, QueuePaneView->QueueView, CollectionHeaderView->AlbumHeaderView, EdgeTrigger->QueueEdgeTrigger, AlbumSelection->LibraryFilterState, Metrics->AlbumMetrics; split AlbumGridView, AlbumCard, TrackCard, QueueRow, SortMenu, ToastView, PlayControlsView, LyricsView, Metal backdrop types; tests renamed; README layout updated. Verified swift build + swift test 66 pass. Visual check not done.
Implemented S2 (learner said "continue"): CoverState (Shared/State) wraps engine + ThumbnailStore; views/rows/backdrop/NowPlayingCoordinator use it; every view takes only the state objects it reads (QueueView(queue:playback:library:covers:width:), etc.); AppState only in HomeView/SettingsView/FluyerApp. Test CoverStateTests (no engine returns nil).
Implemented S3: Shared/Theme/Layout.swift, enums per feature (Window, TrackRow, MusicGrid, Albums, AlbumHeader, Queue, PlayerBar, Play, PlayControls, Lyrics, Toast, Settings); all padding/spacing/frame/cornerRadius numbers in Screens/ and Components/ replaced; fonts, shadows, opacities, timings left inline. AlbumMetrics now reads Layout.Albums; queue sidebar inset uses Layout.Queue.outerInset.
Verified: swift build, swift test 67 pass. Visual check NOT done (raw binary launch showed no window). Windows not built.

## Windows menu bar (replaces left menu sidebar)
Status: Implemented (W4).
Explained: macOS menu bar vs Windows per-window commands; WinUI MenuBar styling via theme resource overrides (MenuBar*, MenuFlyoutPresenter*); options table (keep sidebar / MenuBar / drop).
Learner decisions:
- Use WinUI MenuBar; asked whether it supports custom styling.
- "Make it the exact same as macOS" (no further placement/content decisions).
AI choices: MenuBar in TitleBar.LeftHeader; ☰ + MenuView + Ctrl+M removed; Settings gear moved into File menu; queue shortcut Ctrl+Q -> Ctrl+L (macOS Cmd+L); no Ctrl+, (XAML key parse crash); no Songs/Albums (no Windows mode yet). No custom MenuBar styling applied yet.
Verified: build OK, app launches, screenshot shows File/Playback/View. Tests 112 pass; ThumbnailFetchCoordinatorTests.InvalidatePrefix_AbortsQueuedFetch_NextFetchReloads flaky (1 of 3 runs failed), file untouched by this work.

## Windows library filter (sort / Songs-Albums mode)
Status: Implemented (W5). Learner request: "implement the filter like sorting etc like at the macos" (direct, no design checkpoint).
AI choices: full port of macOS LibraryFilterState; AlbumGridView + SortMenu (Button + MenuFlyout with radio items, rebuilt on open); Songs/Albums as two ToggleButtons (no segmented control in WinUI 2.3.9); new C export fluyer_library_play_tracks with nuint array (x86-safe); deleted album_play_track (no callers left); hand-written natural compare.
Learner decision: toolbar right-aligned next to the window controls (moved to TitleBar.RightHeader).
Verified: cargo test OK, dotnet test 116 pass (6 new sort/search/play-order tests), app screenshots: Songs mode, Albums mode (4 cols). Sort flyout opened via UI Automation but popup not capturable; sort choices not visually checked.
Follow-up (learner): toggle had accent-blue checked state, search box had native accent focus. Fix: new Shared/Theme/Colors.xaml merged in App.xaml overrides ToggleButton*Checked* (white wash) and TextControl*Focused (thin white border, no accent underline). Verified toggle by screenshot; search focus style not visually confirmed (automation focus left window inactive).
Follow-up (learner): remove search focus border entirely; album toggle icon clipped. Fix: TextControl*Focused now point to the resting brushes/thickness (focused = resting look). Toggle clipping cause explained: default ButtonPadding 11px/side left 14px in a 36px button for a 14px glyph; new ToolbarModeButtonPadding 0,5,0,6 in Layout.xaml. Verified toggle icon by zoomed screenshot; search focus not visually confirmed.
