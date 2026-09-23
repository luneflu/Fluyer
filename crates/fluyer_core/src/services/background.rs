use image::{ImageReader, Rgba, RgbaImage};
use rand::seq::IndexedRandom;
use std::cmp::max;
use std::collections::HashMap;
use std::io::Cursor;

pub const DEFAULT_SCALE: f32 = 0.05;
pub const CANVAS_BLOCK_SIZE: u32 = 200;
pub const CANVAS_BLUR_RADIUS: u32 = 300;

// ponytail: quantize channel to match color.js group()
#[inline]
pub fn group_channel(number: u8, grouping: u8) -> u8 {
    if grouping == 0 {
        return number;
    }
    let grouped = ((number as f32 / grouping as f32).round() * grouping as f32) as u32;
    grouped.min(255) as u8
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max_val = rf.max(gf).max(bf);
    let min_val = rf.min(gf).min(bf);
    let delta = max_val - min_val;

    let l = (max_val + min_val) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l * 100.0);
    }

    let s = if l > 0.5 {
        delta / (2.0 - max_val - min_val)
    } else {
        delta / (max_val + min_val)
    };

    let mut h = if (max_val - rf).abs() < f32::EPSILON {
        (gf - bf) / delta + if gf < bf { 6.0 } else { 0.0 }
    } else if (max_val - gf).abs() < f32::EPSILON {
        (bf - rf) / delta + 2.0
    } else {
        (rf - gf) / delta + 4.0
    } / 6.0;

    h *= 360.0;
    if h < 0.0 {
        h += 360.0;
    }

    (h, s * 100.0, l * 100.0)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let sf = (s / 100.0).clamp(0.0, 1.0);
    let lf = (l / 100.0).clamp(0.0, 1.0);
    let h_norm = (h % 360.0 + 360.0) % 360.0 / 360.0;

    if sf == 0.0 {
        let val = (lf * 255.0).round().clamp(0.0, 255.0) as u8;
        return (val, val, val);
    }

    let q = if lf < 0.5 {
        lf * (1.0 + sf)
    } else {
        lf + sf - lf * sf
    };
    let p = 2.0 * lf - q;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    (
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

// ponytail: matches AnimatedBackground.svelte color balancing
pub fn balance_color(color: [u8; 3], is_default_cover: bool) -> [u8; 3] {
    let (h, mut s, mut l) = rgb_to_hsl(color[0], color[1], color[2]);
    if is_default_cover {
        l = 50.0;
        while s > 40.0 {
            s *= 0.9;
        }
    } else {
        while l > 45.0 {
            l *= 0.9;
        }
        while s > 50.0 {
            s *= 0.9;
        }
    }
    let (r, g, b) = hsl_to_rgb(h, s, l);
    [r, g, b]
}

// ponytail: reimplements prominent() from color.js + svelte balancing
pub fn extract_prominent_colors(
    rgba_data: &[u8],
    amount: usize,
    sample: usize,
    group_val: u8,
    is_default_cover: bool,
) -> Vec<[u8; 3]> {
    if rgba_data.len() < 4 {
        return vec![balance_color([30, 30, 40], is_default_cover)];
    }

    let gap = (4 * sample.max(1)).min(rgba_data.len());
    let mut counts: HashMap<[u8; 3], usize> = HashMap::new();

    let mut i = 0;
    while i + 2 < rgba_data.len() {
        let r = group_channel(rgba_data[i], group_val);
        let g = group_channel(rgba_data[i + 1], group_val);
        let b = group_channel(rgba_data[i + 2], group_val);
        *counts.entry([r, g, b]).or_insert(0) += 1;
        i += gap;
    }

    if counts.is_empty() {
        return vec![balance_color([30, 30, 40], is_default_cover)];
    }

    let mut sorted: Vec<([u8; 3], usize)> = counts.into_iter().collect();
    // Sort descending by frequency
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(amount)
        .map(|(rgb, _)| balance_color(rgb, is_default_cover))
        .collect()
}

pub fn extract_prominent_from_bytes(
    image_bytes: &[u8],
    amount: usize,
    is_default_cover: bool,
) -> Vec<[u8; 3]> {
    let img_res = match image::guess_format(image_bytes) {
        Ok(fmt) => {
            let mut reader = ImageReader::new(Cursor::new(image_bytes));
            reader.set_format(fmt);
            reader.decode()
        }
        Err(_) => ImageReader::new(Cursor::new(image_bytes))
            .with_guessed_format()
            .map_err(|e| image::ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
            .and_then(|r| r.decode()),
    };

    match img_res {
        Ok(dyn_img) => {
            // Thumbnail down to at most 128x128 for instant prominent color extraction
            let thumb = dyn_img.thumbnail(128, 128);
            let rgba = thumb.to_rgba8();
            extract_prominent_colors(rgba.as_raw(), amount, 4, 20, is_default_cover)
        }
        Err(_) => vec![balance_color([30, 30, 40], is_default_cover)],
    }
}

pub fn generate_blurred_background(
    colors: &[[u8; 3]],
    width: u32,
    height: u32,
) -> RgbaImage {
    let scaled_width = max(1, (width as f32 * DEFAULT_SCALE) as u32);
    let scaled_height = max(1, (height as f32 * DEFAULT_SCALE) as u32);
    let block_size = max(1, (CANVAS_BLOCK_SIZE as f32 * DEFAULT_SCALE) as u32);

    let cols = (scaled_width as f32 / block_size as f32).ceil() as u32;
    let rows = (scaled_height as f32 / block_size as f32).ceil() as u32;

    let mut canvas = RgbaImage::new(scaled_width, scaled_height);
    let mut rng = rand::rng();

    let fallback_colors = [[20, 20, 26], [30, 30, 38], [15, 18, 24]];
    let color_palette = if !colors.is_empty() {
        colors
    } else {
        &fallback_colors
    };

    for y in 0..rows {
        for x in 0..cols {
            let color = color_palette.choose(&mut rng).unwrap();

            let x_start = x * block_size;
            let y_start = y * block_size;

            for by in 0..block_size {
                for bx in 0..block_size {
                    if x_start + bx < scaled_width && y_start + by < scaled_height {
                        canvas.put_pixel(
                            x_start + bx,
                            y_start + by,
                            Rgba([color[0], color[1], color[2], 255]),
                        );
                    }
                }
            }
        }
    }

    let blur_radius = max(1, (CANVAS_BLUR_RADIUS as f32 * DEFAULT_SCALE) as u32);
    image::imageops::blur(&canvas, blur_radius as f32 / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_channel() {
        assert_eq!(group_channel(0, 20), 0);
        assert_eq!(group_channel(9, 20), 0);
        assert_eq!(group_channel(11, 20), 20);
        assert_eq!(group_channel(255, 20), 255);
    }

    #[test]
    fn test_hsl_roundtrip() {
        let (h, s, l) = rgb_to_hsl(200, 100, 50);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        assert!((r as i32 - 200).abs() <= 1);
        assert!((g as i32 - 100).abs() <= 1);
        assert!((b as i32 - 50).abs() <= 1);
    }

    #[test]
    fn test_balance_color() {
        let col = balance_color([255, 0, 0], false);
        let (_, s, l) = rgb_to_hsl(col[0], col[1], col[2]);
        assert!(l <= 45.0 + 1.0);
        assert!(s <= 50.0 + 1.0);
    }

    #[test]
    fn test_extract_prominent_colors() {
        // Red, Green, Blue pixels
        let mut pixels = Vec::new();
        for _ in 0..50 {
            pixels.extend_from_slice(&[255, 0, 0, 255]);
        }
        for _ in 0..20 {
            pixels.extend_from_slice(&[0, 255, 0, 255]);
        }
        let colors = extract_prominent_colors(&pixels, 2, 1, 20, false);
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_generate_blurred_background() {
        let colors = vec![[100, 50, 150], [50, 100, 200]];
        let bg = generate_blurred_background(&colors, 800, 600);
        assert_eq!(bg.width(), 40);
        assert_eq!(bg.height(), 30);
    }
}
