#![cfg(target_os = "windows")]

use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CHANNEL_TOLERANCE: u8 = 2;
const MAX_MISMATCH_RATIO: f32 = 0.001;

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn checkerboard_matches_reference() {
    run_visual_regression("checkerboard");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn alpha_overlay_matches_reference() {
    run_visual_regression("alpha_overlay");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn gradient_matches_reference() {
    run_visual_regression("gradient");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn empty_project_matches_reference() {
    run_visual_regression("empty_project");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn test_grid_matches_reference() {
    run_visual_regression("test_grid");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn projector_warp_matches_reference() {
    run_visual_regression("projector_warp");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn media_playback_matches_reference() {
    run_visual_regression("media_playback");
}

#[test]
#[ignore = "Requires a local interactive Windows GPU/desktop session"]
fn timeline_step_matches_reference() {
    run_visual_regression("timeline_step");
}

fn run_visual_regression(scenario: &str) {
    let (actual_path, diff_path) = unique_output_paths(scenario);
    let expected_path = reference_image_path(scenario);

    if let Some(parent) = actual_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create temporary screenshot directory");
    }

    let binary = env!("CARGO_BIN_EXE_mapflow_visual_harness");
    let output = Command::new(binary)
        .args(["capture", "--scenario", scenario, "--output"])
        .arg(&actual_path)
        .output()
        .expect("Failed to launch mapflow_visual_harness");

    assert!(
        output.status.success(),
        "Visual harness failed for scenario '{scenario}'.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    compare_images(&expected_path, &actual_path, &diff_path);
    println!(
        "Visual capture '{scenario}' saved to {}",
        actual_path.display()
    );
}

fn compare_images(expected_path: &Path, actual_path: &Path, diff_path: &Path) {
    let expected = image::open(expected_path)
        .unwrap_or_else(|err| {
            panic!(
                "Failed to open reference image '{}': {err}",
                expected_path.display()
            )
        })
        .to_rgba8();
    let actual = image::open(actual_path)
        .unwrap_or_else(|err| {
            panic!(
                "Failed to open actual image '{}': {err}",
                actual_path.display()
            )
        })
        .to_rgba8();

    assert_eq!(
        expected.dimensions(),
        actual.dimensions(),
        "Reference image '{}' and actual image '{}' differ in size",
        expected_path.display(),
        actual_path.display(),
    );

    let (width, height) = expected.dimensions();
    let total_pixels = (width * height) as usize;
    let max_mismatched_pixels = ((total_pixels as f32) * MAX_MISMATCH_RATIO).ceil() as usize;
    let mut mismatched_pixels = 0usize;
    let mut max_channel_diff = 0u8;
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

            max_channel_diff = max_channel_diff.max(local_max);
            if local_max > CHANNEL_TOLERANCE {
                mismatched_pixels += 1;
                diff_image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                diff_image.put_pixel(x, y, *actual_pixel);
            }
        }
    }

    if mismatched_pixels > max_mismatched_pixels {
        diff_image.save(diff_path).unwrap_or_else(|err| {
            panic!("Failed to save diff image '{}': {err}", diff_path.display())
        });
        panic!(
            "Scenario mismatch.\nreference: {}\nactual: {}\ndiff: {}\nmax_channel_diff: {} (allowed: {})\nmismatched_pixels: {} (allowed: {}, ratio: {})",
            expected_path.display(),
            actual_path.display(),
            diff_path.display(),
            max_channel_diff,
            CHANNEL_TOLERANCE,
            mismatched_pixels,
            max_mismatched_pixels,
            MAX_MISMATCH_RATIO,
        );
    }
}

fn reference_image_path(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("reference_images")
        .join(format!("{scenario}.png"))
}

fn unique_output_paths(scenario: &str) -> (PathBuf, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock drifted before UNIX_EPOCH")
        .as_millis();
    let root = capture_output_root();
    let run_context = format!("{}_{}", std::process::id(), timestamp);

    // Schema: {test_case}_win-{window_type}_f{frame_index}_{run_context}.{type}.png
    let base_name = format!("{scenario}_win-main_f001_{run_context}");

    let actual_path = root.join(format!("{base_name}.actual.png"));
    let diff_path = root.join(format!("{base_name}.diff.png"));

    (actual_path, diff_path)
}

fn capture_output_root() -> PathBuf {
    match std::env::var_os("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                workspace_root().join(path)
            }
        }
        None => workspace_root().join("artifacts").join("visual-capture"),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mapmap crate should live under <workspace>/crates/mapmap")
        .to_path_buf()
}
