//! Manual probe for the YouTube Music provider. No engine, no BASS, no database: it
//! exercises `services::ytmusic` directly so the core can be checked before any UI.
//!
//! ```text
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json search "dabin" [--region US] [--lang en] [--data-dir DIR]
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json album MPREb_xxx
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json lyrics <videoId>
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json stream <videoId>
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json login          # device code, then poll
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json player         # prepare the cipher solver
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json solve <n>      # solve one throttle value
//! cargo run -p fluyer_core --example ytmusic_probe -- \
//!   --profile /tmp/ytm-profile.json parse <file> search|album|lyrics|stream
//! ```
//!
//! `--profile` is a `services/profile.rs` payload: `version`, `clientVersion`,
//! `apiKey`, optionally `endpoints` and `authClientId`/`authClientSecret`. Copy
//! `crates/fluyer_core/assets/profile.json` and fill it in (version must be > 0).

use fluyer_core::services::profile::Profile;
use fluyer_core::services::ytmusic::{parsers, YouTubeMusic};
use fluyer_core::services::{SecretStore, Secrets};
use std::sync::Arc;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `play` drives the engine, which owns its own tokio runtime: it must not run
    // inside one, so it gets a plain thread instead of this `block_on`.
    if args.iter().any(|arg| arg == "play") && !args.iter().any(|arg| arg == "parse") {
        return play(&args);
    }
    tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(run(args))
}

/// Plays through BASS: engine (device + mixer) instead of the bare provider.
fn play(args: &[String]) -> Result<(), String> {
    let profile_path = flag(args, "--profile").ok_or_else(usage)?;
    let data_dir = flag(args, "--data-dir")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("fluyer-ytmusic-play"));
    let rest = strip_flags(args);
    let video_id = rest.get(1).ok_or_else(usage)?.clone();
    let title = rest.get(2).cloned().unwrap_or_default();
    let artist = rest.get(3).cloned().unwrap_or_default();
    let seconds: u64 = flag(args, "--seconds").and_then(|v| v.parse().ok()).unwrap_or(8);
    let thumbnail = flag(args, "--thumbnail");

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    std::fs::copy(&profile_path, data_dir.join("profile.json")).map_err(|e| e.to_string())?;
    let engine = fluyer_core::FluyerEngine::new(&data_dir, &data_dir.join("cache"), None)?;
    // `--url <url>` skips the provider: it plays exactly that URL, which is how BASS's
    // own network support is checked (local HTTP server, one googlevideo URL, …).
    if let Some(url) = flag(args, "--url") {
        let metadata = fluyer_core::metadata::MusicMetadata {
            path: url,
            title: Some("probe".into()),
            artist: Some("probe".into()),
            ..Default::default()
        };
        engine.player.add_track_no_auto_play(vec![metadata]);
        engine.player.goto_track(0);
    } else {
        engine.provider_play_track_blocking(video_id.clone(), title, artist, String::new(), thumbnail, 0)?;
    }
    println!("playing {video_id} for {seconds}s (data dir {})", data_dir.display());
    if let Some(seek) = flag(args, "--seek").and_then(|v| v.parse::<u64>().ok()) {
        std::thread::sleep(std::time::Duration::from_secs(2));
        engine.player_seek(seek * 1000);
        println!("seeked to {seek}s");
    }
    println!(
        "queue cover: {} bytes",
        engine.artwork_queue_thumbnail(0, 64).map(|b| b.len()).unwrap_or(0)
    );
    for _ in 0..seconds {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let bar = engine.player_get_bar();
        println!("  {} / {}", bar.position_ms, bar.duration_ms);
    }
    Ok(())
}

