#!/usr/bin/env python3
import subprocess
import time
import json
import statistics
import os
import sys

# Number of iterations to run the benchmark
ITERATIONS = 5

# Mapflow binary path
BINARY_PATH = "./target/release/MapFlow"

# Output directory for the artifacts
ARTIFACT_DIR = "artifacts/performance"

def run_benchmark():
    # Make sure we build the release binary first to avoid including build time in the benchmark
    print("Building MapFlow release binary (this may take a while)...")
    subprocess.run(["cargo", "build", "--release", "--bin", "MapFlow"], check=True)

    if not os.path.exists(BINARY_PATH):
        print(f"Error: Could not find compiled binary at {BINARY_PATH}.")
        sys.exit(1)

    print(f"Running MapFlow performance benchmark ({ITERATIONS} iterations)...")

    execution_times = []

    # We use a dummy xvfb run if in headless CI or lacking DISPLAY
    use_xvfb = os.environ.get('CI') == 'true' or os.environ.get('GITHUB_ACTIONS') == 'true' or not os.environ.get('DISPLAY')

    for i in range(ITERATIONS):
        print(f"Iteration {i + 1}/{ITERATIONS}...")

        # Build command: run automation mode without a specific fixture to just measure baseline
        # GUI rendering overhead. Exit after 300 frames to make the test finite.
        cmd = [
            BINARY_PATH,
            "--mode", "automation",
            "--exit-after-frames", "300"
        ]

        if use_xvfb:
             cmd = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24"] + cmd

        # We redirect stdout to devnull to avoid cluttering but keep stderr for errors
        start_time = time.time()

        try:
            result = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
            if result.returncode != 0:
                # cargo writes status messages (like "Finished ...") to stderr.
                # Let's check if it actually failed based on common error words
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

    print("\nBenchmark Results (300 frames):")
    print(f"  Average: {avg_time:.4f}s")
    print(f"  Median:  {median_time:.4f}s")
    print(f"  Min:     {min_time:.4f}s")
    print(f"  Max:     {max_time:.4f}s")
    print(f"  Std Dev: {std_dev:.4f}s")

    # Generate Reports
    os.makedirs(ARTIFACT_DIR, exist_ok=True)

    report_data = {
        "timestamp": time.time(),
        "iterations": ITERATIONS,
        "frames_per_run": 300,
        "results": {
            "average_seconds": avg_time,
            "median_seconds": median_time,
            "min_seconds": min_time,
            "max_seconds": max_time,
            "std_dev_seconds": std_dev,
            "raw_times": execution_times
        }
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
        f.write(f"Iterations: {ITERATIONS}\n")
        f.write(f"Frames per run: 300\n\n")
        f.write(f"Average Execution Time: {avg_time:.4f} seconds\n")
        f.write(f"Median Execution Time:  {median_time:.4f} seconds\n")
        f.write(f"Minimum Execution Time: {min_time:.4f} seconds\n")
        f.write(f"Maximum Execution Time: {max_time:.4f} seconds\n")
        f.write(f"Standard Deviation:     {std_dev:.4f} seconds\n\n")
        f.write("Raw Execution Times:\n")
        for i, t in enumerate(execution_times):
            f.write(f"  Run {i+1}: {t:.4f} seconds\n")

    print(f"\nReports saved to {ARTIFACT_DIR}/")

if __name__ == "__main__":
    run_benchmark()
