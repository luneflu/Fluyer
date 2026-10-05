# Fluyer Windows — Gap Plan

Target reference: legacy `Fluyer/` (Tauri + Svelte) feature surface.
Baseline: `ui/windows/` at macOS parity. No stubs. No `TODO`/`NotImplemented` in `.cs`.

## Done (macOS parity, verified)

- Library scan → `fluyer_library_scan` + `AppState.ScanFolders`, single-folder picker
- Album carousel, album drill-down (`AlbumSelection`), track grid (`MusicGridView`)
- Player bar transport: play/pause/next/prev/seek/volume/repeat-cycle/shuffle
- Play view + synced lyrics (`GetActiveLyricIndex`, `fluyer_player_get_lyrics`)
- Animated GPU backdrop (D3D11 port) + static fallback (`GenerateBackground`)
- Covers: embedded → disk cache → online fallback, bounded `ThumbnailStore`, single-flight fetch
- Toast, scan progress callbacks, `Fluyer.Tests` (~60 tests) + native smoke test

## Gaps vs legacy `Fluyer/`

| # | Feature (legacy) | Core (`fluyer_core`) | Windows UI | Cost |
|---|---|---|---|---|
| 1 | Search / filterbar (title/artist/album) | DONE in C# (`AlbumSelection.Query`), no FFI | Titlebar `AutoSuggestBox` | — |
| 2 | Queue panel (list, goto, remove, move, shuffle) | DONE: `fluyer_queue_get_json`/`goto`/`remove`/`move` | Queue flyout in player bar | — |
| 3 | Settings page (paths, animated-bg toggle, volume, Discord switch) | DONE: `fluyer_library_remove_folder`, `fluyer_discord_set_enabled`; scans prune missing files; presence updates on every player sync | Settings dialog (titlebar gear) + `settings.json` | — |
| 4 | SMTC / media keys / taskbar | `souvlaki` in `Cargo.toml`, zero call sites | None | Medium: core wire + SMTC |
| 5 | Playlists CRUD + art | Tables in `db/migrations.rs` (`playlists`, `playlist_musics`), zero engine/FFI methods | None | Big: engine + FFI + UI |
| 6 | Folder browser / sidebar tree | Scanner only (`scan_directories`); no `folder_items_get` equivalent | Settings folder manager only; `FolderPicker.PickMusicFoldersAsync` uses `PickSingleFolderAsync` (one dir per pick) | Medium |
| 7 | EQ (10-band) / bit-perfect | Zero matches (`equaliz`, `bit_perfect`, `BASS_FX`) | None | Big, needs DSP design |

Non-goals: tray, global hotkeys (absent legacy too — no plugin/command); mobile bridge (`tauri-plugin-fluyer`); updater; developer log/metrics/screenshot; onboarding intro/swipe guide; icon themes.

## Slices (ordered, cheapest value first)

### Slice 1 — Search + filter (DONE)
- Filtered in C# (`AlbumSelection.Query`) over tracks already in memory; no FFI needed.
- `AutoSuggestBox` in titlebar; click plays unfiltered row so queue stays whole.
- Test: `AlbumSelectionTests.Query_FiltersCaseInsensitive_AndPlaysUnfilteredRow`.
- Skipped: folder-scoped + playlist-scoped filter, add when slices 5/6 land.

### Slice 2 — Queue UI + FFI (DONE)
- Core: `MusicPlayer::queue_snapshot`/`move_track` (+ `remove_track` now emits sync); `FluyerEngine::get_queue_view`/`queue_goto`/`queue_remove`/`queue_move`; FFI `fluyer_queue_*`.
- `QueueState` (reads only while flyout open); flyout on player bar: click = goto, up/down/remove buttons. Shuffle reuses existing transport button.
- Tests: `QueueStateTests` (open-gated load, move clamp, out-of-range no-ops); native smoke calls queue exports.
- Skipped: drag-reorder, add when up/down feels slow on long queues.

### Slice 3 — Settings + persistence (DONE)
- Core: `scan_and_update` now drops rows for files gone from scanned roots (skips missing roots, so an unplugged drive keeps its rows); `remove_folder` drops rows under a folder; `DiscordRpc::update`/`clear` called from `emit_sync_inner`; FFI `fluyer_library_remove_folder`, `fluyer_discord_set_enabled`.
- C#: `SettingsState` (`%AppData%/org.alvindimas05.fluyer/settings.json`, atomic write, corrupt file kept as `.bad`): folders, animated bg, Discord, volume. Launch restores Discord + volume and rescans saved folders; volume saved on window close.
- UI: `ContentDialog` (titlebar gear): folder list + remove, Add folder, Rescan, two toggles. Animated-bg off disposes the GPU renderer and shows the static bitmap.
- Tests: `SettingsStateTests`, `AppStateTests` (restore/rescan, remove forwards stored spelling, Discord toggle), Rust `delete_music_paths` + component-scoped prune; native smoke covers the new exports.
- Skipped: icon themes, bit-perfect, EQ (need core design); multi-select folder picker (slice 6).

### Slice 4 — SMTC integration
- Core: wire `souvlaki` (already dependency) to player sync; expose metadata + transport callbacks.
- Windows: `SystemMediaTransportControls` hookup for keys/taskbar/now-playing.
- Skipped: MPRIS/NowPlaying — Windows-only slice.

### Slice 5 — Playlists (defer, biggest)
- Core: engine CRUD over existing `playlists`/`playlist_musics` tables + FFI + thumbnails.
- UI: playlist views in `Views/`, grid integration via `playlist_paths` filter (already param in `filtered_music`).
- Needs slices 1–2 first (filter + queue patterns reused).

## Next step

Slice 4 (SMTC): media keys + taskbar/lock-screen now-playing.
