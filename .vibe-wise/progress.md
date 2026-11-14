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

## Android UI folder split
Status: Implemented (Implement this step).
Requirement (learner): Android code "still messy compared to other platforms"; components like PlayerBar were not separated and the folder count was far below the other OSes.
Learner decisions: mirror the macOS/Linux shape (Screen > Feature folders, one composable per file, shared views in components/, state in state/, pure helpers in support/); rename `MiniPlayer` -> `PlayerBar` to match the other OSes.
Accepted additions: `LibraryScreen` -> `HomeScreen`, `TrackPane` -> `TrackList`, `CollectionHeader` -> `AlbumHeader`, `Controls` -> `PlayControls` (file/function names had to match the new folders); dropped the dead `Modifier.combinedClickableTap` (it only wrapped `clickable`); `ARCHITECTURE.md` got an Android section.
Implemented: 22 -> 42 main Kotlin files. `screens/home/{HomeScreen,toolbar/{TopBar,SortMenu},albums/{AlbumCarousel,AlbumCard,AlbumGrid,AlbumHeader},musicgrid/{TrackList,TrackRow},playerbar/PlayerBar,queue/{QueueSheet,QueueRow}}`, `screens/play/{PlayScreen,controls/PlayControls,lyrics/Lyrics}`, `screens/settings/SettingsScreen`, `components/{Backdrop,Cover,Toast}` + `components/backdrop/`, `support/{TimeFormat,PlaybackIcons,ThumbnailKey,NaturalCompare,FloatClamped,Placeholders}` (split out of `state/Support.kt`), `state/` keeps 9 files. `StateTest` split: `state/StateTest` 9 + `support/SupportTest` 5.
Verified: `gradlew compileDebugKotlin` OK, `gradlew testDebugUnitTest` 18 pass (9 + 5 + 4). Body diff vs the pre-move files shows renames and extraction only, no logic drift. Not verified: device run, no visual check.
Open (not done): no `theme/Layout` file, so padding / sizes / colors are still inline literals; `AlbumSelection` still merges macOS `AlbumSelection` + `LibraryFilterState`.

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

## Play view (macOS port)
Status: Implemented (learner: Implement this step).
Changes: `screens/play/play_view.rs` overlay layout (left column 50% centred without lyrics / 40% + 40px gutter with lyrics, lyrics 55%, 20px trailing, 24px top), floating 28x28 back chevron at 12/24 that hides after 3s idle and returns on pointer motion; `controls/play_controls_view.rs` info row (elapsed | artist • title | duration, 48px time columns, 15px medium centre), knob-less seek, repeat/prev/play/next/shuffle with flex spacers and 0.9/0.4 active states, mute · thin volume · max-volume; `lyrics/lyrics_view.rs` bold 30 active / 25 inactive, note glyph, 28px row spacing, 260px end inset, click seeks, active class now refreshed via item notify (previously stale on reuse).
Reused `icon_button`, `slider`, `set_active`, `volume_icon` from `player_bar_view` (now pub(crate)).
Divergences (ponytail): cover is `gtk::Image` contain not fill; no lyrics gradient mask (no CSS mask in GTK); no 0.94 scale on inactive lines; inset only pads ends (no centre scroll anchor).
Verified: 0 warnings, 5 tests, screenshots with/without lyrics column.

## Lyrics flicker (advisor-guided)
Status: Implemented.
Advisor diagnosis: compounding — scroll_to vs set_value race, CssProvider reload invalidating cascade and re-emitting page-size (loop), fresh label per connect_bind (blank frame), row.set_opacity per scroll invalidating render nodes.
Changes: new `screens/play/lyrics/fade_bin.rs` (gtk::Widget subclass, BinLayout, snapshot() uses gsk MaskMode::Alpha + linear gradient, 120px fade clamped to 25% height) replaces the per-row opacity walk entirely; factory split into setup (label once) / bind (text + classes + is_active notify) / unbind (disconnect only, keeps child); CSS padding reload guarded by rounded page size; centring unchanged (scroll_to + set_value, loop gone once CSS stops re-emitting).
Learner chose: keep GtkListView, fix the 4 causes (not GtkListBox rewrite).
Verified: 0 warnings, 5 tests. Flicker not visually confirmed by learner.

