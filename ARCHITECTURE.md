# Fluyer Native Architecture (refactor plan)

Status: R1, R2, S1, S2, S3 implemented (macOS). W1, W2, W3 implemented (Windows). macOS visual check pending.
Scope: Rust core (`crates/fluyer_core`), macOS Swift (`ui/macos`), Windows WinUI (`ui/windows`).

## Goal

A human should be able to find the code for anything on screen in one search.
Rule: **search the word you see in the app, land on the file.**

## Decisions (learner)

1. Lookup works by UI keyword: searching "Now Playing" should find the queue view.
   That means file and type names follow what the UI shows.
2. Layout numbers (padding, spacing, sizes) live in **one shared file**. Accepted risk:
   changing one value can change several places.
3. Rust goal is readability. The FFI layer may be rewritten entirely.
4. Order: this document first, then Rust (FFI first), then Swift.
5. People find code by file name as well as by keyword. The queue is the main focus.
6. Rewrite both FFI layers (UniFFI and the Windows C ABI). Windows can't be tested right now.
   Android exists (UniFFI Kotlin, `bindings/kotlin`) and must keep working.
   Goal: renaming an FFI function should be easy.
7. Rust: many small, properly named files are better than a few big ones.
8. "Now Playing" stays with the queue. Full-screen player keeps the Play name (`PlayView`; alt: lyrics page).
9. FFI naming: `<feature>_<verb>`, e.g. `queue_get`, `player_seek`.
10. Windows keeps the hand-written C ABI (P/Invoke + JSON), renamed to the same convention.
11. UniFFI wrapper stays separate, split into small named files.

## Current problems (evidence)

| Where | Problem |
|---|---|
| `Views/Album/AlbumCarouselView.swift` | Also holds `AlbumGridView`, `AlbumCard`, `ResponsiveRule` |
| `Views/Library/MusicGridView.swift` | Hides private `TrackCard` |
| `Views/Queue/QueuePaneView.swift` | Hides private `QueueRow`; header text is "Now Playing" |
| `Views/Library/ContentView.swift` | Window root holds toolbar, sort menu, toast, scan progress, queue hover logic (236 lines) |
| Many views | Inline layout numbers (`12`, `6`, `24`, `40`, `260`); `width - 12 - sidebarWidth` repeated in 3 places |
| `State/AlbumSelection.swift` | Owns mode, sort, search, selection; name hides that |
| `lib.rs` (725 lines) | `FluyerEngine` does playback, queue, scan, covers, thumbnails, lyrics, backdrop, view models |
| `uniffi_api.rs` (449) + `ffi.rs` (885) | Both re-wrap nearly every engine method; one feature = 3 files |
| `audio/player.rs` (1102) | BASS setup, streams, queue navigation, sync in one file |

## Naming collision to resolve

"Now Playing" means two things today:
- the queue panel header (`QueuePaneView`, `Text("Now Playing")`)
- the full-screen player (`PlayView`, help text "Open Now Playing")

Decision 1 only works if one word maps to one place. **Open:** pick names.

## Rust target (proposal, pending learner review)

### Step R2: engine split (DONE)

```
crates/fluyer_core/src/
  lib.rs                 module list + uniffi scaffolding only (16 lines)
  engine/
    mod.rs               FluyerEngine struct + new()
    player.rs            player_*: transport, volume, repeat, bar + play view
    queue.rs             queue_*  ("Now Playing" panel)
    library.rs           library_*: scan, remove folder, tracks (+ prune_rows test)
    album.rs             album_*
    artwork.rs           artwork_*: cover resolution + memoized thumbnails
    lyrics.rs            lyrics_*: resolution + parsed cache
    backdrop.rs          backdrop_*: palette + block artwork
    thumbnail_cache.rs   ThumbnailCache (bounded FIFO)
  uniffi_api/  c_api/    FFI, same feature file names (see R1)
  audio/ db/ library/ metadata/ services/ view_models/   unchanged
```

Each `engine/*.rs` is an `impl FluyerEngine` block. Same struct, split by feature.
Verified: cargo test 26 pass, swift test 66 pass, Android compile OK, C# imports present (`nm`).

FFI consumers today:
- `uniffi_api.rs` serves macOS (Swift) and Android (Kotlin).
- `ffi.rs` (C ABI + JSON) serves Windows only (`FluyerNative.cs`).

Rename inconsistencies to fix (examples): `seek` vs `set_pos`;
`get_current_image` vs `current_thumbnail`; `generate_background_for_current` vs
`load_animated_background`; `fluyer_library_play_index` vs `play_single_from_library`.
`audio/player.rs` split is a later step, not part of the first pass.

### Step R1: FFI layer (DONE)

