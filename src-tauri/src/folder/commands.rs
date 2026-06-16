use crate::database::database::GLOBAL_DATABASE;
use crate::folder::{database, scanner, types::FolderItem};

#[tauri::command]
pub fn folder_items_get(path: String) -> Vec<FolderItem> {
    scanner::get_folder_items(path.as_str())
}

#[tauri::command]
pub fn folder_first_music_path_get(path: String) -> Option<String> {
    let mut conn_guard = GLOBAL_DATABASE.lock().ok()?;
    let conn = conn_guard.as_mut()?;
    database::get_folder_first_music_path(conn, path.as_str())
}
