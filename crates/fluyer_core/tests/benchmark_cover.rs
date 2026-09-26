use fluyer_core::metadata::MusicMetadata;
use fluyer_core::services::coverart::CoverArtService;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn create_synthetic_mp3(path: &PathBuf, dummy_size_mb: usize) -> Vec<u8> {
    let fake_jpeg: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00];

    // APIC body:
    // encoding: 0x00 (ISO-8859-1)
    // mime: "image/jpeg\0"
    // pic_type: 0x03
    // description: "\0"
    // data: fake_jpeg
    let mut apic_body = Vec::new();
    apic_body.push(0x00);
    apic_body.extend_from_slice(b"image/jpeg\0");
    apic_body.push(0x03);
    apic_body.push(0x00);
    apic_body.extend_from_slice(&fake_jpeg);

    let mut id3_body = Vec::new();
    // APIC frame header (ID3v2.3)
    id3_body.extend_from_slice(b"APIC");
    id3_body.extend_from_slice(&(apic_body.len() as u32).to_be_bytes());
    id3_body.extend_from_slice(&[0x00, 0x00]); // flags
    id3_body.extend_from_slice(&apic_body);

    let id3_len = id3_body.len() as u32;
    // Encode syncsafe integer
    let s0 = ((id3_len >> 21) & 0x7F) as u8;
    let s1 = ((id3_len >> 14) & 0x7F) as u8;
    let s2 = ((id3_len >> 7) & 0x7F) as u8;
    let s3 = (id3_len & 0x7F) as u8;

    let mut buf = Vec::new();
    buf.extend_from_slice(b"ID3");
    buf.extend_from_slice(&[0x03, 0x00]); // ID3v2.3
    buf.push(0x00); // flags
    buf.extend_from_slice(&[s0, s1, s2, s3]);
    buf.extend_from_slice(&id3_body);

    // Append valid MP3 frames (MPEG 1 Layer 3, 128 kbps, 44.1 kHz = 417 bytes frame size)
    // Frame header: 0xFF, 0xFB, 0x90, 0x64
    let mut mp3_frame = vec![0u8; 417];
    mp3_frame[0] = 0xFF;
    mp3_frame[1] = 0xFB;
    mp3_frame[2] = 0x90;
    mp3_frame[3] = 0x64;

    let total_frames = (dummy_size_mb * 1024 * 1024) / 417;
    let mut file = File::create(path).unwrap();
    file.write_all(&buf).unwrap();
    for _ in 0..total_frames {
        file.write_all(&mp3_frame).unwrap();
    }
    file.flush().unwrap();

    fake_jpeg
}

fn create_synthetic_flac(path: &PathBuf, dummy_size_mb: usize) -> Vec<u8> {
    let fake_jpeg: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00];

    let mut buf = Vec::new();
    buf.extend_from_slice(b"fLaC");

    // STREAMINFO block (34 bytes): type 0, not last block (0x00), length 34 (0x00, 0x00, 0x22)
    buf.push(0x00);
    buf.extend_from_slice(&[0x00, 0x00, 0x22]);
    let mut streaminfo = vec![0u8; 34];
    streaminfo[0..2].copy_from_slice(&4096u16.to_be_bytes());
    streaminfo[2..4].copy_from_slice(&4096u16.to_be_bytes());
    streaminfo[10] = 0x0A;
    streaminfo[11] = 0xC4;
    streaminfo[12] = 0x42;
    streaminfo[13] = 0xF0;
    buf.extend_from_slice(&streaminfo);

    // PICTURE block (type 6, last block = 0x86)
    let mut pic_block = Vec::new();
    pic_block.extend_from_slice(&3u32.to_be_bytes());
    let mime = b"image/jpeg";
    pic_block.extend_from_slice(&(mime.len() as u32).to_be_bytes());
    pic_block.extend_from_slice(mime);
    pic_block.extend_from_slice(&0u32.to_be_bytes());
    pic_block.extend_from_slice(&500u32.to_be_bytes());
    pic_block.extend_from_slice(&500u32.to_be_bytes());
    pic_block.extend_from_slice(&24u32.to_be_bytes());
    pic_block.extend_from_slice(&0u32.to_be_bytes());
    pic_block.extend_from_slice(&(fake_jpeg.len() as u32).to_be_bytes());
    pic_block.extend_from_slice(&fake_jpeg);

    buf.push(0x86);
    let pic_len = pic_block.len() as u32;
    buf.push((pic_len >> 16) as u8);
    buf.push((pic_len >> 8) as u8);
    buf.push(pic_len as u8);
    buf.extend_from_slice(&pic_block);

    let chunk = vec![0xABu8; 1024 * 1024];
    let mut file = File::create(path).unwrap();
    file.write_all(&buf).unwrap();
    for _ in 0..dummy_size_mb {
        file.write_all(&chunk).unwrap();
    }
    file.flush().unwrap();

    fake_jpeg
}

