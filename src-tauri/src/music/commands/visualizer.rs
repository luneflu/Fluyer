use tauri::path::BaseDirectory;
use tauri::Manager;
use crate::tauri_types::AppHandle;

#[cfg(target_os = "android")]
use tauri_plugin_fluyer::FluyerExt;

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// This command converts the audio file to a standardized format (mono, 44100Hz, 192kbps MP3)
/// suitable for visualizer processing. The conversion is done using FFmpeg.
#[tauri::command]
pub async fn music_visualizer_buffer_get(app_handle: AppHandle, path: String) -> Option<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        #[cfg(desktop)]
        let dir = std::env::temp_dir();

        #[cfg(mobile)]
        let dir = app_handle.path().app_cache_dir().ok().unwrap();

        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let tmp_file: PathBuf = dir.join(format!("converted_{}.mp3", unique_id));

        #[cfg(desktop)]
        let music_path = &path;
        #[cfg(target_os = "android")]
        let music_path = format!("\"{}\"", &path);

        let mut args = vec![];
        #[cfg(desktop)]
        args.push("-y"); // overwrite
        args.extend([
            "-i",
            music_path.as_str(), // input
            "-ac",
            "1", // mono
            "-ar",
            "44100", // fixed sample rate
            "-b:a",
            "192k", // fixed bitrate
            "-q:a",
            "5", // ~45-55 kbps
            // "-c:a", "libmp3lame", // mp3 encoder
            "-map",
            "0:a", // only audio
            tmp_file.to_str().unwrap(),
        ]);

        #[cfg(mobile)]
        {
            let result = app_handle.fluyer().visualizer_get_buffer(args.join(" "));
            if result.is_err() {
                println!("Failed to convert audio to mp3: {}", result.unwrap_err());
                return std::fs::read(path).ok();
            }

            if !result.unwrap().value {
                println!("Failed to convert audio to mp3");
                return std::fs::read(path).ok();
            }

            return std::fs::read(tmp_file).ok();
        }
        #[cfg(desktop)]
        {
            // Get the bundled ffmpeg path
            let ffmpeg_path = if cfg!(target_os = "linux") && !cfg!(debug_assertions) {
                PathBuf::from("/usr/lib/fluyer/ffmpeg")
            } else if cfg!(target_os = "windows") {
                app_handle
                    .path()
                    .resolve("libs/ffmpeg/bin/ffmpeg.exe", BaseDirectory::Resource)
                    .unwrap()
            } else {
                app_handle
                    .path()
                    .resolve("libs/ffmpeg/bin/ffmpeg", BaseDirectory::Resource)
                    .unwrap()
            };

            let ffmpeg_status = {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    Command::new(&ffmpeg_path)
                        .args(&args)
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW on Windows
                        .status()
                }
                #[cfg(not(windows))]
                {
                    Command::new(&ffmpeg_path).args(&args).status()
                }
            };

            if ffmpeg_status.is_err() {
                crate::error!(
                    "Failed to convert audio to mp3: {}",
                    ffmpeg_status.unwrap_err()
                );
                return std::fs::read(path).ok();
            }

            if !tmp_file.exists() {
                crate::error!("Failed to convert audio to mp3");
                return std::fs::read(path).ok();
            }

            let data = std::fs::read(&tmp_file).ok();

            let _ = std::fs::remove_file(tmp_file);

            return data;
        }
    })
    .await
    .ok()
    .flatten()
}
