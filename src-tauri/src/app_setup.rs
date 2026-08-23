use crate::state::{initialize_store, set_main_window};
use crate::tauri_types::{App, TauriPlugin, WebviewWindow};
use tauri::Manager;

#[cfg(target_os = "macos")]
use crate::platform::{TRAFFIC_LIGHTS_INSET_X, TRAFFIC_LIGHTS_INSET_Y};
#[cfg(target_os = "macos")]
use tauri_plugin_decorum::WebviewWindowExt;

pub fn setup_application(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let main_window = app
        .get_webview_window("main")
        .expect("Main window not found");

    configure_window(&main_window);

    set_main_window(main_window);

    initialize_store(app);

    #[cfg(target_os = "android")]
    crate::developer::logcat::start_logcat_listener();

    // Native renderers (renderer.rs, wgpu_renderer.rs, linux_renderer.rs) are disabled.
    // Background is rendered entirely by the frontend canvas. Code kept for reference.
    // #[cfg(not(feature = "cef"))]
    // crate::renderer::init_global_renderer(app);
    //
    // #[cfg(not(target_os = "linux"))]
    // { let _ = crate::wgpu_renderer::setup_wgpu(app); }
    //
    // #[cfg(all(target_os = "linux", not(feature = "cef")))]
    // { let _ = crate::linux_renderer::setup_linux_background(app); }

    Ok(())
}

fn configure_window(window: &WebviewWindow) {
    let size = if let Ok(Some(monitor)) = window.current_monitor() {
        monitor.size().to_logical(monitor.scale_factor())
    } else {
        tauri::LogicalSize::new(0, 0)
    };

    window.set_size(size).unwrap();

    #[cfg(any(windows, all(target_os = "linux", not(feature = "cef"))))]
    {
        window.set_decorations(false).unwrap();
        window.set_shadow(false).unwrap();
    }
    #[cfg(all(target_os = "linux", feature = "cef"))]
    {
        window.set_title("").unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        window.make_transparent().unwrap();
        window
            .set_traffic_lights_inset(TRAFFIC_LIGHTS_INSET_X, TRAFFIC_LIGHTS_INSET_Y)
            .unwrap();
    }
}

#[cfg(debug_assertions)]
pub fn prevent_default_plugin() -> TauriPlugin {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::debug())
        .build()
}

#[cfg(not(debug_assertions))]
pub fn prevent_default_plugin() -> TauriPlugin {
    tauri_plugin_prevent_default::init()
}

#[cfg(desktop)]
pub fn single_instance_plugin() -> TauriPlugin {
    tauri_plugin_single_instance::init(|app, _args, _cwd| {
        let _ = app
            .get_webview_window("main")
            .expect("no main window")
            .set_focus();
    })
}