#[test]
fn test_benchmark_image_read_performance() {
    let temp_dir = std::env::temp_dir().join("fluyer_bench");
    let _ = std::fs::create_dir_all(&temp_dir);
    let mp3_path = temp_dir.join("test_track.mp3");
    let flac_path = temp_dir.join("test_track.flac");
    let cache_dir = temp_dir.join("cache");
    let _ = std::fs::create_dir_all(&cache_dir);

    // Create 10MB MP3 & 20MB FLAC files
    let original_bytes_mp3 = create_synthetic_mp3(&mp3_path, 10);
    let original_bytes_flac = create_synthetic_flac(&flac_path, 20);

    let mp3_path_str = mp3_path.to_str().unwrap();
    let flac_path_str = flac_path.to_str().unwrap();

    // Verify extractions
    assert_eq!(MusicMetadata::get_image_with_lofty(mp3_path_str).unwrap(), original_bytes_mp3);
    assert_eq!(MusicMetadata::get_image_with_symphonia(mp3_path_str).unwrap(), original_bytes_mp3);
    assert_eq!(MusicMetadata::get_image_with_lofty(flac_path_str).unwrap(), original_bytes_flac);

    let cover_service = CoverArtService::new(&cache_dir);
    const ITERATIONS: u32 = 100;

    // A) MP3 Symphonia
    let t0 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = MusicMetadata::get_image_with_symphonia(mp3_path_str).unwrap();
    }
    let symphonia_mp3_time = t0.elapsed();

    // B) MP3 Lofty
    let t1 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = MusicMetadata::get_image_with_lofty(mp3_path_str).unwrap();
    }
    let lofty_mp3_time = t1.elapsed();

    // C) FLAC Lofty
    let t2 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = MusicMetadata::get_image_with_lofty(flac_path_str).unwrap();
    }
    let lofty_flac_time = t2.elapsed();

    // Populate disk cache
    cover_service.save_to_cache("Artist", Some("Album"), Some("Track"), Some(mp3_path_str), &original_bytes_mp3).unwrap();

    // D) Disk Cache Hit
    let t3 = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = cover_service.get_cached_with_fallback("Artist", Some("Album"), Some("Track"), Some(mp3_path_str)).unwrap();
    }
    let disk_cache_time = t3.elapsed();

    let sym_mp3_us = symphonia_mp3_time.as_micros() as f64 / ITERATIONS as f64;
    let lofty_mp3_us = lofty_mp3_time.as_micros() as f64 / ITERATIONS as f64;
    let lofty_flac_us = lofty_flac_time.as_micros() as f64 / ITERATIONS as f64;
    let cache_us = disk_cache_time.as_micros() as f64 / ITERATIONS as f64;

    println!("\n========================================================");
    println!("IMAGE EXTRACTION BENCHMARK ({} iterations)", ITERATIONS);
    println!("========================================================");
    println!("MP3  (10MB) - Symphonia probe : {:>8.2} µs / read", sym_mp3_us);
    println!("MP3  (10MB) - Lofty metadata  : {:>8.2} µs / read", lofty_mp3_us);
    println!("FLAC (20MB) - Lofty metadata  : {:>8.2} µs / read", lofty_flac_us);
    println!("Disk Cache hit (SSD/Cache)    : {:>8.2} µs / read", cache_us);
    println!("--------------------------------------------------------");
    println!("Cache vs MP3 Symphonia : {:.2}x faster", sym_mp3_us / cache_us);
    println!("Cache vs FLAC Lofty    : {:.2}x faster", lofty_flac_us / cache_us);
    println!("========================================================\n");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

use std::io::{Read, Seek, SeekFrom};

struct CountingReader<R> {
    inner: R,
    bytes_read: usize,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read += n;
        Ok(n)
    }
}

impl<R: Seek> Seek for CountingReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

#[test]
fn test_verify_lofty_does_not_read_whole_file() {
    use lofty::config::ParseOptions;
    use lofty::probe::Probe;

    let temp_dir = std::env::temp_dir().join("fluyer_proof");
    let _ = std::fs::create_dir_all(&temp_dir);
    let flac_path = temp_dir.join("proof_50mb.flac");

    // Create a 50MB FLAC file
    let _ = create_synthetic_flac(&flac_path, 50);
    let total_file_size = std::fs::metadata(&flac_path).unwrap().len();

    let file = File::open(&flac_path).unwrap();
    let mut counting = CountingReader {
        inner: file,
        bytes_read: 0,
    };

    let probe = Probe::new(&mut counting)
        .options(ParseOptions::new().read_properties(false))
        .guess_file_type()
        .unwrap();
    let _ = probe.read().unwrap();

    let bytes_read = counting.bytes_read as u64;
    let percent_read = (bytes_read as f64 / total_file_size as f64) * 100.0;

    println!("\n========================================================");
    println!("LOFTY READ VERIFICATION (50MB FLAC FILE)");
    println!("========================================================");
    println!("Total file size on disk : {} bytes (~50 MB)", total_file_size);
    println!("Bytes read by Lofty     : {} bytes (~{:.2} KB)", bytes_read, bytes_read as f64 / 1024.0);
    println!("Percentage of file read : {:.4}%", percent_read);
    println!("========================================================\n");

    // Lofty read less than 0.01% of the file, completely ignoring the 50MB audio payload!
    assert!(bytes_read < 10_000, "Lofty must only read metadata header, read {} bytes", bytes_read);
    assert!(bytes_read < total_file_size / 1000);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