```
crates/fluyer_core/src/
  uniffi_api/        macOS (Swift) + Android (Kotlin)
    mod.rs           FluyerAppEngine struct + constructor
    error.rs         FluyerError
    events.rs        FluyerEvent, FluyerEventListener, EventSinkBridge
    player.rs        player_*
    queue.rs         queue_*
    library.rs       library_*
    album.rs         album_*
    artwork.rs       artwork_*
    lyrics.rs        lyrics_*
    backdrop.rs      backdrop_*
    discord.rs       discord_*
  c_api/             Windows (P/Invoke + JSON)
    mod.rs           fluyer_init / fluyer_free
    memory.rs        fluyer_string_free / fluyer_bytes_free + JSON/bytes helpers
    events.rs        FluyerCallbacks, FluyerPlayerState, CallbackSink
    player.rs ... discord.rs   same names as uniffi_api/, prefixed fluyer_
                     (no backdrop.rs: Windows does not call a backdrop export)
```

Rename map (UniFFI name; C name = `fluyer_` + same; Swift/Kotlin get camelCase):

| File | Old | New |
|---|---|---|
| player | toggle_play, next, previous, seek, get_position, set_volume, cycle_repeat, shuffle | player_toggle_play, player_next, player_previous, player_seek, player_get_position, player_set_volume, player_cycle_repeat, player_shuffle |
| player | get_player_bar_view, get_play_view | player_get_bar, player_get_play_view |
| queue | get_queue_view, queue_goto/remove/move/clear | queue_get, queue_goto, queue_remove, queue_move, queue_clear |
| library | scan_directories, remove_folder, get_scan_status | library_scan, library_remove_folder, library_get_scan_status |
| library | get_track_count, get_track_view, play_all_from_library, play_library_tracks | library_get_track_count, library_get_track, library_play_all, library_play_tracks |
| album | get_album_count, get_album_card, get_album_detail, play_album, queue_album, shuffle_album | album_get_count, album_get_card, album_get_detail, album_play, album_queue, album_shuffle |
| artwork | load_track/album/current_thumbnail | artwork_load_track_thumbnail, artwork_load_album_thumbnail, artwork_load_current_thumbnail |
| lyrics | get_active_lyric_index | lyrics_get_active_index |
| backdrop | load_block_artwork | backdrop_load_block_artwork |
| discord | set_discord_enabled | discord_set_enabled |

Deleted (0 callers): 23 UniFFI methods/functions, 20 C exports, matching C# wrappers
(`Play`, `Pause`, `GetVolume`, `RequestSync`, `PlaySingleFromLibrary`, `GenerateBackground`).
Engine methods renamed to the same `<feature>_<verb>` names in `lib.rs`.

Verified on macOS: `cargo test` (26 pass), Swift bindings regenerated, `swift build` +
`swift test` (66 pass), Android bindings regenerated + `gradlew compileDebugKotlin
testDebugUnitTest` OK, every `FluyerNative.cs` import exists in the built library (`nm`).
Not verified: `dotnet build` (no .NET runtime here), Windows runtime.

## Swift target (confirmed design)

Pattern: MVVM-like. Views draw state and forward actions; state objects own logic and
are the only code that calls the engine. Names: `XState` + `XView`.

```
ui/macos/Sources/
  App/            FluyerApp, AppDelegate
  Screens/
    Home/         HomeView                    (was ContentView)
      MusicGrid/  MusicGridView, TrackCard
      Albums/     AlbumCarouselView, AlbumGridView, AlbumCard, AlbumHeaderView
      Queue/      QueueView, QueueRow, QueueEdgeTrigger, QueueState   ("Now Playing")
      PlayerBar/  PlayerBarView
      Toolbar/    SortMenu, ToolbarSearchField
      LibraryFilterState                       (was AlbumSelection)
    Play/         PlayView
      Controls/   PlayControlsView
      Lyrics/     LyricsView
    Settings/     SettingsView
  Shared/
    State/        AppState, EngineHandle, PlaybackState, PlaybackClock, LibraryState,
                  SettingsState, ToastState, CoverState (new)
    Support/      TimeFormat, PlaybackIcons, ThumbnailKey, ThumbnailStore, NowPlayingCoordinator, ...
    Theme/        Layout.swift
  Components/     SliderTrack, CurrentCoverView, AnimatedBackgroundView, ArtworkBackdropRenderer,
                  ToastView, SidebarOcclusion
```

Rules:
- Feature state lives with its screen; state used by several screens lives in `Shared/State`.
- Views used by several screens live in `Components/`.
- One view type per file; file name = type name.
- Views receive only the state objects they use (no whole `AppState`).
- Transient look-only state (hover, idle timer) stays `@State` in the view.
- Home and Play player bars are different views over the same `PlaybackState`.
- Layout numbers live in `Shared/Theme/Layout.swift`, named by role, grouped by feature.