async fn run(args: Vec<String>) -> Result<(), String> {
    let Some(profile_path) = flag(&args, "--profile") else {
        return Err(usage());
    };
    let profile = load_profile(&profile_path)?;

    let rest: Vec<String> = strip_flags(&args);
    let Some(command) = rest.first().map(String::as_str) else { return Err(usage()) };
    if command == "parse" {
        return parse_offline(&rest);
    }

    let data_dir = flag(&args, "--data-dir")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("fluyer-ytmusic-probe"));
    let music = YouTubeMusic::new(profile, Arc::new(Secrets::memory()) as Arc<dyn SecretStore>, &data_dir)
        .with_locale(
            flag(&args, "--lang").as_deref().unwrap_or("en"),
            flag(&args, "--region").as_deref().unwrap_or("US"),
        );

    match command {
        "search" => {
            let query = rest.get(1).ok_or_else(usage)?;
            let items = music.search(query).await?;
            println!("{}", serde_json::to_string_pretty(&items).unwrap_or_default());
        }
        "home" => {
            let shelves = music.home().await?;
            for shelf in &shelves {
                println!("== {} ({} items)", shelf.title, shelf.items.len());
                for item in shelf.items.iter().take(5) {
                    println!("  [{}] {} | {}", item.kind as u8, item.title, item.subtitle.clone().unwrap_or_default());
                }
            }
        }
        "album" => {
            let browse_id = rest.get(1).ok_or_else(usage)?;
            let album = music.album(browse_id).await?;
            println!("{}", serde_json::to_string_pretty(&album).unwrap_or_default());
        }
        "lyrics" => {
            let video_id = rest.get(1).ok_or_else(usage)?;
            println!("{}", music.lyrics(video_id).await?);
        }
        "stream" => {
            let video_id = rest.get(1).ok_or_else(usage)?;
            let info = music.stream_formats(video_id).await?;
            println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
        }
        "player" => {
            let (id, sts) = music.cipher_warm().await?;
            println!("player {id} signatureTimestamp {sts}");
        }
        "solve" => {
            let n = rest.get(1).ok_or_else(usage)?;
            let repeat: u32 = flag(&args, "--repeat").and_then(|v| v.parse().ok()).unwrap_or(1);
            let mut total = std::time::Duration::ZERO;
            let mut last = String::new();
            for index in 0..repeat {
                let started = std::time::Instant::now();
                last = music.cipher_solve_n(n).await?;
                let elapsed = started.elapsed();
                total += elapsed;
                if repeat > 1 {
                    println!("#{index}: {elapsed:?}");
                }
            }
            println!("{last}");
            if repeat > 1 {
                println!("{repeat} solves: {:?} total, {:?} each", total, total / repeat);
            }
        }
        "login" => {
            let login = music.begin_login().await?;
            println!("open {} and enter {}", login.verification_url, login.user_code);
            for attempt in 1..=login.expires_in_seconds / login.interval_seconds.max(1) {
                tokio::time::sleep(std::time::Duration::from_secs(login.interval_seconds.max(1))).await;
                match music.poll_login(&login.device_code).await? {
                    fluyer_core::services::ytmusic::TokenStatus::Pending => {
                        println!("attempt {attempt}: pending");
                    }
                    status => {
                        println!("{status:?}");
                        return Ok(());
                    }
                }
            }
            return Err("device code expired".to_string());
        }
        other => return Err(format!("unknown command {other}\n{}", usage())),
    }
    Ok(())
}

fn load_profile(path: &str) -> Result<Profile, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    Profile::parse(&bytes).ok_or_else(|| {
        format!("{path} is not a usable profile (needs version > 0 and clientVersion)")
    })
}

/// Parses a saved response body offline: no network, no profile — the fastest way to
/// check a parser against a real payload without a UI.
fn parse_offline(rest: &[String]) -> Result<(), String> {
    let file = rest.get(1).ok_or_else(usage)?;
    let kind = rest.get(2).map(String::as_str).unwrap_or("search");
    let bytes = std::fs::read(file).map_err(|e| format!("cannot read {file}: {e}"))?;
    let body: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| format!("not JSON: {e}"))?;
    let out = match kind {
        "search" => serde_json::to_value(parsers::parse_search(&body)),
        "home" => serde_json::to_value(parsers::parse_home(&body)),
        "album" => serde_json::to_value(parsers::parse_album(&body, "offline")),
        "lyrics" => Ok(serde_json::to_value(parsers::parse_lyrics(&body)).unwrap()),
        "stream" => serde_json::to_value(parsers::parse_streaming(&body)),
        other => return Err(format!("unknown parse target {other}")),
    };
    println!("{}", serde_json::to_string_pretty(&out.map_err(|e| e.to_string())?).unwrap_or_default());
    Ok(())
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn strip_flags(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg.starts_with("--") {
            skip = true;
            continue;
        }
        out.push(arg.clone());
    }
    out
}

fn usage() -> String {
    "usage: ytmusic_probe --profile <profile.json> [--lang en] [--region US] \
     home | search <query> | album <browseId> | lyrics <videoId> | stream <videoId> | player | solve <n> | login | \
     parse <file.json> [home|search|album|lyrics|stream]"
        .to_string()
}
