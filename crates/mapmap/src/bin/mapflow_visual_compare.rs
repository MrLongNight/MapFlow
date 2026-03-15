use image::{ImageBuffer, Rgba};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CHANNEL_TOLERANCE: u8 = 2;
const MAX_MISMATCH_RATIO: f32 = 0.001;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <expected_path> <actual_path> [diff_path]",
            args[0]
        );
        std::process::exit(1);
    }

    let expected_path = Path::new(&args[1]);
    let actual_path = Path::new(&args[2]);
    let diff_path = if args.len() > 3 {
        PathBuf::from(&args[3])
    } else {
        PathBuf::from("diff.png")
    };

    if !expected_path.exists() {
        eprintln!("Expected path does not exist: {}", expected_path.display());
        std::process::exit(1);
    }

    if !actual_path.exists() {
        eprintln!("Actual path does not exist: {}", actual_path.display());
        std::process::exit(1);
    }

    println!("Running pixel-based diff...");
    if pixel_diff(expected_path, actual_path, &diff_path) {
        println!("Pixel comparison passed.");
        std::process::exit(0);
    }

    println!("Running FFmpeg SSIM comparison...");
    if ffmpeg_ssim(expected_path, actual_path) {
        println!("FFmpeg SSIM comparison passed.");
        std::process::exit(0);
    }

    println!("Running AI fallback comparison...");
    if ai_fallback(expected_path, actual_path) {
        println!("AI comparison passed.");
        std::process::exit(0);
    }

    println!("All comparisons failed.");
    std::process::exit(1);
}

fn pixel_diff(expected_path: &Path, actual_path: &Path, diff_path: &Path) -> bool {
    let expected = match image::open(expected_path) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            eprintln!("Error opening expected image: {}", e);
            return false;
        }
    };
    let actual = match image::open(actual_path) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            eprintln!("Error opening actual image: {}", e);
            return false;
        }
    };

    if expected.dimensions() != actual.dimensions() {
        eprintln!(
            "Size mismatch: expected {:?}, got {:?}",
            expected.dimensions(),
            actual.dimensions()
        );
        return false;
    }

    let (width, height) = expected.dimensions();
    let total_pixels = (width * height) as usize;
    let max_mismatched_pixels = ((total_pixels as f32) * MAX_MISMATCH_RATIO).ceil() as usize;
    let mut mismatched_pixels = 0usize;
    let mut diff_image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let expected_pixel = expected.get_pixel(x, y);
            let actual_pixel = actual.get_pixel(x, y);

            let local_max = expected_pixel
                .0
                .iter()
                .zip(actual_pixel.0.iter())
                .map(|(left, right)| left.abs_diff(*right))
                .max()
                .unwrap_or(0);

            if local_max > CHANNEL_TOLERANCE {
                mismatched_pixels += 1;
                diff_image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                diff_image.put_pixel(x, y, *actual_pixel);
            }
        }
    }

    if mismatched_pixels > max_mismatched_pixels {
        eprintln!(
            "Pixel mismatch: {} > {}",
            mismatched_pixels, max_mismatched_pixels
        );
        if let Err(e) = diff_image.save(diff_path) {
            eprintln!("Failed to save diff image: {}", e);
        }
        return false;
    }

    true
}

fn ffmpeg_ssim(expected_path: &Path, actual_path: &Path) -> bool {
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(actual_path)
        .arg("-i")
        .arg(expected_path)
        .arg("-filter_complex")
        .arg("ssim")
        .arg("-f")
        .arg("null")
        .arg("-")
        .output();

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            for line in stderr.lines() {
                if line.contains("SSIM") {
                    println!("FFmpeg comparison: {}", line);
                    if let Some(all_part) = line.split("All:").nth(1) {
                        let score_str = all_part.split_whitespace().next().unwrap_or("0");
                        if let Ok(score) = score_str.parse::<f32>() {
                            if score >= 0.99 {
                                return true;
                            } else {
                                eprintln!("SSIM score too low: {} < 0.99", score);
                                return false;
                            }
                        }
                    }
                }
            }
            eprintln!("SSIM score not found in FFmpeg output");
            false
        }
        Err(e) => {
            eprintln!("FFmpeg comparison failed: {}", e);
            false
        }
    }
}

fn ai_fallback(_expected_path: &Path, _actual_path: &Path) -> bool {
    eprintln!(
        "AI fallback not fully implemented. Consider adding Gemini Vision API integration here."
    );
    false
}
