use crate::tauri_types::AppHandle;

#[cfg(desktop)]
use tauri::Emitter;
#[cfg(desktop)]
use tauri_plugin_dialog::DialogExt;

use crate::state::{app_handle, app_store};

pub static MUSIC_STORE_PATH_NAME: &str = "music-path";

#[cfg(desktop)]
#[tauri::command]
pub fn music_directory_request(app: AppHandle) {
    app.dialog().file().pick_folder(|dir_path| {
        let dir = dir_path
            .unwrap()
            .into_path()
            .expect("Failed to get music dir path.")
            .into_os_string()
            .into_string()
            .expect("Failed to get music dir path.");

        app_store().set(MUSIC_STORE_PATH_NAME, dir);

        app_handle()
            .emit(crate::commands::route::MUSIC_DIRECTORY_REQUEST, ())
            .unwrap_or_else(|_| {
                crate::error!(
                    "Failed to emit {}",
                    crate::commands::route::MUSIC_DIRECTORY_REQUEST
                )
            })
    });
}
