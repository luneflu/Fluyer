// src-tauri/src/tauri_types.rs
#[cfg(feature = "cef")]
pub type Runtime = tauri::Cef;

#[cfg(not(feature = "cef"))]
pub type Runtime = tauri::Wry;

// Generic Tauri Type Aliases
pub type App = tauri::App<Runtime>;
pub type AppHandle = tauri::AppHandle<Runtime>;
pub type WebviewWindow = tauri::WebviewWindow<Runtime>;
pub type Window = tauri::Window<Runtime>;
pub type TauriPlugin = tauri::plugin::TauriPlugin<Runtime>;
pub type Invoke = tauri::ipc::Invoke<Runtime>;
