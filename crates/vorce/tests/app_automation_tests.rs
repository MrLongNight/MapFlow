//! Application E2E automation tests.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[test]
#[ignore = "requires GPU and display"]
fn test_release_smoke_automation_empty_project() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("vorce crate should live under <workspace>/crates/vorce");

    let fixture_path = workspace_root.join("tests/fixtures/empty_project.vorce");
    let output_dir = workspace_root.join("target/automation_test_output");

    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).ok();
    }
    std::fs::create_dir_all(&output_dir).expect("failed to create output dir");

    let binary_path = workspace_root.join("target/release/Vorce.exe");
    if !binary_path.exists() {
        // Fallback to debug if release not available, but automation tests should ideally run in release
        let debug_path = workspace_root.join("target/debug/Vorce.exe");
        if !debug_path.exists() {
            panic!("Vorce binary not found. Run 'cargo build --bin Vorce' first.");
        }
    }

    // Run the app in automation mode
    let mut child = Command::new(&binary_path)
        .arg("--mode")
        .arg("automation")
        .arg("--fixture")
        .arg(fixture_path.to_str().unwrap())
        .arg("--exit-after-frames")
        .arg("10")
        .arg("--screenshot-dir")
        .arg(output_dir.to_str().unwrap())
        .spawn()
        .expect("failed to start Vorce");

    // Wait for the app to finish (should exit after 10 frames)
    let status = child
        .wait_timeout(Duration::from_secs(30))
        .expect("failed to wait for Vorce")
        .expect("Vorce timed out");

    assert!(status.success(), "Vorce exited with failure");

    // Verify screenshot was created
    let screenshot_path = output_dir.join("automation_frame_10.png");
    assert!(
        screenshot_path.exists(),
        "screenshot was not created at {:?}",
        screenshot_path
    );

    let img = image::open(&screenshot_path).expect("failed to open created screenshot");
    assert_eq!(img.width(), 1280);
    assert_eq!(img.height(), 720);
}

// Extension trait for Command to add timeout
trait CommandTimeout {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl CommandTimeout for std::process::Child {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(Some(status));
            }
            if start.elapsed() >= timeout {
                self.kill()?;
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
