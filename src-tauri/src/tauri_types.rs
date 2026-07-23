#[cfg(all(target_os = "linux", feature = "cef"))]
pub type Runtime = tauri::Cef;

#[cfg(any(not(target_os = "linux"), not(feature = "cef")))]
pub type Runtime = tauri::Wry;

// Generic Tauri Type Aliases
pub type App = tauri::App<Runtime>;
pub type AppHandle = tauri::AppHandle<Runtime>;
pub type WebviewWindow = tauri::WebviewWindow<Runtime>;
pub type Window = tauri::Window<Runtime>;
pub type TauriPlugin = tauri::plugin::TauriPlugin<Runtime>;
pub type Invoke = tauri::ipc::Invoke<Runtime>;
