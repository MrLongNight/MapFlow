use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};
use mapmap_media::GifDecoder;
use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

#[test]
fn test_gif_decoder_limit_exceeded() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("too_many_frames.gif");
    let file = File::create(&file_path).unwrap();
    let mut encoder = GifEncoder::new(BufWriter::new(file));
    encoder.set_repeat(Repeat::Infinite).unwrap();

    // Create a 1x1 image
    let image = RgbaImage::new(1, 1);
    let frame = Frame::from_parts(
        image,
        0,
        0,
        Delay::from_saturating_duration(Duration::from_millis(100)),
    );

    // Write 501 frames (MAX_GIF_FRAMES is 500)
    for _ in 0..501 {
        encoder.encode_frame(frame.clone()).unwrap();
    }

    // Ensure properly closed/flushed (Drop handles this, but explicit scope helps if needed)
    drop(encoder);

    // Attempt to load
    match GifDecoder::open(&file_path) {
        Ok(_) => panic!("Should have failed due to frame limit"),
        Err(e) => {
            assert!(format!("{}", e).contains("GIF has too many frames"));
        }
    }
}

#[test]
fn test_gif_decoder_success() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("ok_frames.gif");
    let file = File::create(&file_path).unwrap();
    let mut encoder = GifEncoder::new(BufWriter::new(file));
    encoder.set_repeat(Repeat::Infinite).unwrap();

    // Create a 1x1 image
    let image = RgbaImage::new(1, 1);
    let frame = Frame::from_parts(
        image,
        0,
        0,
        Delay::from_saturating_duration(Duration::from_millis(100)),
    );

    // Write 10 frames
    for _ in 0..10 {
        encoder.encode_frame(frame.clone()).unwrap();
    }
    drop(encoder);

    // Attempt to load
    let result = GifDecoder::open(&file_path);
    assert!(result.is_ok());

    let _decoder = result.unwrap();
}
