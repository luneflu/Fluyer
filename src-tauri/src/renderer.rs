use crate::state::app_handle;
use femtovg::{renderer::Renderer, Canvas, Color, ImageFlags, Paint, Path};
use image::RgbaImage;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct SharedRendererState {
    pub current_image_id: Option<femtovg::ImageId>,
    pub next_image_id: Option<femtovg::ImageId>,
    pub transition_start: Option<std::time::Instant>,
    pub needs_redraw: bool,
    pub cached_image: Option<RgbaImage>,
    pub pending_next: Option<RgbaImage>,
    pub pending_current: Option<RgbaImage>,
}

unsafe impl Send for SharedRendererState {}
unsafe impl Sync for SharedRendererState {}

impl Default for SharedRendererState {
    fn default() -> Self {
        Self {
            current_image_id: None,
            next_image_id: None,
            transition_start: None,
            needs_redraw: false,
            cached_image: None,
            pending_next: None,
            pending_current: None,
        }
    }
}

pub struct GlobalRenderer {
    pub bg_state: Mutex<SharedRendererState>,
    #[cfg(not(target_os = "linux"))]
    pub cond: std::sync::Condvar,
}

pub fn init_global_renderer(app: &mut tauri::App) {
    app.manage(Arc::new(GlobalRenderer {
        bg_state: Mutex::new(SharedRendererState::default()),
        #[cfg(not(target_os = "linux"))]
        cond: std::sync::Condvar::new(),
    }));
}


pub fn update_background(img: RgbaImage) {
    if let Some(shared) = app_handle().try_state::<Arc<GlobalRenderer>>() {
        let mut state = shared.bg_state.lock().unwrap();
        state.cached_image = Some(img.clone());
        state.pending_next = Some(img);
        state.needs_redraw = true;

        #[cfg(not(target_os = "linux"))]
        shared.cond.notify_one();
    }
}

pub fn restore_background() {
    if let Some(shared) = app_handle().try_state::<Arc<GlobalRenderer>>() {
        let mut state = shared.bg_state.lock().unwrap();

        if let Some(img) = state.cached_image.clone() {
            state.pending_current = Some(img);
            state.transition_start = None;
            state.needs_redraw = true;

            #[cfg(not(target_os = "linux"))]
            shared.cond.notify_one();
        }
    }
}

pub fn trigger_redraw() {
    if let Some(shared) = app_handle().try_state::<Arc<GlobalRenderer>>() {
        let mut state = shared.bg_state.lock().unwrap();
        state.needs_redraw = true;

        #[cfg(not(target_os = "linux"))]
        shared.cond.notify_one();
    }
}

// ── Common Canvas Helpers ──────────────────────────────────────────────────────

fn load_rgba_as_image<T: Renderer>(
    canvas: &mut Canvas<T>,
    img: &RgbaImage,
) -> Option<femtovg::ImageId> {
    use std::io::Cursor;
    let mut png_bytes = Vec::new();
    if image::write_buffer_with_format(
        &mut Cursor::new(&mut png_bytes),
        img.as_raw(),
        img.width(),
        img.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .is_err()
    {
        return None;
    }
    canvas.load_image_mem(&png_bytes, ImageFlags::empty()).ok()
}

// Common drawing logic for both WGPU and GTK/OpenGL loops.
pub fn draw_background<T: Renderer>(canvas: &mut Canvas<T>, state: &mut SharedRendererState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;

    canvas.clear_rect(
        0,
        0,
        canvas.width(),
        canvas.height(),
        Color::rgba(0, 0, 0, 0),
    );

    // Upload pending images into the canvas using whatever backend renderer is active
    if let Some(img) = state.pending_next.take() {
        let old = state.next_image_id.take();
        if let Some(id) = old {
            let _ = canvas.delete_image(id);
        }
        state.next_image_id = load_rgba_as_image(canvas, &img);
        state.transition_start = Some(std::time::Instant::now());
    }

    if let Some(img) = state.pending_current.take() {
        let old = state.current_image_id.take();
        if let Some(id) = old {
            let _ = canvas.delete_image(id);
        }
        state.current_image_id = load_rgba_as_image(canvas, &img);
    }

    // Check transition progress
    if let Some(start) = state.transition_start {
        let elapsed = start.elapsed().as_secs_f32();
        if elapsed >= 0.75 {
            // Transition complete: Promote next into current!
            let old = state.current_image_id.take();
            if let Some(id) = old {
                let _ = canvas.delete_image(id);
            }
            state.current_image_id = state.next_image_id.take();
            state.transition_start = None;
        }
    }

    let mix = if let Some(start) = state.transition_start {
        (start.elapsed().as_secs_f32() / 0.75).min(1.0)
    } else {
        0.0
    };

    // Draw current image
    if let Some(cur_id) = state.current_image_id {
        let paint = Paint::image(cur_id, 0.0, 0.0, w, h, 0.0, 1.0);
        let mut path = Path::new();
        path.rect(0.0, 0.0, w, h);
        canvas.save();
        canvas.global_composite_operation(femtovg::CompositeOperation::Copy);
        canvas.set_global_alpha(1.0);
        canvas.fill_path(&path, &paint);
        canvas.restore();
    }

    // Draw next image fading in
    if let Some(next_id) = state.next_image_id {
        let paint = Paint::image(next_id, 0.0, 0.0, w, h, 0.0, 1.0);
        let mut path = Path::new();
        path.rect(0.0, 0.0, w, h);
        canvas.save();
        if state.current_image_id.is_none() {
            canvas.global_composite_operation(femtovg::CompositeOperation::Copy);
        }
        canvas.set_global_alpha(mix.max(0.0));
        canvas.fill_path(&path, &paint);
        canvas.restore();
    }
}
