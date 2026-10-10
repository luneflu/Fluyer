# Fluyer Native Architecture (refactor plan)

Status: R1 (FFI layer) and R2 (engine split) implemented. Swift pending.
Scope: Rust core (`crates/fluyer_core`) and macOS Swift (`ui/macos`). Windows UI is out of scope.

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

## Swift target (proposal, pending learner review)

```
ui/macos/Sources/
  App/            FluyerApp, AppDelegate, MainWindow (was ContentView)
  Theme/Layout.swift     every padding/spacing/size constant, named by role
  Features/
    Songs/        SongList, SongCard
    Albums/       AlbumCarousel, AlbumGrid, AlbumCard, AlbumHeader
    Queue/        QueuePanel, QueueRow, QueueEdgeTrigger, QueueState
    NowPlaying/   full-screen player: screen, controls, lyrics
    PlayerBar/    PlayerBar
    Toolbar/      SortMenu, SearchField, LibraryFilter (was AlbumSelection)
    Backdrop/     AnimatedBackground, ArtworkBackdropRenderer
    Settings/     SettingsView, SettingsState
    Toast/        Toast, ToastState
  Shared/         SliderTrack, CurrentCover, SidebarOcclusion, helpers
```

Rules:
- One view type per file; file name = type name.
- Each file starts with a one-line comment listing the UI words it renders
  (e.g. `// UI: "Queue is empty", Clear Queue button`) so keyword search hits it.
- Layout constants named by role (`Layout.panelInset`), not by value (`Layout.twelve`),
  so changing one meaning does not silently change an unrelated one.

## Open questions

1. Later: split `audio/player.rs` (1102 lines).
2. Swift step: confirm the proposed layout above.

## Where to change X (fill in after the move)

| I want to change | File |
|---|---|
| A Swift/Kotlin engine call, e.g. `queueGet` | `crates/fluyer_core/src/uniffi_api/queue.rs` |
| A Windows export, e.g. `fluyer_queue_get` | `crates/fluyer_core/src/c_api/queue.rs` + `ui/windows/Fluyer.Core/Native/FluyerNative.cs` |
| Engine logic behind a call, e.g. queue | `crates/fluyer_core/src/engine/queue.rs` |
| Events sent to the UI | `uniffi_api/events.rs` / `c_api/events.rs` |
| Swift views | after the Swift step |
