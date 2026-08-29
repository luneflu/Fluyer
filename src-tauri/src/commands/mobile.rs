#[cfg(mobile)]
use crate::state::app_handle;
#[cfg(mobile)]
use tauri::Emitter;
#[cfg(mobile)]
use tauri_plugin_fluyer::FluyerExt;

#[cfg(target_os = "android")]
#[tauri::command]
pub fn mobile_audio_permission_read_check() -> bool {
    use tauri::plugin::PermissionState;

    let permissions_result = app_handle().fluyer().check_permissions().unwrap();
    if permissions_result.is_none() {
        return false;
    }

    let permissions = permissions_result.unwrap();
    permissions.audio == PermissionState::Granted || permissions.storage == PermissionState::Granted
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn mobile_audio_permission_read_request() -> bool {
    use crate::state::app_handle;
    use tauri::plugin::PermissionState;
    use tauri_plugin_fluyer::models::PermissionType;

    if !mobile_audio_permission_read_check() {
        let permissions = app_handle()
            .fluyer()
            .request_permissions(Some(vec![
                if app_handle().fluyer().get_sdk_version().unwrap().value >= 33 {
                    PermissionType::Audio
                } else {
                    PermissionType::Storage
                },
            ]))
            .unwrap();
        return permissions.audio == PermissionState::Granted
            || permissions.storage == PermissionState::Granted;
    }
    true
}

#[cfg(mobile)]
#[tauri::command]
pub fn mobile_navigation_bar_height_get() -> u8 {
    app_handle()
        .fluyer()
        .navigation_bar_height_get()
        .unwrap()
        .value
}

#[cfg(mobile)]
#[tauri::command]
pub fn mobile_status_bar_height_get() -> u8 {
    app_handle().fluyer().status_bar_height_get().unwrap().value
}

#[cfg(mobile)]
#[tauri::command]
pub fn mobile_navigation_bar_visibility_set(visible: bool) {
    app_handle()
        .fluyer()
        .navigation_bar_visibility_set(visible)
        .unwrap();
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn mobile_android_directory_request() {
    use crate::state::app_handle;

    app_handle()
        .fluyer()
        .android_pick_folder(|payload| {
            if let Some(dir) = payload.value {
                app_handle()
                    .emit(
                        crate::commands::route::MOBILE_ANDROID_DIRECTORY_REQUEST,
                        dir,
                    )
                    .unwrap();
            }
        })
        .ok();
}