## Play view runaway resize (fixed)
Status: Implemented.
Trace: window 1100 -> 1103 -> 1110 ... -> 1372 on every `apply` (temporary eprintln in play_view, `notify::default-width` hook).
Cause: `left_col` min = width*0.40 and `lyrics` min = width*0.55 are GTK minimums, plus gutter 40 + trailing 20 -> child minimum 0.95*width+60 > width, so GTK enlarged the window to satisfy it, which re-ran `apply` with the larger width. Independent of the tick; the tick only made it iterate faster.
Fix: fixed gaps are subtracted before ratios (`usable = width - (gutter + trailing)`, column/lyrics from `usable`), so children never demand more than the window gives.
Verified: with 34 lyrics, window stays 1100x720 (logged over 14s), build 0 warnings, 5 tests.

## macOS album cards: hide default cover art once the real cover arrives
Status: Implemented.
Requirement (learner): in `AlbumGridView` / `AlbumCarouselView`, hide the default cover art after receiving the real one.
Facts found: both views render `AlbumCard.swift`; the placeholder is the `Rectangle` + `music.note` in `cover`, always drawn underneath `thumbnail` (only the image was swapped in the `ZStack`). Covers arrive via `.task(id: cacheKey)` -> `CoverState.album`; refreshes via `AppState` `albumCoverLoaded` -> `CoverState.invalidateAlbum`. Core has no synthetic default artwork (`artwork_resolve_album` -> `Option`), so the placeholder is the only default art.
Learner decision: keep the placeholder as the no-cover fallback, but hide it once `thumbnail != nil` (rejected deleting it outright).
Implemented: `AlbumCard.cover` placeholder wrapped in `if thumbnail == nil { ... }`; real image + selection/hover border unchanged. `AlbumGridView` / `AlbumCarouselView` untouched (they only pass the card).
Verified: `swift build` OK. Not verified: visual fade-in (placeholder now disappears 0.3s before the image finishes easing in).

## Cross-platform dedup: push shared logic into Rust
Status: Build checkpoint open (criterion + how much state Rust owns).
Requirement (learner): less per-platform code; suggested process = scan per feature, decide if it can be Rust-only, update the md, repeat. Asked for an analysis first.
Evidence gathered:
- Per-platform state layers: macOS 994 LOC, Windows 1451, Linux 1203, Android 840. Views: 3854 / 10062 / 4614 / 2985.
- Already Rust: playback/queue commands, scan + DB, metadata, cover fetch/dedup + JPEG thumbnails, LRC parse, backdrop palette/blocks, view-model snapshots, event stream.
- Duplicated 4x: sort/filter/search (mac `range(of:options:)`, Linux `glib::FilenameCollationKey`, Windows hand-written `NaturalCompare`, Android `Normalizer` NFD fold); `settings.json` reader/writer (mac/linux/windows/android each hand-rolled); 250ms playback clock + time labels (4 copies; core emits only on discrete actions); queue drop-offset math; album column/cover-size formulas.
- Correctly per-platform: decoded bitmap caches (NSImage/BitmapImage/Texture), shaders (GPU API per platform), layout constants (DPI/fonts), view code.
- Intersects an open pending decision: "Backdrop color tuning" (which layer owns the look) still open.
Learner decision (partial): settings persistence moves to Rust only, all platforms consistent — implemented, see "Settings owned by the core" below.
Pending decision: the general rule for "forceable to Rust", whether Rust keeps filter/sort state (stateful `library_list`) or exposes pure functions, and migration order for the remaining duplicated logic (sort/filter/search, playback clock, album metrics formulas).

