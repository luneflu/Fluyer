# Fluyer

Desktop music player. Rust core (`fluyer_core`) holds all logic — library scanning,
metadata, playback, and view models — and exposes it over C ABI + UniFFI. UIs are thin
SwiftUI/C++ shells that only render view models and forward events.

## Layout

| Path | What it is |
| :--- | :--- |
| `crates/fluyer_core/` | Core crate: engine, services, view models. Produces `libfluyer_core.dylib` |
| `crates/fluyer_core/src/view_models/` | Per-component view models (player bar, play view, track item, album card, …) |
| `bindings/swift/` | **Generated** UniFFI Swift bindings + SPM package `FluyerCore` |
| `ui/macos/` | macOS SwiftUI app (SPM executable `FluyerApp`) — the current UI |
| `ui/macos/Sources/State/` | Observable state: `AppState` (coordinator) + `Playback`/`Library`/`Selection`/`Toast` |
| `ui/macos/Tests/` | `FluyerAppTests` — XCTest over the pure logic in `Support/` and `State/` |
| `libs/macos/` | Prebuilt `libbass.dylib`, `libbassmix.dylib` |
| `scripts/` | `generate_swift_bindings.sh`, `bundle_macos_app.sh` |
| `dist/` | Build output: `Fluyer.app`, `Fluyer.dmg` |

Runtime state (created on first launch):

- Database — `~/Library/Application Support/org.alvindimas05.fluyer/fluyer.db`
- Cover + lyrics cache — `~/Library/Caches/org.alvindimas05.fluyer/`

## Prerequisites

- Rust (stable) and the macOS Command Line Tools
- Swift 5.9+ toolchain (Xcode or standalone)
- `libs/macos/*.dylib` must be present — they are gitignored, so a fresh clone needs them
  copied in before anything links

## Build

### 1. Core

```bash
cargo build -p fluyer_core           # debug   -> target/debug/libfluyer_core.dylib
cargo build -p fluyer_core --release # release -> target/release/libfluyer_core.dylib
```

### 2. Regenerate Swift bindings

Only needed after changing the `#[uniffi::export]` API surface in
`crates/fluyer_core/src/uniffi_api.rs`. It rebuilds the dylib, runs `uniffi-bindgen`,
and rewrites `bindings/swift/`.

```bash
./scripts/generate_swift_bindings.sh
```

> `Warning: Unable to auto-format fluyer_core.swift using swiftformat` is harmless —
> `swiftformat` is optional and only affects whitespace.

### 3. App

```bash
swift build --package-path ui/macos
./ui/macos/.build/debug/FluyerApp
```

`ui/macos/Package.swift` links `libfluyer_core` from `target/debug` plus the BASS
dylibs from `libs/macos`, and sets matching rpaths — so the Swift app needs no
`DYLD_LIBRARY_PATH` when run from the build directory.

#### macOS source layout

```
ui/macos/Sources/
  FluyerApp.swift     @main App
  App/                NSApplicationDelegate
  State/              observable state (see below)
  Views/Library/      ContentView (window shell), MusicGridView
  Views/Album/        AlbumCarouselView, CollectionHeaderView
  Views/Player/       PlayerBarView
  Views/Play/         PlayView (now playing)
  Views/Shared/       CurrentCoverView, SliderTrack
  Views/Background/   AnimatedBackgroundView
  Rendering/          ArtworkBackdropRenderer (Metal, no SwiftUI)
  Support/            ThumbnailStore, TimeFormat, PlaybackIcons, FolderPicker
```

`State/AppState` is a coordinator, not a god object. It owns the engine, turns
`FluyerEvent`s into updates, and runs the cross-cutting full refresh; everything else is
delegated:

| Type | Owns |
| :--- | :--- |
| `PlaybackState` | Player-bar/play-view snapshots and every transport command |
| `PlaybackClock` | Position sampled ~4×/s, plus the active lyric cursor |
| `LibraryState` | Scanned tracks/albums and scan progress |
| `AlbumSelection` | Which album the grid shows, and album-scoped playback commands |
| `ToastState` | Transient messages, with a cancellable dismissal task |