Steps:
- S1 (DONE): move/rename files, one view type per file. No behavior change.
  Renames: ContentView->HomeView, QueuePaneView->QueueView, CollectionHeaderView->AlbumHeaderView,
  EdgeTrigger->QueueEdgeTrigger, AlbumSelection->LibraryFilterState, AlbumCarouselView.Metrics->AlbumMetrics.
  Split out: AlbumGridView, AlbumCard, AlbumMetrics, TrackCard, QueueRow, SortMenu, ToastView,
  PlayControlsView, LyricsView, MetalBackdropRepresentable, MetalBackdropView.
  Small option enums (LibraryMode, TrackSort, AlbumSort, BackdropSource) stay with their state.
  Verified: swift build, swift test 66 pass.
- S2 (DONE): `Shared/State/CoverState` is the only image path to the engine; every view
  receives only the state objects it uses (no `AppState` below `HomeView`/`SettingsView`).
  `NowPlayingCoordinator` and the backdrop also go through `CoverState`. Test: `CoverStateTests`.
- S3 (DONE): every padding / spacing / frame size / corner radius in `Screens/` and
  `Components/` comes from `Shared/Theme/Layout.swift` (`Layout.Queue.panelPadding`, ...).
  Not moved: font sizes, shadows, blur, opacities, animation timings.

## Windows target (port of the Swift rules)

Same rules as the Swift target: UI-word names, Screen > Feature folders, one view per
file, views get only the state they use, one shared layout file. Applied by the agent
on a direct "execute" request; the Windows-specific choices below were the agent's,
not learner decisions, and are open to change.

Windows has two projects. `Fluyer.Core` (net8.0, no WinUI) holds state + P/Invoke and
is what `Fluyer.Tests` can test. `Fluyer` (WinUI) holds XAML views. Both mirror the same
folder tree, so a feature's state and view sit at the same path in each project:

```
ui/windows/
  App.xaml(.cs), MainWindow.xaml(.cs)      MainWindow = Home window shell (macOS HomeView)
                                           + MenuBar in the title bar = macOS menu bar
                                           (File / Playback / View, same items + shortcuts)
  Screens/
    Home/
      Albums/     AlbumCarouselView, AlbumHeaderView        (was CollectionHeaderView)
                  AlbumGridView (Albums mode)
      Toolbar/    SortMenu
      MusicGrid/  MusicGridView
      Queue/      QueueView                                 (was QueuePaneView)
      PlayerBar/  PlayerBarView
    Play/         PlayView
    Settings/     SettingsView   (split out of MainWindow's dialog)
  Components/     CurrentCoverView, SliderTrack, ToastView (new, split out of MainWindow),
                  SidebarOcclusion
    Backdrop/     AnimatedBackgroundView, ArtworkBackdropRenderer, BackdropShaders
  Shared/
    State/        CoverState (new)
    Support/      ThumbnailStore, CoverImages, FolderPicker, MediaTransportCoordinator
    Theme/        Layout.xaml (+ Layout.cs for code-behind)
                  Colors.xaml (theme-resource overrides: no accent blue)
  Fluyer.Core/
    Native/       FluyerEngine, FluyerNative, Models   (C ABI)
    Screens/Home/ LibraryFilterState                    (was AlbumSelection)
      Queue/      QueueState
    Shared/State/ AppState, EngineHandle, IThumbnailInvalidator, LibraryState,
                  PlaybackState, PlaybackClock, SettingsState, ToastState
    Shared/Support/  TimeFormat, ThumbnailKey, PlaybackIcons, ...
    Components/Backdrop/  BackdropMesh, BackdropUniforms
```

C# namespaces follow the top folder (`Fluyer.Screens`, `Fluyer.Components`,
`Fluyer.Shared`); `Fluyer.Core` keeps its old namespaces.

Steps:
- W1 (DONE): move/rename as above. Tests split one class per file
  (`LibraryStateTests`, `LibraryFilterStateTests`, `ToastStateTests`).
- W2 (DONE): `Shared/State/CoverState` is the only UI code that asks the engine for
  images (grid, carousel, current cover, backdrop, media overlay). It lives in the WinUI
  project because it loads into `Image` via the UI-thread `ThumbnailStore`. Views take
  narrow state objects (`QueueView.Queue`, `MusicGridView.Library/Filter/Covers`, ...);
  only `MainWindow` and `SettingsView` take `AppState`. Menu "Play All" raises an event
  the window forwards to `AppState.PlayAll`.
