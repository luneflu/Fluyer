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

## Windows queue styling (match macOS)
Status: Implemented. Learner request: "update the queue to have the same styling as macos one" (direct, ported from QueueView.swift/QueueRow.swift).
- QueueView: "Now Playing" header 22pt + 28px clear button, glass card (acrylic + #1AD4D4D4 wash, 1px #1FFFFFFF border, radius 4), margins 0,6,12,12 (Layout.Queue values).
- Rows reuse TrackRow look (44px cover radius 4, title 13 + speaker icon on current, "artist • album" 11, mono duration); cover cache key shared with song grid (library index resolved by path, needs new Library + Covers DPs).
- Hover: cover becomes jump button (back/forward glyph vs current row), duration swaps to remove. Up/down buttons moved to right-click menu (Play / Move Up / Move Down / Remove), as on macOS.
- Not ported: drag-to-reorder (macOS List onMove). Hover state not visually verified.

## Windows backdrop source (artwork vs color blocks)
Status: Implemented (Implement this step).
Learner decisions: go through FFI (Windows C ABI); show the Artwork / Color blocks picker in Windows settings. Carried from macOS section: Rust generates blocks, default Artwork, grey blocks with no track/cover.
Accepted additions: C ABI `fluyer_backdrop_get_block_artwork` returns JPEG via `bytes_into_c` (renderer already decodes JPEG; size travels in the file); `IFluyerEngine.GetBlockArtworkAsync`; `CoverState.BackdropBytes(source, hasTrack)`; `BackdropSource` enum + persisted string setting (unknown -> Artwork); ComboBox "Backdrop" in SettingsView; key `source|path|palette`, no-track shows grey blocks.
Verified: cargo test (new rgba_to_jpeg test), dotnet build, dotnet test 118 pass (round-trip, default, unknown value). Screenshot: no-track grey blocks in both modes. Not verified: colored blocks while playing (automation could not start playback), picker UI.

## Windows lyrics centering + edge fade
Status: Implemented (Implement this step).
Learner decisions: spacer above/below lyrics sized to half the visible height (their Svelte py-[50%] idea); fade via composition mask ("the other 2 is just hacky trick" — rejected overlay gradient and per-line opacity).
Explained concepts: CSS % padding is relative to width; WinUI Thickness is fixed px; SizeChanged; ListView Header/Footer keep item indices intact; XAML has no OpacityMask, Composition visual surface + mask brush does.
Accepted additions: spacers in ListView.Header/Footer (not items, so CurrentLyricIndex/ContainerFromIndex stay aligned); Lyrics.SizeChanged sets heights = height/2 and resizes snapshot + sprite; LyricsPadding 28,0,28,0; LyricsFadeHost Border overlays column, list's own visual opacity 0; stops 0/0.4/0.6/1 like Swift.
Verified: dotnet build ok; screenshot shows first line centered and bottom fade. Not verified: top fade mid-song, window resize.

## Linux GTK (gtk-rs) port
Status: G1 implemented.
Requirement (learner): implement the app on gtk-rs; macOS Swift UI is the main reference (most complete); plan first.
Facts found: Linux host; GTK 4.14 + libadwaita 1.5 runtime installed, -dev packages not installed (no sudo for agent); no `libs/linux` (build.rs expects `libs/<target_os>`); BASS .so files exist in legacy `~/Fluyer/src-tauri/libs/`; core already has Linux cfg for plugin names; `fluyer_core` builds as rlib too.
Learner decisions:
- GTK app calls `FluyerEngine` directly (Rust dependency, no FFI).
- Keep the Swift state/view split (XState + XView).
- Events: "same answer as first one" (direct; exact mechanism not chosen).
- Order: build everything in one pass ("yolo"), fix afterwards.
Learner assumption to correct: "GTK doesn't have observable state" (it has GObject properties + notify, ListModel).
Setup done: learner installed gtk4/adwaita dev pkgs; BASS .so copied to libs/linux (gitignored); `.cargo/config.toml` disables rust-lld (libbassmix.so symbol table rejected by rust-lld); core builds, cargo test passes with LD_LIBRARY_PATH=$PWD/libs/linux.
Explained: GObject properties/notify/bind_property, gio::ListModel; non-Send widgets need a channel + glib::spawn_future_local for core events.
- Views observe state via GObject properties (+ notify / bind_property); learner chose it as "the preferable way for GTK".
Implemented G1:
- `build.rs`: rpath embeds `$ORIGIN/../../libs/linux` and `libs/linux`.
- `events.rs`: `EventBridge` implements `EventSink`, forwards to `async_channel::unbounded`.
- `shared/`: GObject models (`TrackItemObject`, `AlbumCardObject`, `LyricLineObject`), states (`PlaybackState`, `LibraryState`, `QueueState`, `CoverState`, `SettingsState`, `AppState`), clock 250ms sync.
- `components/`: `AnimatedBackgroundView` (GSK snapshot `push_blur(50.0)`, texture render, dark scrim overlay).
- `screens/home/`: `HomeView` with `adw::OverlaySplitView`, `ToolbarView` (scan spinner, mode toggle, search, add folder, queue toggle, settings), `MusicGridView` (track list, empty state, click to play), `AlbumGridView` (album cards grid), `QueueView` ("Now Playing" header, clear button, remove button, click to jump), `PlayerBarView` (seek scale, transport buttons, cover thumbnail, title/artist, volume scale, repeat, shuffle).
- `screens/play/`: `PlayView` (two-column layout, large cover, `PlayControlsView`, `LyricsView` with active lyric auto-scroll and seek on click).
- `screens/settings/`: `SettingsView` (`adw::PreferencesDialog` with music folders, folder picker dialog, backdrop source combo row).
- `main.rs`: `adw::Application`, CSS styles, keyboard shortcuts (Space, Esc, Ctrl+L, Ctrl+1, Ctrl+2).
Verified: `cargo test -p fluyer_linux` passes (1 model unit test), `cargo build -p fluyer_linux` OK with 0 warnings. Visual runtime check not done.
Bugfix (learner report: folder added but list empty; after restart no window):
- No window: list-row bind decoded every cover on the main thread (gdb: zune_jpeg under `TrackRow::bind`), unoptimized deps made it seconds per cover. Fix: `CoverState` loads via `gio::spawn_blocking`, sets texture when done; root `Cargo.toml` `[profile.dev.package."*"] opt-level = 3`. Window now shows in ~0.5s.
- Empty after scan: UI waited for `current >= total`, but core counts only changed files. Fix: reload on the "Library scan completed" toast (same trick as uniffi_api/events.rs).
Not fixed (core, all platforms): repeated MusicBrainz requests for the same album when covers are missing.
Bugfix 2 (learner report: no album carousel, song list not a grid, `gtk_list_view_scroll_to` CRITICAL):
- Lyrics CRITICAL: cursor index set before the lyric store was refilled / past its end. Fix: `PlaybackState::set_active_lyric` clamps to the store; `sync_lyrics` resets to -1, fills, then sets.
- Added (macOS port): `albums/album_carousel_view.rs` (horizontal ListView), `album_header_view.rs` (subtitle, Back/Play/Queue/Shuffle), `album_metrics.rs` (macOS AlbumMetrics, 1x rules only; test: cards fill width). Album card click opens album (`LibraryState::select_album`, filter by track paths from `album_get_detail`), right-click Play/Add to Queue/Shuffle, selected/dimmed CSS. Album grid uses same metrics.
- Song grid: `GridView`, columns = width / (280 + 12) like macOS adaptive grid. Click plays the shown list from that track (`library_play_tracks`).
- Width tracking via `add_tick_callback` (ponytail: GTK4 has no resize signal).
Verified: build 0 warnings, 2 tests pass, screenshot shows carousel with names + 3-column song grid, 0 CRITICAL at startup. Not verified: album click/header, Albums mode, lyrics scroll while playing.

## Linux sort (macOS SortMenu port)
Status: Implemented. Learner: "yes, and just make it the same as macos" (no own design; layout copied from macOS LibraryFilterState/SortMenu).
- `LibraryState`: `track_sort` (Album/Title/Artist/Duration), `album_sort` (Name/Artist/Year/Track Count), shared `sort_ascending`; session-only like macOS. `refresh_displayed` re-derives `filtered_tracks` + new `displayed_albums` store on any notify of query/sort.
- Natural order via `glib::FilenameCollationKey` on casefolded text (test caught C-locale case sensitivity). Search: casefold + NFD minus combining marks (diacritic-insensitive), albums match name/artist only.
- Play from grid / header Play = sorted unfiltered list (macOS `playTrack`/`playSelected`). Carousel + Albums grid use `displayed_albums_store`.
- `toolbar/sort_menu.rs`: MenuButton + gio::Menu radio items via stateful actions; keys swap with mode.
Verified: 4 tests pass (incl. sort/filter, natural/folded search), build 0 warnings, screenshot shows sort button. Not verified: clicking menu items (no xdotool).

## Cover fetch dedup (core, all platforms)
Status: Implemented (Implement this step).
Learner reasoning: skip a request if another is already running; check the cache first; layer unsure, noted the core is shared. Explained: disk-cache check already existed in `fetch_and_cache`; spam came from parallel row requests on a cold cache all missing before the first download finished; core is the layer every platform shares.
Learner decisions: in-flight dedup in `CoverArtService`, keyed by cache path.
Accepted additions: duplicate caller gets `Ok(None)` (UI reloads on cover-loaded event); failures not remembered (retry next launch); one unit test.
Implemented: `in_flight: Mutex<HashSet<PathBuf>>`, `begin_fetch` + `InFlight` drop guard (clears on error/cancel).
Verified: core tests 26 pass incl. `second_fetch_of_same_cover_is_skipped_until_first_ends`; live cold-cache run: 1 MusicBrainz request for 'Dabin Between Broken' (was 13+), cover re-cached. Linux 4 tests pass. macOS/Windows not rebuilt.

## Linux album grid grows forever (Bugfix)
Status: Implemented (Implement this step).
Learner report: Albums-mode grid kept expanding; Gtk-CRITICAL `gtk_widget_measure: for_size >= -1`, negative allocations.
Explained: layout feedback loop. `ScrolledWindow` hscroll `Never` forces window >= content min width; card size computed from window width; last column's CSS margin-right 12px made content width+12, so it grew every frame until overflow.
Learner decisions: confirmed proposed fix (External policy + split margins).
Implemented: `album_grid_view.rs` hscroll policy `External`; grid side margins EDGE - CARD_GAP/2 (10px); CSS `.album-grid > child { margin: 0 6px 12px 6px; }` so n*(cover+12)+20 == width.
Verified: Albums-mode launch (temp env hook, reverted) shows 5 columns at stable width, 0 CRITICAL/WARNING in log. Window resize not tested manually.

## Linux animated backdrop (macOS port)
Status: Implemented. Learner: "just implement it like macos; the current one can be the off option for now" (direct, no design checkpoint).
Explained: macOS pipeline = rotation (3 rotating copies, sat x1.3, 0.5s crossfade) → Gaussian blur → pinch mesh warp (sat x2, dark scrim, clamp); GTK options (snapshot nodes, deprecated GLShader, GLArea).
AI choices (unreviewed): `gtk::GLArea` + new dep `glow` (GL loaded through GTK's libepoxy via dlsym, no extra system lib); blur as 2-pass separable GLSL at 320px long edge (macOS 960 + MPS); Reduce Motion = GTK enable-animations off (1/10 speed); GL failure falls back to static for the session.
Learner decisions: off = existing static GSK blur view.
Implemented: components/backdrop/{backdrop_mesh.rs (Catmull-Clark mesh port + test), backdrop_renderer.rs (GL pipeline), animated_backdrop_view.rs (GLArea, tick-driven), static_backdrop_view.rs (renamed old view), mod.rs backdrop_view() Stack switch}; SettingsState `animated_background` (default true, persisted); Settings "Animated background" SwitchRow.
Verified: build 0 warnings, 5 tests pass; screenshots 4s apart while playing differ, colorful moving blur from cover; ~17% CPU steady (dev build). Not verified: toggle switch in Settings, crossfade on track change, Wayland/GLES path.
Banding fix (learner: "already fixed on windows"): ported Windows fixes from BackdropShaders.cs/ArtworkBackdropRenderer.cs: per-pixel temporal ±1 LSB dither (`DitherNoise` hash) in pinch shader, RGBA16F intermediate targets (Windows R16G16B16A16_Float). Gotcha: GTK gave a GLES context; without `precision highp int` Mesa ran the hash at mediump and the noise came out 0. Verified by pixel row: before flat 137,137,...; after 79,79,78,80,79,... Build 0 warnings, 5 tests pass.

## Android backdrop (Linux renderer port)
Status: Implemented (Implement this step). Visual banding check pending (learner).
Learner decision: replicate Linux renderer on Android (same OpenGL; fixes banding elsewhere).
Explained: banding = 8-bit intermediates + no dither; fixes = RGBA16F targets + temporal ±1 LSB dither; GLES needs `precision highp int` for the hash.
Accepted additions: RGBA8 fallback when RGBA16F framebuffer incomplete (GLES 3.0 needs EXT_color_buffer_half_float); keep 30 fps cap; keep music-note placeholder for null artwork (later replaced, see below).
Implemented (BackdropRenderer.kt): highp, ditherNoise in PINCH_FS (time uniform), RGBA16F/HALF_FLOAT targets w/ fallback, single 320px work size (down4 pass removed), blur zero edges, sigma floor 1.0; blur uniform renamed `texelStep` (avoid shadowing GLSL `step()`).
Verified: compile + unit tests OK; installed on Z2351N (Mali-G57, GLES 3.2), app runs, no Backdrop errors/fallback in logcat. Not verified: visual (screenshot black, Firefox was focused).
Follow-up (learner: keep grey animated bg when empty; chose "Grey like Linux"): empty queue already gets Rust GREY_PALETTE blocks via backdropLoadBlockArtwork (unchanged); null-artwork placeholder now 1x1 rgb(51,51,54) like Linux (music note removed). Build + tests OK, installed. Not verified visually (phone on lock/notification shade).

## Android library mode toggle icons
Status: Implemented (direct request: icons instead of Songs/Albums text, remove checkmark).
AI choices (unreviewed): Icons.Rounded.MusicNote (Songs), Icons.Rounded.Album (Albums); contentDescription keeps labels for TalkBack; `icon = {}` hides the SegmentedButton checkmark.
Verified: build OK, installed. Not checked visually.

## Linux player bar (macOS PlayerBarView port)
Status: Implemented (Implement this step). Learner request: "update the playerbar to be the same as swift".
Learner decision: match macOS. AI-proposed + confirmed: restyle gtk::Scale via CSS (no custom widget), GTK symbolic icons at PlaybackIcons volume thresholds, drop time label.
Implemented: player_bar_view.rs rewritten (seek line, capsule 52px, 190px side columns, 24px icon buttons, inactive = 50% white, cover click opens play view); main.rs CSS `.player-bar*`, `.bar-*`, `scale.bar-slider(.thin)`; removed `.player-bar-container/.player-seek-bar/.active-control` (now unused).
Verified: build 0 warnings, 5 tests, screenshot matches layout. Not verified: hover thickness, drag seek/volume.
Bugfix: bar cover grew 40px to 52px once a track loaded. Cause: gtk::Picture natural size = texture size (80px @2x thumb); set_size_request is only a minimum. Fix: bar cover is gtk::Image with pixel_size 40 (CoverState::current_image added). Verified via size log: 40x40 with cover loaded (was 52x52).

## Player bar icon sharpness
Status: Implemented.
Learner decision: all player bar icons at 16px (was 15/17/13 from macOS pt sizes) — 16 matches GTK symbolic SVG grid and centres on a whole pixel in the 24px button.
Hypothesis not yet visually confirmed by learner.
