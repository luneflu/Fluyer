use crate::logger;
use crate::state::{app_handle, app_store};
use crate::utils::toast::{Toast, ToastType};
use tauri::Manager;
use tauri_plugin_fluyer::FluyerExt;

#[tauri::command]
pub fn log_error(message: String) {
    crate::error!("{}", message);
}

#[tauri::command]
pub fn log_info(message: String) {
    crate::info!("{}", message);
}

// #[cfg(target_os = "android")]
#[tauri::command]
pub fn toast(message: String) {
    Toast::show(message, ToastType::Info);
}

#[tauri::command]
pub fn developer_log_save() {
    let path = format!(
        "{}/{}",
        app_handle().path().home_dir().unwrap().display(),
        logger::get_log_name()
    );
    std::fs::copy(logger::get_log_path(), path.clone()).unwrap();
    Toast::show(
        format!("Log file saved to {}", path).to_string(),
        ToastType::Info,
    );
}

#[tauri::command]
pub fn developer_mpv_log_save() {
    let path = format!(
        "{}/{}",
        app_handle().path().home_dir().unwrap().display(),
        logger::get_mpv_log_name()
    );
    std::fs::copy(logger::get_mpv_log_path(), path.clone()).unwrap();
    Toast::show(
        format!("Log MPV file saved to {}", path).to_string(),
        ToastType::Info,
    );
}

#[tauri::command]
pub fn developer_clear_data() {
    if let Ok(path) = app_handle().path().app_data_dir() {
        if path.exists() {
            app_store().clear();

            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
        }
    }
    #[cfg(desktop)]
    app_handle().restart();

    #[cfg(target_os = "android")]
    app_handle().fluyer().restart_app();
}

#[tauri::command]
pub fn developer_clear_cache() {
    if let Ok(path) = app_handle().path().app_cache_dir() {
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
        }
    }
    #[cfg(desktop)]
    app_handle().restart();

    #[cfg(target_os = "android")]
    app_handle().fluyer().restart_app();
}

#[derive(serde::Deserialize)]
struct LatestRelease {
    version: String,
}

fn parse_version(v: &str) -> Vec<u32> {
    v.trim_start_matches('v')
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    let curr = parse_version(current);
    let late = parse_version(latest);
    let max_len = curr.len().max(late.len());
    for i in 0..max_len {
        let c = curr.get(i).copied().unwrap_or(0);
        let l = late.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if c > l {
            return false;
        }
    }
    false
}

/// Check for update using reqwest
#[tauri::command]
pub async fn update_check(current_version: String) -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent("fluyer-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://github.com/alvindimas05/Fluyer/releases/latest/download/latest.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let release: LatestRelease = response.json().await.map_err(|e| e.to_string())?;

    if is_newer_version(&current_version, &release.version) {
        Ok(Some(release.version))
    } else {
        Ok(None)
    }
}
