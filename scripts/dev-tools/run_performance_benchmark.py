#!/usr/bin/env python3
import subprocess
import time
import json
import statistics
import os
import sys
import argparse

# Number of iterations to run the benchmark
DEFAULT_ITERATIONS = 5

# Mapflow binary path
BINARY_PATH = "./target/release/MapFlow"

# Output directory for the artifacts
ARTIFACT_DIR = "artifacts/performance"

def run_benchmark():
    parser = argparse.ArgumentParser(description="MapFlow Performance Benchmark")
    parser.add_argument("--iterations", type=int, default=DEFAULT_ITERATIONS, help=f"Number of iterations (default: {DEFAULT_ITERATIONS})")
    parser.add_argument("--threshold", type=float, help="Maximum allowed average execution time in seconds")
    parser.add_argument("--fail-on-regression", action="store_true", help="Exit with code 1 if threshold is exceeded")
    parser.add_argument("--binary", type=str, default=BINARY_PATH, help=f"Path to the binary (default: {BINARY_PATH})")
    parser.add_argument("--frames", type=int, default=300, help="Number of frames to run per iteration (default: 300)")

    args = parser.parse_args()

    iterations = args.iterations
    binary_path = args.binary
    frames = args.frames

    # Make sure we build the release binary first to avoid including build time in the benchmark
    print(f"Building MapFlow release binary for {binary_path}...")
    # MapFlow binary is usually produced by 'mapmap' crate
    subprocess.run(["cargo", "build", "--release", "-p", "mapmap"], check=True)

    # Resolve binary path - cargo might put it in different places depending on OS
    if not os.path.exists(binary_path):
        # Try windows extension
        if os.path.exists(binary_path + ".exe"):
            binary_path += ".exe"
        else:
            print(f"Error: Could not find compiled binary at {binary_path}.")
            sys.exit(1)

    print(f"Running MapFlow performance benchmark ({iterations} iterations, {frames} frames each)...")

    execution_times = []

    # We use a dummy xvfb run if in headless CI or lacking DISPLAY (Linux only)
    is_windows = sys.platform == "win32"
    use_xvfb = not is_windows and (os.environ.get('CI') == 'true' or os.environ.get('GITHUB_ACTIONS') == 'true' or not os.environ.get('DISPLAY'))

    for i in range(iterations):
        print(f"Iteration {i + 1}/{iterations}...")

        # Build command: run automation mode without a specific fixture to just measure baseline
        # GUI rendering overhead. Exit after specified frames to make the test finite.
        cmd = [
            binary_path,
            "--mode", "automation",
            "--exit-after-frames", str(frames)
        ]

        if use_xvfb:
             cmd = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24"] + cmd

        # We redirect stdout to devnull to avoid cluttering but keep stderr for errors
        start_time = time.time()

        try:
            result = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
            if result.returncode != 0:
                # Some apps exit with non-zero even on success if they are killed or closed via automation
                # Let's check stderr for actual errors
                if "error:" in result.stderr.lower() or "panic" in result.stderr.lower():
                    print(f"Error running iteration {i+1}:")
                    print(result.stderr)
                    sys.exit(1)
        except Exception as e:
            print(f"Exception during execution: {e}")
            sys.exit(1)

        end_time = time.time()

        duration = end_time - start_time
        execution_times.append(duration)
        print(f"  Duration: {duration:.4f} seconds")

    # Calculate statistics
    avg_time = statistics.mean(execution_times)
    min_time = min(execution_times)
    max_time = max(execution_times)
    median_time = statistics.median(execution_times)
    std_dev = statistics.stdev(execution_times) if len(execution_times) > 1 else 0.0

    print(f"\nBenchmark Results ({frames} frames):")
    print(f"  Average: {avg_time:.4f}s")
    print(f"  Median:  {median_time:.4f}s")
    print(f"  Min:     {min_time:.4f}s")
    print(f"  Max:     {max_time:.4f}s")
    print(f"  Std Dev: {std_dev:.4f}s")

    # Regression check
    regression = False
    if args.threshold is not None:
        print(f"\nThreshold check: {avg_time:.4f}s vs {args.threshold:.4f}s")
        if avg_time > args.threshold:
            print("  FAIL: Performance regression detected!")
            regression = True
        else:
            print("  PASS: Performance within threshold.")

    # Generate Reports
    os.makedirs(ARTIFACT_DIR, exist_ok=True)

    report_data = {
        "timestamp": time.time(),
        "iterations": iterations,
        "frames_per_run": frames,
        "results": {
            "average_seconds": avg_time,
            "median_seconds": median_time,
            "min_seconds": min_time,
            "max_seconds": max_time,
            "std_dev_seconds": std_dev,
            "raw_times": execution_times
        },
        "threshold": args.threshold,
        "regression": regression
    }

    # Write JSON
    json_path = os.path.join(ARTIFACT_DIR, "performance_report.json")
    with open(json_path, "w") as f:
        json.dump(report_data, f, indent=4)

    # Write TXT
    txt_path = os.path.join(ARTIFACT_DIR, "performance_report.txt")
    with open(txt_path, "w") as f:
        f.write("MapFlow Performance Benchmark Report\n")
        f.write("====================================\n\n")
        f.write(f"Timestamp: {time.ctime()}\n")
        f.write(f"Iterations: {iterations}\n")
        f.write(f"Frames per run: {frames}\n\n")
        f.write(f"Average Execution Time: {avg_time:.4f} seconds\n")
        f.write(f"Median Execution Time:  {median_time:.4f} seconds\n")
        f.write(f"Minimum Execution Time: {min_time:.4f} seconds\n")
        f.write(f"Maximum Execution Time: {max_time:.4f} seconds\n")
        f.write(f"Standard Deviation:     {std_dev:.4f} seconds\n\n")
        if args.threshold is not None:
            f.write(f"Threshold: {args.threshold:.4f} seconds\n")
            f.write(f"Status:    {'FAIL' if regression else 'PASS'}\n\n")
        f.write("Raw Execution Times:\n")
        for i, t in enumerate(execution_times):
            f.write(f"  Run {i+1}: {t:.4f} seconds\n")

    print(f"\nReports saved to {ARTIFACT_DIR}/")

    if regression and args.fail_on_regression:
        sys.exit(1)

if __name__ == "__main__":
    run_benchmark()