## Settings owned by the core (all platforms)
Status: Implemented. Learner: "migrate it to Rust only and make all platforms consistent".
Requirement: settings were 4 hand-rolled `settings.json` readers/writers that had drifted (Linux only wrote `backdrop_source`/`animated_background`, Android had no `discordRpc`, key casing differed).
Learner decision: the core owns the file and the rules; UIs keep an observable mirror only.
Implemented:
- Rust: `services/settings.rs` (load/save, atomic tmp+rename, camelCase with snake_case aliases, `.bad` copy on unreadable file, folder trim/dedupe, volume clamp, unknown backdrop -> `artwork`), `view_models/settings.rs` (`SettingsViewModel`), `engine/settings.rs` (`settings_get`/`settings_set`), `uniffi_api/settings.rs`, `c_api/settings.rs` (`fluyer_settings_get`/`fluyer_settings_set`, JSON).
- macOS: `SettingsState` is a mirror (no disk I/O); `AppState.attach` sets `settings.engine` + `load()`; `scanFolders` scans the folders the core actually stored.
- Windows: same shape via `IFluyerEngine.GetSettings/SetSettings` + `fluyer_settings_*` P/Invoke; `AppState.AttachEngine` loads; `App.xaml.cs` no longer builds a path.
- Android: `SettingsState` mirror over `FluyerAppEngineInterface`; `AppState.attach` loads.
- Linux: `settings_state.rs` mirrors the core directly; fields with no Linux switch (folders, volume, Discord) are carried through unchanged; `AppState::init` loads.
Tests moved with the logic: file/JSON/normalization coverage is now Rust (`services::settings::tests`, 5 tests); platform tests kept only mirror behavior + path matching.
Verified: `cargo test -p fluyer_core` 31 + 2 integration pass; `swift build` + `swift test` 65 pass; `gradlew compileDebugKotlin` + `testDebugUnitTest` 18 pass; Swift/Kotlin bindings regenerated; `nm` shows both `fluyer_settings_*` exports.
Not verified: `dotnet build`/`dotnet test` (this machine's dotnet is broken: `[/usr/local/share/dotnet/host/fxr] does not contain any version-numbered child folders`); Linux crate cannot be built on macOS (no GTK dev, so the Rust-core dependency is compiled there only).
Note: `cargo test` needs an absolute `DYLD_LIBRARY_PATH=$PWD/libs/macos` — a relative one resolves against the crate dir and the test binary silently fails to load BASS.

## YouTube Music provider (core) + signed profile updates
Status: Implemented (core + both FFI layers). No UI wired yet.
Requirement (learner): browse music/albums/lyrics, pass streaming onward; cookie/OAuth auth via external browser (device code first); auto update of the unofficial API "with confirmation"; providers stay separate modules, sharing the profile fetcher; secrets in platform keystore; store builds skipped.
Learner decisions: everything in Rust (parsers too, not JS); confirmation prompt before a profile update is applied; separate provider modules; shared profile/secret plumbing.
Implemented:
- `services/profile.rs`: `Profile` (version, clientVersion, apiKey, endpoints, authClientId/Secret), bundled placeholder via `include_str!`, cached file wins on restart, `check()` downloads + ed25519-verifies (`.sig` asset) and only parks it as pending, `apply()`/`dismiss()` (dismissed version not offered again), source URL from `FLUYER_PROFILE_URL` env (no baked release URL yet), `PROFILE_PUBKEY_HEX` empty so an unkeyed build applies nothing.
- `services/secrets.rs`: `SecretStore` trait; `Secrets` = keyring (macOS/Windows/Linux via target-gated deps) then `EncryptedFile` (HKDF(machine id) -> AES-256-GCM; Android/no-machine-id uses a random `secrets.key` file); `Memory` for tests.
- `services/ytmusic/`: `mod.rs` (WEB_REMIX envelope from the profile, search/album/lyrics/stream_formats), `parsers.rs` (tolerant `nav`/`text` walkers, search/album/lyrics/streaming), `models.rs` (records), `auth.rs` (device code begin/poll, refresh, token in secret store).
- Engine: `engine/provider.rs` (`*_blocking` calls, provider cached per applied profile version, check/apply/dismiss). FFI: `uniffi_api/provider.rs` (async + `ProfileUpdate` confirm record), `c_api/provider.rs` (sync JSON, `fluyer_provider_*`).
Tests: core 51 lib + 2 integration pass (parsers, profile signature/apply/dismiss, secrets round-trip/no-plaintext, token parsing, envelope/endpoint override).
Verified: `cargo test -p fluyer_core`; `swift build` + `swift test` 65; `gradlew compileDebugKotlin` + `testDebugUnitTest` 18; Swift/Kotlin bindings regenerated.
Not verified: no live network call was made, fixtures are hand-written samples (not captured responses), no UI, cipher solving not implemented (`signatureCipher` passed through), Windows build untestable on this machine.

### Provider probing (no UI, before Swift work)
Tool: `crates/fluyer_core/examples/ytmusic_probe.rs` — runs `services::ytmusic` directly (no engine, no BASS, no DB).
- `--profile <file>` needs a real profile payload (`version>0`, `clientVersion`, `apiKey`), e.g. `/tmp/ytm-profile.json`.
- Live: `search`, `album`, `lyrics`, `stream`, `login` (device code + poll). Offline: `parse <saved-response.json> search|album|lyrics|stream`.
Live results (2026-11-13, WEB_REMIX clientVersion `1.20241127.01.00`, ytmusicapi's public api key):
- `search` OK: real rows (album + songs) with browseId/videoId/thumbnails.
- `album` OK: header name/artist/year/cover, track list with durations.
- `lyrics` OK on a track that has lyrics; the first test track genuinely had none (browse returned `messageRenderer: "Lyrics not available"`).
- `stream` initially failed (`WEB_REMIX` → UNPLAYABLE, `ANDROID_MUSIC` → LOGIN_REQUIRED). Cause was the client, not a PO-token wall: the VISIONOS recipe (ytm-rs) works, and the missing piece was `visitorData` + `www.youtube.com` + the client headers. Now implemented and verified end-to-end.
Tested: URL builder double slash bug found by the live probe (`/v1//search` → 404), fixed, unit test added (`url_has_exactly_one_separator_and_carries_the_key`).

### Streaming via VISIONOS + QuickJS n/sig solver
Status: Implemented (core). Verified live.
Learner: "update the Rust core. You can add QuickJS".
Design (learned from `grapedjuice/ytm-rs`, GPL-3 → approach only, no code copied):
- `player` goes to `www.youtube.com/youtubei/v1/player` with client `VISIONOS` 1.02, `X-YouTube-Client-Name: 101`, `X-Goog-Visitor-Id`, `Origin`. Returns plain `url`s, no PO token. `visitorData` is captured from any earlier response (`responseContext.visitorData`), else fetched with a `FEmusic_home` browse.
- `services/ytmusic/cipher.rs`: yt-dlp **EJS 0.8.0** solver vendored as `assets/ejs/jsc.min.js` (Unlicense; bundles meriyah ISC + astring MIT; rebuilt with esbuild), run in `rquickjs` 0.14. Player is preprocessed once per player id (`jsc({type:"player"…})`), cached at `<data dir>/ejs/<id>.js` + `.sts`; each solve re-extracts functions from the cached preprocessed player (`jsc({type:"preprocessed"…})`). Worker thread owns the runtime (QuickJS is not `Send`), interrupt watchdog 60 s, exits after 20 s idle to return ~20 MB. Solver failure only costs speed (`n` = throttle), never playback.
- `stream_formats` now rejects non-OK `playabilityStatus` with the reason, solves `n`/`sig` on the response (best effort), then parses.
- Probe gained `player` (prepare solver, prints id/sts) and `solve <n>` commands.
Verified live: `player d65a3158 signatureTimestamp 20769` in 5.2 s cold (cached after); `solve AbCdEfGhIjKlMn → SQKzz0hUiPk` (identical to node/esbuild run); `stream` → 23 formats, 5 audio (itag 140 aac 131k, 251 opus 134k), no `n` in VISIONOS URLs; `GET itag=140` → 206, `audio/mp4`, 200 000 bytes.
Cross-compile: `rquickjs` needs `features=["bindgen"]` on Android (no prebuilt aarch64 bindings) and `BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android` pointing at the NDK sysroot/clang includes; `scripts/build_android.sh` now sets it.
Verified: cargo test 56 lib + 2 integration; `swift build` + `swift test` 65; `gradlew compileDebugKotlin` + `testDebugUnitTest` 18; `build_android.sh` cross-build OK.
Not verified: playback through BASS (`BASS_StreamCreateURL` not bound yet — URLs fetched with curl only), solve timing in-process on a warm runtime (only cold CLI runs measured), Windows.

### macOS YouTube Music tab (anonymous)
Status: Implemented. Learner: "implement the whole app. Create a new tab button at the left top which toggles between local and Youtube Music. Anonymous for now".
Core:
- `provider_play_track`: resolve formats, pick AAC over Opus (`pick_audio_url`), download once into `<cache>/ytmusic/<videoId>.m4a`, then hand BASS a local path. BASS's own URL streaming has **no TLS** on macOS (verified: local `http://127.0.0.1` stream plays, googlevideo `https://` hangs), so the core fetches over rustls. The CDN truncates a plain GET at ~1 MB; the request sends `Range: bytes=0-` (206, full file).
- UniFFI `async` methods now hop onto the engine tokio runtime (`FluyerEngine::on_runtime`): UniFFI polls exported async methods on its own executor, so calling `reqwest` directly panicked with "there is no reactor running". Verified with `futures::executor::block_on(engine.provider_search(..))` → 30 results, no panic.
- `assets/profile.json` now ships usable anonymous constants (version 1, WEB_REMIX client version, public api key) instead of the version-0 placeholder, so the tab works before any profile update lands.
UI (macOS): `Screens/Home/YoutubeMusic/{YoutubeMusicState,YoutubeMusicView}.swift` (debounced search, rows with thumbnails, click → `providerPlayTrack`), `BrowseSource` enum + `AppState.youtube`, segmented toggle in the toolbar's leading slot, local-only toolbar controls hidden in YouTube mode, `Layout.YoutubeMusic` constants.
Verified: cargo test 57 lib + 2 integration; swift build + swift test 65; build_android.sh; `swift run` launches, screenshot shows the new leading toggle; core path `play ju5cgtOJDuE` → BASS position advanced 0→6.3 s / 247 s.
Not verified: clicking the toggle / playing from the app UI (no accessibility permission for UI automation here), Windows/Linux/Android UI (core only), queue/album/artist pages in the tab.

## Play view: fixed back button + real 50% lyrics inset
Status: Implemented.
Requirement (learner): lyrics top/bottom padding is not really half the height, and the back button sits inside the controls container instead of a fixed top-left.
Cause found: `PlayView.body` put `backButton` inside `ZStack(alignment: .topLeading)`, and the `ZStack` shrink-wraps the cover column (`0.5·width` + 20 trailing), so the outer `.frame(width:height:)` centred that box — chevron measured at ≈376pt from the window edge instead of 12. `Layout.Lyrics.verticalInset` was a fixed 260, enough to centre only when the viewport is ≈520pt tall.
Learner reasoning: CSS `position: fixed` has a SwiftUI equivalent (anchor to a full-window container, not the size-deciding one); the lyric inset should come from the container size because it adapts.
Implemented: `backButton` moved to `.overlay(alignment: .topLeading)` on the full-size frame in `PlayView.swift`; `LyricsView` wrapped in `GeometryReader` and `.padding(.vertical, geo.size.height / 2)` replaces `Layout.Lyrics.verticalInset`, which is deleted. Middle lines were never affected — inset only decides whether the first/last line can reach `anchor: .center`.
Verified: `swift build` OK; screenshot with the play view forced open shows the chevron at ≈26pt from the window's left edge. Lyric centring not visually checked (no track with lyrics loaded).
Note: commit `e5c4a0de` also carries the learner's uncommitted `Layout.PlayControls` tweaks (paddings, spacings).

## Shared cover card + macOS YouTube Music home feed
Status: Implemented.
Requirement (learner): "view the home feed and also auto algorithm by youtube" — then, on layout, "generalise it since it's basically just a card of cover art, title, and artist… reuse the item but separate the container to make it consistent. We can do macOS for now".
Facts found: `AlbumCarouselView` (macOS) is a horizontal `LazyHStack` of `AlbumCard` sized by `AlbumMetrics`; `MusicGridView.xaml` (Windows) is a vertical `ListView` of track rows, so only the carousel matches a feed shelf. `AlbumCard`'s `cover` + `labels` bodies were index-agnostic — reusable; its thumbnail plumbing (`CoverState.album(index:px:)`) is not.
Learner decision: one shared cover card component, containers stay separate per surface. Autoplay/radio (`RDAMVM` continuation paging) deliberately out of scope until the queue-ownership decision is made.
Implemented:
- Core: `FeedShelf { title, items: [SearchItem] }`; `parse_search`'s row mapping factored to `item_of`; `parse_home` collects every `musicCarouselShelfRenderer` (drops empty shelves) and now reads both `musicTwoRowItemRenderer` (feed cards) and `musicResponsiveListItemRenderer` (song rows) in render order; `YouTubeMusic::home()` → `browse {browseId: FEmusic_home}`; `provider_home` (async via `on_runtime`) + blocking; UniFFI export; probe `home` command.
- Bug found live: feed cards carry their image under `thumbnailRenderer.musicThumbnailRenderer.thumbnail.thumbnails`, not `thumbnail.*` — the second shelf rendered placeholders until `best_thumbnail` learned that path.
- macOS: `Shared/Views/CoverCard.swift` (cover + labels + selection/hover border + optional context actions, image source injected as `cached`/`load` closures); `AlbumCard` is now a thin mapping; `YoutubeMusicState.shelves` + `loadHome()` (fetched once); `YoutubeShelfView` (title + `LazyHStack` of `CoverCard` via `AlbumMetrics`) with an `NSCache` remote-cover store; `YoutubeMusicView` shows shelves while the query is empty.
Verified: `cargo test -p fluyer_core` 58 lib + 2 integration; probe `home` live → 2 shelves / 30 items (`Quick picks` 20 songs, `Trending community playlists` 10); `swift build` + `swift test` 65; screenshot with the tab forced open shows both shelves with covers and labels.
Not verified: clicking a card to play from the UI (no accessibility permission), album/artist/playlist drill-down (nonexistent), Windows/Linux/Android.

### Follow-up: shelf renderer follows the item kind
Status: Implemented. Learner: "detect whether it's playlist or track. Playlist use album card and track use music card."
`YoutubeShelfView.isTrackShelf` (`items.first?.kind == .track`) picks the renderer: tracks → vertical `YoutubeTrackRow` rows, playlists/albums → the `CoverCard` carousel. The search row was promoted out of `YoutubeMusicView` into `Screens/Home/YoutubeMusic/YoutubeTrackRow.swift` (now shared by search results and track shelves), along with `SearchItemKind.label`.
Verified: `swift build`, `swift test` 65; screenshot of the tab shows "Quick picks" as rows and the playlist shelf as cards. Not verified: clicking rows/cards.

### Follow-up: track grid styling, no search field
Status: Implemented (uncommitted; learner commits).
- `YoutubeTrackRow` restyled to mirror `TrackCard` (`Layout.TrackRow` metrics: 44 cover, `coverToText`/`titleToSubtitle`/`iconToTitle`, `verticalPadding` 6, `.tint` title + speaker icon while playing), no hover chrome, no fixed row height; trailing label only for items with no video id.
- Track shelves use the library's adaptive `LazyVGrid` (`Layout.MusicGrid` minColumnWidth/columnSpacing/rowSpacing/horizontalPadding).
- Container spacing: `HomeView` no longer pads `YoutubeMusicView` (removed the doubled 16pt); each piece pads itself like its library counterpart (grid 16, carousel edge 16, shelf title 16).
- Search UI removed from the tab: `YoutubeMusicState` keeps `shelves`/`loadHome`/`play`/`playingVideoId`/`error`; `query`, debounce, `items`, `isSearching`, `search()` deleted. Unused `Layout.YoutubeMusic` field/layout constants deleted.
Pending decision: sort/order for the feed (learner asked for ideas; no choice yet).
Verified: `swift build`, `swift test` 65. Not visually verified (app not frontmost for screenshots).

### Follow-up: provider cover art + real artist credit
Status: Implemented (uncommitted; learner commits).
Bug: remote tracks are not in the library, so `artwork_resolve_track` found no embedded cover and fell through to MusicBrainz, and the queue had no cover at all (`QueueRow.libraryIndex` looks the track up by path). Artist came from `subtitle.split("•")[1]`, which on feed rows is `"3.6B plays"`.
Learner reasoning: none recorded — direct fix request.
Implemented:
- Core: `provider_play_track(_blocking)` takes `thumbnail`, downloads it and writes it into the artwork disk cache under the exact key `artwork_resolve_track` looks up (artist, album, title, audio path) via `cache_remote_cover`, and stores the URL in `MusicMetadata.image`; new `artwork_queue_thumbnail(index, max)` resolves a queue position's cover through `player.queue_get_by_index` (works without a library index); UniFFI `artwork_load_queue_thumbnail`.
- macOS: `ThumbnailKey.queue(px:path:)` + `CoverState.queue(position:px:path:)`; `QueueRow.loadCover` falls back to it when the track is not in the library; `YoutubeMusicState.credit(_:)` drops `plays`/`views` segments before taking artist/album; `play` passes `item.thumbnail`.
- Probe: `--thumbnail <url>` and a `queue cover: N bytes` line.
Verified: probe play with a real cover URL → cache file at `covers/track/Probe Artist - Probe Track.jpg`, `queue cover: 2021 bytes` (a real JPEG, no MusicBrainz call); `cargo test` 58 lib + 2 integration; `swift build`, `swift test` 65. Not verified in the running app (click-through).

### Progressive streaming via a loopback proxy
Status: Implemented (uncommitted; learner commits).
Learner decision: "make it actually streaming instead of downloading the whole track", transport choice = loopback proxy (over `BASS_StreamCreateFileUser`).
Constraint that forced it: macOS BASS has no TLS, so a `https://googlevideo…` URL in `BASS_StreamCreateURL` blocks forever; `http://127.0.0.1` plays.
Implemented:
- `services/stream_proxy.rs`: `StreamProxy` binds `127.0.0.1:0` once (`spawn` on the engine runtime), `link(id, url, expires_in)` registers a route and returns `http://127.0.0.1:<port>/<id>`. Handler parses the request head, resolves the id, forwards `Range` (default `bytes=0-`) over rustls, mirrors `content-type`/`content-length`/`content-range`, adds `accept-ranges: bytes`, answers `HEAD` without a body, then writes the body chunk-by-chunk (`connection: close`, one request per socket). Expired routes are purged on every link/resolve, with 60 s of headroom before the CDN expiry.
- Engine: `stream_proxy` field constructed in `FluyerEngine::new` and spawned before use.
- `engine/provider.rs`: both play paths take `pick_audio_url`'s URL + `expires_in_seconds` and hand the proxy link to `MusicMetadata.path`; `download_audio` and the `ytmusic` download cache are gone. `audio/player.rs` already routed `http://` paths to `BASS_StreamCreateURL`, so queue/bar/seek were untouched.
- Probe: `--seek <seconds>` (calls `player_seek` mid-play) and `--thumbnail <url>`.
Verified: probe `play` → `BASS_StreamCreateURL` on `http://127.0.0.1:60020/ju5cgtOJDuE`, bar advances 770→3770 ms of 247523 (no local `.m4a` written by the play path); `--seek 90` mid-play → bar jumps to 91460 ms and keeps advancing; `cargo test` 58 lib + 2 integration; `swift build`. Not verified: app click-through, expired-URL mid-track behaviour (needs refresh logic), Windows/Linux/Android.
Known leftovers: stale `<cache>/ytmusic/*.m4a` files from the download era are still on disk (no cleanup yet).
