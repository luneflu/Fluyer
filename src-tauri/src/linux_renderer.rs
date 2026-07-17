#![cfg(target_os = "linux")]

use femtovg::{renderer::OpenGl, Canvas, Color, ImageFlags, Paint, Path};
use gtk::prelude::*;
use image::RgbaImage;
use std::sync::{Arc, Mutex};
use tauri::Manager;

use crate::state::app_handle;

pub struct LinuxRendererState {
    canvas: Option<Canvas<OpenGl>>,
}

unsafe impl Send for LinuxRendererState {}
unsafe impl Sync for LinuxRendererState {}

pub struct SharedLinuxRenderer {
    pub state: Mutex<LinuxRendererState>,
}

unsafe fn load_gl_fn(s: &str) -> *const std::ffi::c_void {
    let mut ptr: *const std::ffi::c_void = std::ptr::null();
    let name = std::ffi::CString::new(s).unwrap();
    if let Ok(lib) = libloading::Library::new("libGL.so.1") {
        if let Ok(sym) = lib.get::<unsafe extern "C" fn(*const i8) -> *const std::ffi::c_void>(
            b"glXGetProcAddress\0",
        ) {
            ptr = sym(name.as_ptr());
        }
    }
    if ptr.is_null() {
        if let Ok(lib) = libloading::Library::new("libEGL.so.1") {
            if let Ok(sym) = lib.get::<unsafe extern "C" fn(*const i8) -> *const std::ffi::c_void>(
                b"eglGetProcAddress\0",
            ) {
                ptr = sym(name.as_ptr());
            }
        }
    }
    ptr
}

fn on_realize(gl_area: &gtk::GLArea, shared: Arc<SharedLinuxRenderer>) {
    gl_area.make_current();
    if gl_area.error().is_some() {
        crate::error!("GLArea: context error on realize");
        return;
    }

    let renderer = unsafe { OpenGl::new_from_function(|s| load_gl_fn(s)) };

    match renderer {
        Ok(r) => {
            let mut canvas = Canvas::new(r).expect("Cannot create femtovg canvas");
            let alloc = gl_area.allocation();
            canvas.set_size(alloc.width() as u32, alloc.height() as u32, 1.0);
            let mut state = shared.state.lock().unwrap();
            state.canvas = Some(canvas);
        }
        Err(e) => {
            crate::error!("femtovg OpenGL init failed: {:?}", e);
        }
    }
}

fn on_render(
    _gl_area: &gtk::GLArea,
    _ctx: &gtk::gdk::GLContext,
    shared: Arc<SharedLinuxRenderer>,
) -> gtk::glib::Propagation {
    let mut state = shared.state.lock().unwrap();
    if let Some(canvas) = state.canvas.as_mut() {
        if let Some(global) =
            crate::state::app_handle().try_state::<Arc<crate::renderer::GlobalRenderer>>()
        {
            let mut bg_state = global.bg_state.lock().unwrap();
            crate::renderer::draw_background(canvas, &mut bg_state);
            canvas.flush();
        }
    }
    gtk::glib::Propagation::Proceed
}

fn on_resize(_gl_area: &gtk::GLArea, width: i32, height: i32, shared: Arc<SharedLinuxRenderer>) {
    let mut state = shared.state.lock().unwrap();
    if let Some(canvas) = state.canvas.as_mut() {
        canvas.set_size(width as u32, height as u32, 1.0);
    }
}

fn on_tick(widget: &gtk::GLArea, _clock: &gtk::gdk::FrameClock) -> gtk::glib::ControlFlow {
    let mut redraw = false;
    if let Some(global) =
        crate::state::app_handle().try_state::<Arc<crate::renderer::GlobalRenderer>>()
    {
        let mut bg_state = global.bg_state.lock().unwrap();
        redraw = bg_state.needs_redraw;
        bg_state.needs_redraw = false;

        if bg_state.transition_start.is_some() {
            redraw = true;
        }
    }

    if redraw {
        widget.queue_render();
    }
    gtk::glib::ControlFlow::Continue
}

fn on_unrealize(gl_area: &gtk::GLArea, shared: Arc<SharedLinuxRenderer>) {
    gl_area.make_current();
    let mut state = shared.state.lock().unwrap();
    state.canvas = None;
}

fn inject_gtk_overlay(
    gtk_window: &gtk::ApplicationWindow,
    overlay: &gtk::Overlay,
    gl_area: &gtk::GLArea,
) {
    if let Some(child) = gtk_window.child() {
        gtk_window.remove(&child);
        overlay.add(gl_area);

        if let Ok(container) = child.clone().downcast::<gtk::Container>() {
            for c in container.children() {
                container.remove(&c);
                overlay.add_overlay(&c);
            }
        } else {
            overlay.add_overlay(&child);
        }

        gtk_window.add(overlay);
    } else {
        overlay.add(gl_area);
        gtk_window.add(overlay);
    }
    overlay.show_all();
}

pub fn setup_linux_background(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    crate::debug!("setup_linux_background: Starting GTK femtovg OpenGL initialization");

    let window = app.get_webview_window("main").unwrap();
    let gtk_window = window.gtk_window()?;

    let overlay = gtk::Overlay::new();
    let gl_area = gtk::GLArea::new();
    gl_area.set_has_alpha(true);
    gl_area.set_auto_render(true);

    let shared_renderer = Arc::new(SharedLinuxRenderer {
        state: Mutex::new(LinuxRendererState { canvas: None }),
    });

    let shared = shared_renderer.clone();
    gl_area.connect_realize(move |g| on_realize(g, shared.clone()));

    let shared = shared_renderer.clone();
    gl_area.connect_render(move |g, ctx| on_render(g, ctx, shared.clone()));

    let shared = shared_renderer.clone();
    gl_area.connect_resize(move |g, w, h| on_resize(g, w, h, shared.clone()));

    gl_area.add_tick_callback(on_tick);

    let shared = shared_renderer.clone();
    gl_area.connect_unrealize(move |g| on_unrealize(g, shared.clone()));

    inject_gtk_overlay(&gtk_window, &overlay, &gl_area);

    app.manage(shared_renderer);
    Ok(())
}
