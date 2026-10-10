# Fluyer

Music player for desktop and Android. Rust core (`fluyer_core`) holds all logic — library scanning,
metadata, playback, and view models — and exposes it over C ABI + UniFFI. UIs are thin
native shells that only render view models and forward events.

## Layout

| Path | What it is |
| :--- | :--- |
| `crates/fluyer_core/` | Core crate: engine, services, view models. Produces `libfluyer_core.dylib` |
| `crates/fluyer_core/src/view_models/` | Per-component view models (player bar, play view, track item, album card, …) |
| `bindings/swift/` | **Generated** UniFFI Swift bindings + SPM package `FluyerCore` |
| `ui/macos/` | macOS SwiftUI app (SPM executable `FluyerApp`) |
| `ui/windows/` | Windows WinUI 3 app (`Fluyer`) + `Fluyer.Core` (P/Invoke/state) + `Fluyer.Tests` (xUnit) |
| `libs/macos/` | Prebuilt `libbass.dylib`, `libbassmix.dylib` |
| `libs/windows/` | Prebuilt `bass.dll`, `bassmix.dll`, plugins (`.dll` + `.lib`) |
| `ui/android/` | Android Jetpack Compose app (Gradle, `org.alvindimas05.fluyer`) |
| `bindings/kotlin/` | **Generated** UniFFI Kotlin bindings (`uniffi.fluyer_core`) |
| `libs/android/arm64-v8a/` | Prebuilt `libbass*.so` (core, mix, flac, opus, ape, wv, aac, alac) |
| `scripts/` | `generate_swift_bindings.sh`, `bundle_macos_app.sh`, `bundle_windows_app.ps1`, `build_android.sh` |
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
`crates/fluyer_core/src/uniffi_api/`. It rebuilds the dylib, runs `uniffi-bindgen`,
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
  App/                FluyerApp (@main), AppDelegate
  Screens/            one folder per screen, then one per feature (state + views)
    Home/             HomeView (window shell), LibraryFilterState
      MusicGrid/      MusicGridView, TrackCard
      Albums/         AlbumCarouselView, AlbumGridView, AlbumCard, AlbumHeaderView, AlbumMetrics
      Queue/          QueueView ("Now Playing"), QueueRow, QueueState, QueueEdgeTrigger
      PlayerBar/      PlayerBarView
      Toolbar/        SortMenu, ToolbarSearchField
    Play/             PlayView; Controls/PlayControlsView; Lyrics/LyricsView
    Settings/         SettingsView
  Shared/State/       state used by several screens (see below)
  Shared/Support/     ThumbnailStore, TimeFormat, PlaybackIcons, FolderPicker, ...
  Components/         views used by several screens: SliderTrack, CurrentCoverView,
                      ToastView, SidebarOcclusion, Backdrop/ (Metal backdrop)