Position lives in `PlaybackClock` rather than on the player-bar snapshot so a tick
invalidates only the widgets that read position, not every view bound to the bar.

### 4. Bundle

```bash
./scripts/bundle_macos_app.sh              # debug   -> dist/Fluyer.app
./scripts/bundle_macos_app.sh --release    # release -> dist/Fluyer.app
./scripts/bundle_macos_app.sh --release --dmg  # + dist/Fluyer.dmg
```

This does the whole chain: builds the core for the chosen profile, regenerates bindings,
builds the Swift executable, copies `libfluyer_core` + BASS into `Contents/Frameworks`,
adds an rpath, generates `Info.plist`, and ad-hoc signs. Release builds are what to
measure — debug Swift carries far larger unoptimized images.

## Debug

### VS Code

`.vscode/launch.json` has four configurations (Debug/Release × `ui/macos`). Open the
folder, then pick a configuration and hit F5. Rust breakpoints resolve by source location,
so click the gutter next to the line in `crates/fluyer_core` as usual — the app links the
core debug dylib, which carries the DWARF.

### Tests

```bash
# Unit tests — DYLD_LIBRARY_PATH is required for the BASS dylib at load time
DYLD_LIBRARY_PATH="$PWD/libs/macos" cargo test --lib --manifest-path crates/fluyer_core/Cargo.toml

# macOS app — pure logic only, no engine or audio device needed
swift test --package-path ui/macos

# Lints
cargo clippy -p fluyer_core
cargo fmt --all -- --check
```

### Logs

Core logging uses the `flog!` / `flog_err!` macros (`crates/fluyer_core/src/logger.rs`)
and writes **colored output to stderr**, not a file or `RUST_LOG`. To see it, run the
binary from a terminal:

```bash
./ui/macos/.build/debug/FluyerApp 2>&1 | tee /tmp/fluyer.log
```

Launching the bundle instead hides it — `open dist/Fluyer.app` discards stderr. For panic
backtraces set `RUST_BACKTRACE=1`.

### Native debugger

Rust lives in `libfluyer_core.dylib`, so break on **source location, not symbol name** —
`breakpoint set -n fluyer_core` never resolves, because the Rust symbols are mangled.
Attach to the running app:

```bash
open dist/Fluyer.app
lldb -p "$(pgrep FluyerApp)"
(lldb) breakpoint set --file fluyer_core/src/lib.rs --line <line>
(lldb) continue
```

Do not script lldb with `-b` (batch) around a bare `run`: a GUI app never exits, so the
batch script hangs until interrupted. `bt`, `thread backtrace`, and `expr` all work as
usual.

### Memory

Covers and lyrics dominate the footprint, so measure after the library has loaded:

```bash
open dist/Fluyer.app
sleep 4
vmmap -summary "$(pgrep FluyerApp)" | grep -i footprint
```

Reference numbers on macOS 15 (ARM64, 1440×875 window): wxWidgets ≈ 75 MB, SwiftUI
release ≈ 71 MB steady state. `Physical footprint (peak)` is much higher right after a
library scan while covers decode — judge steady state, not the peak.

Image handling is worth watching after any change to the thumbnail path: covers are
downscaled and re-encoded in Rust, cached in a bounded LRU (`ThumbnailCache`), and decoded
into a bounded `NSCache` on the Swift side. Synchronous image decoding inside a SwiftUI
`body` is what previously caused scroll lag, so new image loads belong in a
`.task(id:)` rather than in the view body.

## Troubleshooting

**`PCH was compiled with module cache path .../apps/macos-swiftui/...`** — the package
directory moved but the build cache came with it. Clear the stale cache and rebuild:

```bash
find ui/macos/.build -type d -name ModuleCache -exec rm -rf {} +
```

**Test binary aborts with a missing BASS symbol** — `DYLD_LIBRARY_PATH` is not set. Every
`cargo test` invocation needs it.

**Blank or stale UI after changing a `#[uniffi::export]` signature** — rerun
`./scripts/generate_swift_bindings.sh`; the checked-in bindings under `bindings/swift/`
are generated output and will not match until you do.

**`future version of Rust` / `block v0.1.6` warning** — third-party dependency
forward-compat notice, not an error. Builds are unaffected.
