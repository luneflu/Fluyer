use crate::state::{initialize_store, set_main_window};
use tauri::Manager;
use crate::tauri_types::{App, WebviewWindow, TauriPlugin};

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

    crate::renderer::init_global_renderer(app);

    #[cfg(not(target_os = "linux"))]
    {
        crate::debug!("setup_application: About to call setup_wgpu");
        match crate::wgpu_renderer::setup_wgpu(app) {
            Ok(_) => crate::debug!("setup_application: WGPU initialized successfully"),
            Err(e) => crate::error!("setup_application: WGPU initialization failed: {:?}", e),
        }
    }

    #[cfg(target_os = "linux")]
    {
        crate::debug!("setup_application: About to call setup_linux_background");
        match crate::linux_renderer::setup_linux_background(app) {
            Ok(_) => crate::debug!("setup_application: GTK OpenGL initialized successfully"),
            Err(e) => crate::error!(
                "setup_application: GTK OpenGL initialization failed: {:?}",
                e
            ),
        }
    }

    Ok(())
}

fn configure_window(window: &WebviewWindow) {
    #[cfg(any(windows, target_os = "linux"))]
    {
        window.set_decorations(false).unwrap();
        window.set_shadow(false).unwrap();
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