- W4 (DONE, learner: "exact same as macOS"): left menu sidebar (MenuView, Ctrl+M) removed;
  WinUI `MenuBar` in `TitleBar.LeftHeader` mirrors `FluyerApp.swift` `.commands`. Shortcuts
  use Ctrl for Cmd: Ctrl+O, Ctrl+Left/Right, Ctrl+Shift+P, Ctrl+L (queue, was Ctrl+Q).
  Settings moved from the title-bar gear to File > Settings... (no Ctrl+, yet: XAML can't
  parse that key). No Songs/Albums items: Windows has no library mode toggle yet.
- W5 (DONE, learner: "filter like sorting etc like at the macos"): `LibraryFilterState` ported in
  full: `LibraryMode` (Songs/Albums), `TrackSort`, `AlbumSort`, shared `SortAscending`,
  `DisplayedAlbums`, diacritic-insensitive search, natural sort ("2" before "10";
  hand-written, .NET 8 lacks `NumericOrdering`). New `Screens/Home/Albums/AlbumGridView`
  (Albums mode, right-click Play / Add to Queue / Shuffle) and `Screens/Home/Toolbar/SortMenu`.
  Title bar (RightHeader, next to the window buttons, macOS order): scan progress,
  Songs/Albums toggle (Ctrl+1 / Ctrl+2, also in View menu), sort button, search.
  Playing a sorted list: new C export `fluyer_library_play_tracks` (same as UniFFI
  `library_play_tracks`); unused `fluyer_album_play_track` + engine `album_play_track` deleted.
- W3 (DONE): every padding / spacing / size / corner radius in `Screens/`, `Components/`,
  `MainWindow.xaml`, `App.xaml` comes from `Shared/Theme/Layout.xaml`. Keys are
  `<Feature><Role>` (`QueuePadding`, `PlayerBarCornerRadius`). Code-behind layout math
  (carousel slots, grid columns, play-screen cover side, slider thickness, window size)
  reads the same keys through `Layout.Number(...)` / `Layout.Edges(...)`.
  Not moved: font sizes, opacities, timings, the carousel responsive ratio table.

Verified: `dotnet build` (x64) OK, `dotnet test` 112 pass, app launches and renders the
library (window capture). Not checked: queue/menu sidebars, play screen, settings dialog.

## Open questions

1. Later: split `audio/player.rs` (1102 lines).
2. Swift S1-S3.

## Where to change X (fill in after the move)

| I want to change | File |
|---|---|
| A Swift/Kotlin engine call, e.g. `queueGet` | `crates/fluyer_core/src/uniffi_api/queue.rs` |
| A Windows export, e.g. `fluyer_queue_get` | `crates/fluyer_core/src/c_api/queue.rs` + `ui/windows/Fluyer.Core/Native/FluyerNative.cs` |
| Engine logic behind a call, e.g. queue | `crates/fluyer_core/src/engine/queue.rs` |
| Events sent to the UI | `uniffi_api/events.rs` / `c_api/events.rs` |
| Any padding / spacing / size | `ui/macos/Sources/Shared/Theme/Layout.swift`, section named after the feature |
| Cover loading (any view) | `Shared/State/CoverState.swift` |
| Queue panel look ("Now Playing") | `ui/macos/Sources/Screens/Home/Queue/QueueView.swift`, rows: `QueueRow.swift` |
| Queue behavior | `Screens/Home/Queue/QueueState.swift` |
| Song grid / one song row | `Screens/Home/MusicGrid/MusicGridView.swift` / `TrackCard.swift` |
| Album strip, grid, card, sizes | `Screens/Home/Albums/` (`AlbumMetrics.swift` = card sizing) |
| Bottom player bar | `Screens/Home/PlayerBar/PlayerBarView.swift` |
| Play screen layout / controls / lyrics | `Screens/Play/PlayView.swift` / `Controls/PlayControlsView.swift` / `Lyrics/LyricsView.swift` |
| Window shell, toolbar layout | `Screens/Home/HomeView.swift`, `Screens/Home/Toolbar/` |
| Shared state (playback, library, settings) | `Shared/State/` |
| Shared views (slider, cover, toast, backdrop) | `Components/` |
| Windows: any padding / spacing / size | `ui/windows/Shared/Theme/Layout.xaml`, keys prefixed with the feature |
| Windows: queue panel look / behavior | `ui/windows/Screens/Home/Queue/QueueView.xaml` / `Fluyer.Core/Screens/Home/Queue/QueueState.cs` |
| Windows: cover loading | `ui/windows/Shared/State/CoverState.cs` |
| Windows: window shell, menu bar, queue sidebar | `ui/windows/MainWindow.xaml` |
