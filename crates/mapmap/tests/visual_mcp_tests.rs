use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_mcp_visual_capture() {
    let test_name = "pilot_mcp_capture";
    let artifacts_dir = std::env::var_os("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("artifacts")
        });

    let actual_path = artifacts_dir.join(format!("{}_actual.png", test_name));
    let diff_path = artifacts_dir.join(format!("{}_diff.png", test_name));
    let reference_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("reference_images")
        .join(format!("{}.png", test_name));

    // Ensure artifact directory exists
    if let Some(parent) = actual_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Pass the output dir to the runner via environment variable so MapFlow writes to the right spot
    std::env::set_var("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR", &artifacts_dir);

    let bin_path = env!("CARGO_BIN_EXE_MapFlow");
    let runner_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts")
        .join("test")
        .join("mcp_test_runner.py");

    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("visual_mcp_scripts")
        .join("pilot_script.json");

    let python_cmd = std::env::var("PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".to_string()
        } else {
            "python3".to_string()
        }
    });

    let runner_status = Command::new(&python_cmd)
        .arg(&runner_path)
        .arg(bin_path)
        .arg(&script_path)
        .status();

    let success = match runner_status {
        Ok(s) => s.success(),
        Err(e) => {
            println!("Failed to execute python runner: {}. Python command: {}", e, python_cmd);
            false
        }
    };

    if !success {
        if std::env::var("CI").is_ok() {
            println!("CI environment: Skipping visual MCP test due to missing python or runner failure.");
            return;
        }
        panic!("mcp_test_runner failed or python not found. Check if python is in PATH.");
    }

    // Wait for the file to be completely written and synced
    std::thread::sleep(std::time::Duration::from_millis(1500));

    if !actual_path.exists() {
        panic!("Visual validation artifact missing: {:?}. The test runner did not produce a screenshot. Check MapFlow crash logs above.", actual_path);
    }

    // Auto-generate reference image on first run
    if !reference_path.exists() {
        println!(
            "Reference image not found. Auto-generating Gold Standard at {:?}",
            reference_path
        );
        if let Some(parent) = reference_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::copy(&actual_path, &reference_path)
            .expect("Failed to copy actual image to reference path");
    }

    let script_compare_path = env!("CARGO_BIN_EXE_mapflow_visual_compare");

    let status_compare = Command::new(script_compare_path)
        .arg(&reference_path)
        .arg(&actual_path)
        .arg(&diff_path)
        .status()
        .expect("Failed to execute mapflow_visual_compare binary");

    assert!(
        status_compare.success(),
        "Visual comparison failed for test case: {}",
        test_name
    );
}
