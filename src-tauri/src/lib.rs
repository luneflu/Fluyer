// Core modules
pub mod animated_background;
mod api;
pub(crate) mod commands;
mod coverart;
mod database;
mod folder;
pub mod library;
#[cfg(all(target_os = "linux", not(feature = "cef")))]
mod linux_renderer;
pub mod logger;
mod lyric;
mod music;
mod playlist;
pub mod renderer;
pub mod screenshot;
mod sidebar;
mod system;
mod utils;
#[cfg(not(target_os = "linux"))]
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
    let builder = tauri::Builder::<tauri_types::Runtime>::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fluyer::init())
        .plugin(tauri_plugin_device_info::init())
        .plugin(app_setup::prevent_default_plugin());

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(app_setup::single_instance_plugin());

    builder
        .setup(app_setup::setup_application)
        .invoke_handler(commands::COMMAND_HANDLERS)
        .on_window_event(events::handle_window_events)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(events::handle_app_events);
}
