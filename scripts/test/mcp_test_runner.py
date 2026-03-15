#!/usr/bin/env python3
import sys
import json
import subprocess
import time
import os
import threading
import queue

def reader(pipe, queue):
    try:
        with pipe:
            for line in iter(pipe.readline, ''):
                queue.put(line)
    except Exception:
        pass

def run_test_script(mapflow_exe, script_path):
    with open(script_path, 'r') as f:
        test_script = json.load(f)

    print(f"Starting test runner for script: {script_path}")
    print(f"Launching MapFlow: {mapflow_exe}")

    env = os.environ.copy()
    # Add RUST_LOG to help debug readiness
    if not env.get("RUST_LOG"):
        env["RUST_LOG"] = "info,mapmap=debug"

    cmd = [mapflow_exe]
    if not env.get("DISPLAY") and sys.platform != 'win32':
        import shutil
        if shutil.which("xvfb-run"):
            print("DISPLAY not set. Launching with xvfb-run...")
            cmd = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24"] + cmd
        else:
            print("DISPLAY not set and xvfb-run not found. GUI launch might fail.")

    proc = subprocess.Popen(
        cmd,
        env=env,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

    q = queue.Queue()
    t = threading.Thread(target=reader, args=(proc.stdout, q))
    t.daemon = True
    t.start()

    ready_timeout = 30.0 # Increased timeout for slow CI environments
    start_wait = time.time()
    is_ready = False
    captured_output = []

    print("Waiting for MapFlow MCP server to initialize...")
    while time.time() - start_wait < ready_timeout:
        if proc.poll() is not None:
            print(f"MapFlow process died prematurely with exit code {proc.returncode}")
            # Collect all remaining output
            while not q.empty():
                captured_output.append(q.get())
            print(f"Process output:\n{''.join(captured_output)}")
            sys.exit(1)

        try:
            line = q.get(timeout=0.1)
            captured_output.append(line)
            print(f"[MapFlow] {line.strip()}")
            if "Starting MapFlow" in line or "McpServer" in line or "Started" in line or "Ready" in line:
                print("Detected MapFlow readiness signal.")
                is_ready = True
                time.sleep(2.0) # Increased buffer time
                break
        except queue.Empty:
            continue

    if not is_ready:
        print("Warning: Did not see an explicit 'Ready' log within timeout. Proceeding anyway.")
        time.sleep(2.0)

    commands = test_script.get('commands', [])
    test_name = test_script.get('name', 'unknown_test')

    for i, cmd in enumerate(commands):
        if proc.poll() is not None:
            print(f"MapFlow process died before command could be sent. Exit code {proc.returncode}")
            while not q.empty():
                captured_output.append(q.get())
            print(f"Process output:\n{''.join(captured_output)}")
            sys.exit(1)

        request = {
            "jsonrpc": "2.0",
            "id": i + 1,
            "method": cmd['method'],
            "params": cmd.get('params', {})
        }
        req_str = json.dumps(request)
        print(f"Sending request {i+1}: {req_str}")
        try:
            proc.stdin.write(req_str + "\n")
            proc.stdin.flush()
        except BrokenPipeError:
            print(f"Failed to send command {cmd['method']}: Broken pipe (MapFlow process likely crashed)")
            while not q.empty():
                captured_output.append(q.get())
            print(f"Process output:\n{''.join(captured_output)}")
            sys.exit(1)
        except Exception as e:
            print(f"Failed to send command {cmd['method']}: {e}")
            break
        time.sleep(1.0) # Increased wait for processing

    output_dir = os.environ.get("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR")
    if not output_dir:
        output_dir = os.path.join(os.getcwd(), "tests", "artifacts")
    expected_output = os.path.join(output_dir, f"{test_name}_actual.png")

    timeout = 15
    start = time.time()
    found = False
    print(f"Waiting for screenshot artifact: {expected_output} ...")
    while time.time() - start < timeout:
        if os.path.exists(expected_output):
            # Check if file size is > 0
            if os.path.getsize(expected_output) > 0:
                found = True
                print(f"Success: Capture generated and valid: {expected_output}")
                break
        time.sleep(1)

    print("Terminating MapFlow...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        print("MapFlow did not terminate gracefully. Killing process.")
        proc.kill()

    if not found:
        print(f"Test failed: Output {expected_output} not generated or empty.", file=sys.stderr)
        # Dump output on failure
        while not q.empty():
            captured_output.append(q.get())
        print(f"Full MapFlow logs for debugging:\n{''.join(captured_output)}", file=sys.stderr)
        sys.exit(1)

    print("MapFlow E2E MCP execution completed successfully.")
    sys.exit(0)

def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <mapflow_executable_path> <test_script_json_path>")
        sys.exit(1)

    mapflow_exe = sys.argv[1]
    script_path = sys.argv[2]
    run_test_script(mapflow_exe, script_path)

if __name__ == "__main__":
    main()
