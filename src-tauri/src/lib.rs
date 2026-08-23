// Core modules
pub mod animated_background;
mod api;
pub(crate) mod commands;
mod coverart;
mod database;
mod developer;
mod folder;
pub mod library;
// linux_renderer kept for reference; not called — background rendered by frontend canvas.
#[cfg(all(target_os = "linux", not(feature = "cef")))]
#[allow(dead_code)]
mod linux_renderer;
pub mod logger;
mod lyric;
mod music;
mod playlist;
// renderer kept for reference; not called — background rendered by frontend canvas.
#[cfg(not(feature = "cef"))]
#[allow(dead_code)]
pub mod renderer;
pub mod screenshot;
mod sidebar;
mod utils;
// wgpu_renderer kept for reference; not called — background rendered by frontend canvas.
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
mod wgpu_renderer;

// Application modules
mod app_setup;
mod events;
pub(crate) mod state;

// Re-export platform module from main
pub mod platform;

pub mod tauri_types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix Nvidia issue crash on Linux
    // https://github.com/tauri-apps/tauri/issues/9394
    #[cfg(target_os = "linux")]
    if std::path::Path::new("/proc/driver/nvidia/version").exists()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    let builder = tauri::Builder::<tauri_types::Runtime>::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_device_info::init())
        .plugin(tauri_plugin_fluyer::init())
        .plugin(app_setup::prevent_default_plugin());

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(app_setup::single_instance_plugin());

    builder
        .setup(app_setup::setup_application)
        .invoke_handler(commands::COMMAND_HANDLERS)
        .on_window_event(events::handle_window_events)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(events::handle_app_events);
}
