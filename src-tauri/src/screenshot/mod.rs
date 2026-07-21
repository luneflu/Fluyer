use base64::{engine::general_purpose, Engine as _};
use tauri::Manager;
use std::fs::File;
use std::io::Write;
use crate::tauri_types::{AppHandle};

#[tauri::command]
pub async fn screenshot_save(app: AppHandle, base64_data: String) -> Result<(), String> {
    // Decode base64 string
    let data = general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Get home directory using Tauri's path API
    let home_dir = app.path().home_dir().map_err(|e| e.to_string())?;
    let file_path = home_dir.join("fluyer_preview.png");

    // Write file
    let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;

    println!("[Screenshot] Saved to {:?}", file_path);

    Ok(())
}