```

See `ARCHITECTURE.md` for the rules.

`Shared/State/AppState` is a coordinator, not a god object. It owns the engine, turns
`FluyerEvent`s into updates, and runs the cross-cutting full refresh; everything else is
delegated:

| Type | Owns |
| :--- | :--- |
| `PlaybackState` | Player-bar/play-view snapshots and every transport command |
| `PlaybackClock` | Position sampled ~4×/s, plus the active lyric cursor |
| `LibraryState` | Scanned tracks/albums and scan progress |
| `LibraryFilterState` | Mode, sort, search, which album the grid shows, and album-scoped playback commands |
| `QueueState` | Queue snapshot and queue commands (`Screens/Home/Queue`) |
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

## Android (Jetpack Compose)

Kotlin talks to the core through UniFFI-generated bindings (`bindings/kotlin/`, JNA at
runtime), the same API surface the Swift shell uses. `ui/android/.../state/` is a 1:1
port of `ui/macos/Sources/Shared/State/` onto Compose snapshot state; `AppState` and the engine
live in `FluyerApplication` so the Activity and `PlaybackService` (foreground
`mediaPlayback` service: MediaSession, notification controls, audio focus, unplug-to-pause)
share one player. The Metal backdrop is ported to OpenGL ES 3 in `ui/android/.../backdrop/`
(same three spinning instances, warp mesh, timing and 0.5s crossfade; blur is a
quarter-res separable gaussian on the macOS sigma), with a static image fallback.

Folders are picked through the system folder picker and mapped to filesystem paths; the
core reads files directly, which `READ_MEDIA_AUDIO` allows for audio. Discord Rich
Presence is disabled on Android.

Runtime state: database + `settings.json` in the app's `filesDir`, cover + lyrics cache
in `cacheDir` (both wiped on uninstall).

### Prerequisites

- Rust with `aarch64-linux-android` (`rustup target add aarch64-linux-android`)
- Android SDK (platform 35) + NDK (`ANDROID_HOME`; `ANDROID_NDK_HOME` or newest
  `$ANDROID_HOME/ndk/*`), JDK 17+
- `libs/android/arm64-v8a/libbass*.so` must be present (gitignored; from the BASS Android
  packages, including `libbass_aac.so` and `libbassalac.so`)

### Build

```bash
./scripts/build_android.sh            # core -> jniLibs + Kotlin bindings (--release for release)
cd ui/android
./gradlew testDebugUnitTest           # JVM tests: state ports + backdrop mesh
./gradlew installDebug                # build + install on the connected device/emulator
adb logcat -s Fluyer                  # core logs (flog!/log::) go to logcat on Android
```

Rerun `build_android.sh` after any core change; Gradle only packages what it staged.
arm64-v8a only for now (all current phones and the arm64 emulator image); add ABIs in
both the script and `abiFilters`.

## Windows (WinUI 3)

UniFFI 0.28 has no C# backend, so the Windows shell talks to the core over the C ABI
(`crates/fluyer_core/src/c_api/`) via P/Invoke in `ui/windows/Fluyer.Core/Native/`.
View models cross as JSON and are deserialized into matching records. The state layer
(`AppState` + `Playback`/`Library`/`Selection`/`Toast`/`Clock`) is a 1:1 port of
`ui/macos/Sources/Shared/State/` onto `INotifyPropertyChanged`; views are XAML ports of
`ui/macos/Sources/Screens/`, in the same `Screens/` / `Components/` / `Shared/` layout
(see `ARCHITECTURE.md`, Windows section). The Metal backdrop is ported to D3D11 in
`ui/windows/Components/Backdrop/` (same spinning instances, warp mesh, timing and 0.5s
crossfade; MPS gaussian becomes a calibrated half-res Kawase chain, and the
final frame is presented via staging readback since WinUI 3's SwapChainPanel
does not expose the UWP native interop contract). The core's pre-blurred
ambient frame remains as the static fallback when Direct3D is unavailable.

Runtime state (created on first launch):

- Database — `%AppData%\org.alvindimas05.fluyer\fluyer.db`
- Cover + lyrics cache — `%LocalAppData%\org.alvindimas05.fluyer\`

### Prerequisites

- Rust (stable) and the .NET 8 SDK
- `libs/windows/*.dll` must be present (they are gitignored; copy them in on a fresh clone)

### Build

```powershell
cargo build -p fluyer_core              # debug -> target/debug/fluyer_core.dll
dotnet test ui/windows/Fluyer.Tests     # xUnit: state/support ports + native smoke test
dotnet run --project ui/windows         # unpackaged, self-contained WinAppSDK
```

The `CopyNativeDeps` target in `Fluyer.csproj` stages `fluyer_core.dll` + BASS next to
the managed exe after every build (the Windows equivalent of the macOS rpath step),
so no `PATH` tweaks are needed. First run needs the Windows 10 SDK (10.0.22621+) only
at build time.

### Bundle

```powershell
./scripts/bundle_windows_app.ps1              # debug   -> dist/Fluyer-windows-x64.zip
./scripts/bundle_windows_app.ps1 -Release     # release -> dist/Fluyer-windows-x64.zip
```

Framework-dependent: the zip needs the .NET 8 desktop runtime on the target machine.

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
`./scripts/generate_swift_bindings.sh` (and `./scripts/build_android.sh` for Android); the
checked-in bindings under `bindings/swift/` and `bindings/kotlin/` are generated output and
will not match until you do.

**Kotlin bindings fail with `'message' hides member of supertype 'Throwable'`** — a
`uniffi::Error` variant has a field named `message`. Kotlin maps errors to exceptions, so
name it something else (`FluyerError` uses `reason`).

**`future version of Rust` / `block v0.1.6` warning** — third-party dependency
forward-compat notice, not an error. Builds are unaffected.
