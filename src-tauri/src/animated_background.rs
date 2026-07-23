use image::{DynamicImage, Rgba, RgbaImage};
use rand::seq::IndexedRandom;
use std::cmp::max;

const SCALE: f32 = 0.05;
const CANVAS_BLOCK_SIZE: u32 = 200; // Avg of 100 and 150
const CANVAS_BLUR_RADIUS: u32 = 300;

#[derive(serde::Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

pub fn generate_blurred_background(
    colors: Vec<Color>,
    width: u32,
    height: u32,
) -> Result<RgbaImage, String> {
    // Generate Grid
    let scaled_width = (width as f32 * SCALE) as u32;
    let scaled_height = (height as f32 * SCALE) as u32;
    let block_size = (CANVAS_BLOCK_SIZE as f32 * SCALE) as u32;

    // Ensure minimum dimensions
    let scaled_width = max(1, scaled_width);
    let scaled_height = max(1, scaled_height);
    let block_size = max(1, block_size);

    let cols = (scaled_width as f32 / block_size as f32).ceil() as u32;
    let rows = (scaled_height as f32 / block_size as f32).ceil() as u32;

    let mut canvas = RgbaImage::new(scaled_width, scaled_height);
    let mut rng = rand::rng();

    for y in 0..rows {
        for x in 0..cols {
            let color = colors.choose(&mut rng).unwrap();

            let x_start = x * block_size;
            let y_start = y * block_size;

            // Fill block
            for by in 0..block_size {
                for bx in 0..block_size {
                    if x_start + bx < scaled_width && y_start + by < scaled_height {
                        canvas.put_pixel(
                            x_start + bx,
                            y_start + by,
                            Rgba([color.r, color.g, color.b, 255]),
                        );
                    }
                }
            }
        }
    }

    // Blur
    let blur_radius = (CANVAS_BLUR_RADIUS as f32 * SCALE) as u32;

    let dyn_img = DynamicImage::ImageRgba8(canvas);
    let kernel_size = (blur_radius * 2) + 1;
    let blurred_dyn = libblur::gaussian_blur_image(
        dyn_img,
        libblur::GaussianBlurParams {
            x_kernel: kernel_size,
            x_sigma: 0.0,
            y_kernel: kernel_size,
            y_sigma: 0.0,
        },
        libblur::EdgeMode2D::new(libblur::EdgeMode::Clamp),
        libblur::ConvolutionMode::FixedPoint,
        libblur::ThreadingPolicy::Adaptive,
    )
    .ok_or_else(|| "Failed to blur".to_string())?;
    Ok(blurred_dyn.to_rgba8())
}

#[tauri::command]
pub async fn is_cef_enabled() -> Result<bool, String> {
    #[cfg(feature = "cef")]
    {
        Ok(true)
    }
    #[cfg(not(feature = "cef"))]
    {
        Ok(false)
    }
}

#[tauri::command]
pub async fn animated_background_update(
    colors: Vec<Color>,
    width: u32,
    height: u32,
) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
    let blurred = generate_blurred_background(colors, width, height)?;

    #[cfg(all(target_os = "linux", feature = "cef"))]
    {
        return Ok(Some((
            blurred.clone().into_raw(),
            blurred.width(),
            blurred.height(),
        )));
    }

    #[cfg(any(not(target_os = "linux"), not(feature = "cef")))]
    {
        crate::renderer::update_background(blurred);
        Ok(None)
    }
}

#[tauri::command]
pub async fn animated_background_restore() -> Result<(), String> {
    #[cfg(any(not(target_os = "linux"), not(feature = "cef")))]
    {
        crate::renderer::restore_background();
    }
    Ok(())
}
