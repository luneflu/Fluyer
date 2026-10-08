# Project Map

## Purpose
Desktop music player. Rust core (`fluyer_core`) manages playback, library scanning, SQLite storage, and view models. Native UIs (macOS SwiftUI, Windows WinUI 3) render view models and forward events via UniFFI and C ABI.

## Requirements
- Scan local audio files, extract metadata and cover art, store in local SQLite database.
- Audio playback via BASS audio engine library.
- Native desktop shell frontends consume view models from Rust core.

## Components
- `crates/fluyer_core`: Audio engine (`audio`), database (`db`), library scanner (`library`), metadata extractor (`metadata`), services (`services`), view models (`view_models`), UniFFI export (`uniffi_api.rs`).
- `bindings/swift`: UniFFI-generated Swift bindings (`FluyerCore`).
- `ui/macos`: macOS native SwiftUI app (`FluyerApp`).
- `ui/windows`: Windows WinUI 3 native app (`Fluyer`).
- `libs/macos`, `libs/windows`: Prebuilt BASS dynamic libraries.

## Main Flow
```
[Music Files] ---> [Scanner / Metadata] ---> [SQLite DB]
                                                    |
[UI Shell (macOS / Windows)] <--- [UniFFI / C ABI] <--- [fluyer_core Engine]
             |                                              |
      [User Actions] ------------------------------> (Commands / Events)
                                                            |
                                                    [BASS Audio Output]
```

## Data and Trust Boundaries
- Database: Local SQLite file (`~/Library/Application Support/org.alvindimas05.fluyer/fluyer.db`).
- Cache: Local filesystem for covers and lyrics (`~/Library/Caches/org.alvindimas05.fluyer/`).
- External: Discord Rich Presence (`discord-rich-presence`), optional HTTP requests (`reqwest`).

## Build and Deployment
- Core: `cargo build -p fluyer_core`
- Swift bindings: `./scripts/generate_swift_bindings.sh`
- macOS app: `swift build --package-path ui/macos`
- Bundling: `scripts/bundle_macos_app.sh`, `scripts/bundle_windows_app.ps1`

## Unknowns
- Specific component focus (Core, macOS UI, Windows UI, Audio, DB).
- Current feature / bugfix goals.
