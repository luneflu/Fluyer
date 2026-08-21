//! Desktop OS media controls via souvlaki (macOS Now Playing, Windows SMTC, Linux MPRIS).

use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// One MediaControls instance per process.
static CONTROLS: OnceLock<Mutex<Option<MediaControls>>> = OnceLock::new();

fn controls() -> &'static Mutex<Option<MediaControls>> {
    CONTROLS.get_or_init(|| Mutex::new(None))
}

/// Initialise souvlaki and attach the event handler.
/// Must be called once after the main window exists (RunEvent::Ready).
pub fn init<F>(handler: F)
where
    F: Fn(MediaControlAction) + Send + Sync + 'static,
{
    #[cfg(target_os = "windows")]
    let hwnd = {
        use crate::state::main_window;
        main_window()
            .hwnd()
            .ok()
            .map(|h| h.0 as *mut std::ffi::c_void)
    };
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    let config = PlatformConfig {
        dbus_name: "org.mpris.MediaPlayer2.fluyer",
        display_name: "Fluyer",
        hwnd,
    };

    let mut mc = match MediaControls::new(config) {
        Ok(c) => c,
        Err(e) => {
            crate::error!("Failed to create MediaControls: {:?}", e);
            return;
        }
    };

    // Register as a player immediately so the OS picks it up.
    let _ = mc.set_playback(MediaPlayback::Stopped);

    if let Err(e) = mc.attach(move |event| {
        if let Some(action) = map_event(event) {
            handler(action);
        }
    }) {
        crate::error!("Failed to attach handler: {:?}", e);
    }

    if let Ok(mut guard) = controls().lock() {
        *guard = Some(mc);
    }
}

/// Update the Now Playing metadata.
pub fn update_metadata(
    title: &str,
    artist: &str,
    album: &str,
    duration_ms: u64,
    cover_path: Option<&str>,
) {
    // souvlaki cover_url: use file:// URI on all platforms.
    let cover_url: Option<String> = cover_path.map(|p| {
        #[cfg(target_os = "linux")]
        let p = {
            // If we have a cover path on Linux, create a uniquely named symlink or copy
            // to bypass MPRIS clients aggressive caching.
            use std::fs;
            use std::path::Path;
            use std::time::{SystemTime, UNIX_EPOCH};

            let path = Path::new(p);
            if path.exists() {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
                let new_path = std::env::temp_dir().join(format!("fluyer_cover_{}.{}", ts, ext));

                // Try to copy, ignore errors (fallback to original path)
                if fs::copy(path, &new_path).is_ok() {
                    new_path.to_string_lossy().to_string()
                } else {
                    p.to_string()
                }
            } else {
                p.to_string()
            }
        };

        #[cfg(not(target_os = "linux"))]
        let p = p.to_string();

        if p.starts_with("file://") {
            p
        } else {
            format!("file://{}", p)
        }
    });

    let duration = if duration_ms > 0 {
        Some(Duration::from_millis(duration_ms))
    } else {
        None
    };

    if let Ok(mut guard) = controls().lock() {
        if let Some(mc) = guard.as_mut() {
            let _ = mc.set_metadata(MediaMetadata {
                title: Some(title),
                artist: Some(artist),
                album: Some(album),
                cover_url: cover_url.as_deref(),
                duration,
            });
        }
    }
}

/// Update the playback state (playing / paused) and current position.
pub fn set_playback_state(is_playing: bool, position_ms: u64) {
    let progress = Some(MediaPosition(Duration::from_millis(position_ms)));
    let playback = if is_playing {
        MediaPlayback::Playing { progress }
    } else {
        MediaPlayback::Paused { progress }
    };

    if let Ok(mut guard) = controls().lock() {
        if let Some(mc) = guard.as_mut() {
            let _ = mc.set_playback(playback);
        }
    }
}

/// Strongly-typed media control actions dispatched from the OS.
#[derive(Debug, Clone)]
pub enum MediaControlAction {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,
    /// Seek to absolute position in milliseconds.
    SeekTo(u64),
    /// Seek forward by milliseconds.
    SeekForward(u64),
    /// Seek backward by milliseconds.
    SeekBackward(u64),
    Raise,
    Quit,
}

fn map_event(event: souvlaki::MediaControlEvent) -> Option<MediaControlAction> {
    use souvlaki::{MediaControlEvent as E, SeekDirection};

    match event {
        E::Play => Some(MediaControlAction::Play),
        E::Pause => Some(MediaControlAction::Pause),
        E::Toggle => Some(MediaControlAction::Toggle),
        E::Next => Some(MediaControlAction::Next),
        E::Previous => Some(MediaControlAction::Previous),
        E::Stop => Some(MediaControlAction::Stop),
        E::SetPosition(MediaPosition(dur)) => {
            Some(MediaControlAction::SeekTo(dur.as_millis() as u64))
        }
        E::SeekBy(SeekDirection::Forward, dur) => {
            Some(MediaControlAction::SeekForward(dur.as_millis() as u64))
        }
        E::SeekBy(SeekDirection::Backward, dur) => {
            Some(MediaControlAction::SeekBackward(dur.as_millis() as u64))
        }
        E::Seek(SeekDirection::Forward) => Some(MediaControlAction::SeekForward(10_000)),
        E::Seek(SeekDirection::Backward) => Some(MediaControlAction::SeekBackward(10_000)),
        E::Raise => Some(MediaControlAction::Raise),
        E::Quit => Some(MediaControlAction::Quit),
        // SetVolume / OpenUri — not handled.
        _ => None,
    }
}
