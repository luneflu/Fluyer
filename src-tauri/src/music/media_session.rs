#![allow(unused_variables, unused_imports)]

use crate::music::metadata::MusicMetadata;
use crate::state::app_handle;

#[cfg(target_os = "android")]
use tauri_plugin_fluyer::FluyerExt;

pub struct MediaSession;

impl MediaSession {
    pub fn init() {
        #[cfg(target_os = "android")]
        {
            crate::info!("Initializing Android Media Control");
            let _ = app_handle().fluyer().init_media_control(|event| {
                std::thread::spawn(move || {
                    use tauri::Manager;

                    crate::info!("Media Control Action: {}", event.action);
                    let handle: &crate::tauri_types::AppHandle = app_handle();
                    let state = handle.state::<crate::state::AppState>();

                    if event.action == "play" {
                        state.music_player.play();
                    } else if event.action == "pause" {
                        state.music_player.pause();
                    } else if event.action.starts_with("seek:") {
                        if let Ok(pos) = event.action[5..].parse::<u64>() {
                            state.music_player.set_pos(pos);
                        }
                    } else if event.action == "previous" {
                        state.music_player.play_previous();
                    } else if event.action == "next" {
                        state.music_player.play_next(true);
                    } else if event.action == "stop" {
                        state.music_player.pause();
                    } else {
                        crate::warn!("Unknown media session command: {}", event.action);
                    }
                });
            });
        }

        #[cfg(desktop)]
        {
            use crate::music::media_control::MediaControlAction;

            crate::info!("Initializing Desktop Media Control");
            crate::music::media_control::init(|action| {
                std::thread::spawn(move || {
                    use tauri::Manager;

                    crate::info!("Desktop Media Control Action: {:?}", action);
                    let handle: &crate::tauri_types::AppHandle = app_handle();
                    let state = handle.state::<crate::state::AppState>();

                    match action {
                        MediaControlAction::Play => state.music_player.play(),
                        MediaControlAction::Pause => state.music_player.pause(),
                        MediaControlAction::Toggle => {
                            let sync = state.music_player.get_sync_info(false);
                            if sync.is_playing() {
                                state.music_player.pause();
                            } else {
                                state.music_player.play();
                            }
                        }
                        MediaControlAction::Next => state.music_player.play_next(true),
                        MediaControlAction::Previous => state.music_player.play_previous(),
                        MediaControlAction::Stop => state.music_player.pause(),
                        MediaControlAction::SeekTo(ms) => state.music_player.set_pos(ms),
                        MediaControlAction::SeekForward(delta) => {
                            let cur = state.music_player.get_sync_info(false).position_ms();
                            state.music_player.set_pos(cur.saturating_add(delta));
                        }
                        MediaControlAction::SeekBackward(delta) => {
                            let cur = state.music_player.get_sync_info(false).position_ms();
                            state.music_player.set_pos(cur.saturating_sub(delta));
                        }
                        MediaControlAction::Raise => {
                            if let Some(window) = handle.get_webview_window("main") {
                                let _ = window.set_focus();
                            }
                        }
                        MediaControlAction::Quit => handle.exit(0),
                    }
                });
            });
        }
    }

    pub fn set_state(is_playing: bool, position: u64) {
        #[cfg(target_os = "android")]
        {
            let _ = app_handle()
                .fluyer()
                .set_media_control_state(is_playing, position);
        }

        #[cfg(desktop)]
        {
            crate::music::media_control::set_playback_state(is_playing, position);
        }
    }

    pub async fn update_metadata(
        music: &MusicMetadata,
        is_playing: bool,
        is_first: bool,
        is_last: bool,
    ) {
        #[cfg(target_os = "android")]
        {
            let handle = app_handle();
            let image_path = match handle.fluyer().metadata_get_image(music.path.clone()) {
                Ok(res) => res.path,
                Err(_) => None,
            };

            let _ = handle.fluyer().update_media_control(
                music
                    .title
                    .clone()
                    .unwrap_or_else(|| MusicMetadata::default_title().to_string()),
                music
                    .artist
                    .clone()
                    .unwrap_or_else(|| MusicMetadata::default_artist().to_string()),
                music.album.clone().unwrap_or("Unknown".to_string()),
                music.duration.unwrap_or(0) as u64,
                image_path,
                is_playing,
                is_first,
                is_last,
            );
        }

        #[cfg(desktop)]
        {
            let title = music
                .title
                .clone()
                .unwrap_or_else(|| MusicMetadata::default_title().to_string());
            let artist = music
                .artist
                .clone()
                .unwrap_or_else(|| MusicMetadata::default_artist().to_string());
            let album = music.album.clone().unwrap_or_else(|| "Unknown".to_string());
            let duration_ms = music.duration.unwrap_or(0) as u64;

            let cover_path = extract_cover_to_cache(music).await;

            crate::music::media_control::update_metadata(
                &title,
                &artist,
                &album,
                duration_ms,
                cover_path.as_deref(),
            );

            crate::music::media_control::set_playback_state(is_playing, 0);
        }
    }
}

/// Extract cover art (embedded or online via coverart) to a cache file, returning the local path.
#[cfg(desktop)]
async fn extract_cover_to_cache(music: &MusicMetadata) -> Option<String> {
    use std::io::Write;
    use tauri::Manager;

    let bytes = match MusicMetadata::get_image_from_path(music.path.clone()).await {
        Ok(b) => Some(b),
        Err(_) => {
            let query = crate::coverart::types::CoverArtQuery {
                artist: music.artist.clone().unwrap_or_default(),
                album: music.album.clone(),
                title: if music.album.is_some() { None } else { music.title.clone() },
            };
            crate::coverart::commands::cover_art_get(query, None).await
        }
    }?;

    if bytes.is_empty() {
        return None;
    }

    let cache_dir = app_handle().path().app_cache_dir().ok()?;
    let cover_path = cache_dir.join("media_control_cover.jpg");
    let mut f = std::fs::File::create(&cover_path).ok()?;
    f.write_all(&bytes).ok()?;
    cover_path.to_str().map(|s| s.to_string())
}
