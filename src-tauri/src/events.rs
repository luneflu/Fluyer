use crate::tauri_types::{AppHandle, Window};
use tauri::{RunEvent, WindowEvent};

#[cfg(target_os = "macos")]
use crate::platform::{TRAFFIC_LIGHTS_INSET_X, TRAFFIC_LIGHTS_INSET_Y};
#[cfg(target_os = "macos")]
use crate::state::main_window;
#[cfg(target_os = "macos")]
use tauri_plugin_decorum::WebviewWindowExt;

pub fn handle_window_events(_window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Resized(_) => {
            #[cfg(target_os = "macos")]
            {
                main_window()
                    .set_traffic_lights_inset(TRAFFIC_LIGHTS_INSET_X, TRAFFIC_LIGHTS_INSET_Y)
                    .unwrap();
            }
        }
        WindowEvent::Focused(_focused) =>
        {
            // wgpu_renderer suspend/resume disabled — background rendered by frontend canvas.
            // #[cfg(not(target_os = "linux"))]
            // if *focused { crate::wgpu_renderer::resume_wgpu(...) } else { suspend_wgpu(...) }
        }
        _ => {}
    }
}

pub fn handle_app_events(app_handle: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::Ready => {
            crate::state::initialize_on_ready(app_handle);

            let scale_factor = crate::state::main_window().scale_factor().unwrap_or(1.0);
            crate::music::image_cache::ImageCache::init_base_cover_size(scale_factor);

            #[cfg(desktop)]
            {
                crate::info!("Initializing desktop media controls");
                crate::music::media_session::MediaSession::init();
            }

            #[cfg(target_os = "linux")]
            let _ = crate::sidebar::linux_listen_mouse_leave();
            // wgpu_renderer::start_render_loop disabled — background rendered by frontend canvas.
            // #[cfg(not(target_os = "linux"))]
            // crate::wgpu_renderer::start_render_loop(app_handle.clone());
        }
        RunEvent::WindowEvent {
            label: _,
            event: tauri::WindowEvent::Resized(_size),
            ..
        } => {
            // wgpu_renderer::handle_wgpu_resize disabled — frontend canvas handles resize.
            // #[cfg(not(target_os = "linux"))]
            // crate::wgpu_renderer::handle_wgpu_resize(app_handle, size.width, size.height);
        }
        RunEvent::Resumed => {
            // wgpu_renderer::resume_wgpu disabled — frontend canvas handles resume.
            // #[cfg(not(target_os = "linux"))]
            // crate::wgpu_renderer::resume_wgpu(app_handle);
        }
        _ => {}
    }
}
