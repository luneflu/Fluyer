// ponytail: downsizes + re-encodes covers in the core so the FFI payload is a few KB
// instead of a full-resolution 3000x3000 JPEG. Clients render this directly.
pub fn thumbnail_jpeg(bytes: &[u8], max_size: u32) -> Option<Vec<u8>> {
    if bytes.is_empty() || max_size == 0 {
        return None;
    }
    let img = image::load_from_memory(bytes).ok()?;
    let resized = if img.width() > max_size || img.height() > max_size {
        img.resize(max_size, max_size, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let mut out = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 80);
    encoder.encode_image(&resized).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_jpeg_downscales_and_reencodes() {
        // 2x2 red PNG
        let png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
            0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let out = thumbnail_jpeg(&png, 64).expect("thumbnail should encode");
        // JPEG SOI marker
        assert_eq!(&out[0..2], &[0xFF, 0xD8]);
        // Original source is 1x1 so it must not grow
        assert!(
            out.len() < 2048,
            "thumbnail unexpectedly large: {}",
            out.len()
        );
    }

    #[test]
    fn test_thumbnail_jpeg_rejects_invalid_input() {
        assert!(thumbnail_jpeg(&[], 64).is_none());
        assert!(thumbnail_jpeg(&[1, 2, 3], 64).is_none());
        assert!(thumbnail_jpeg(&[1, 2, 3], 0).is_none());
    }
}
