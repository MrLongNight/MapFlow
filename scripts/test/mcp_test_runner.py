#!/usr/bin/env python3
import sys
import json
import subprocess
import time
import os

def run_test_script(mapflow_exe, script_path):
    with open(script_path, 'r') as f:
        test_script = json.load(f)

    print(f"Starting test runner for script: {script_path}")
    print(f"Launching MapFlow: {mapflow_exe}")

    # For CI and headless testing we should pass env vars, e.g. WGPU_BACKEND=software or similar
    env = os.environ.copy()

    # We launch mapflow in background
    # For headless CI environments without a display, wrap the launch with xvfb-run
    cmd = [mapflow_exe]
    if not env.get("DISPLAY"):
        import shutil
        if shutil.which("xvfb-run"):
            print("DISPLAY not set. Launching with xvfb-run...")
            cmd = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24"] + cmd
        else:
            print("DISPLAY not set and xvfb-run not found. GUI launch might fail.")

    proc = subprocess.Popen(cmd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    # Wait for the server to spin up by monitoring stdout for a ready signal, or timeout
    ready_timeout = 15.0
    start_wait = time.time()
    is_ready = False

    import select
    print("Waiting for MapFlow MCP server to initialize...")
    while time.time() - start_wait < ready_timeout:
        # Check if process died early
        if proc.poll() is not None:
            print(f"MapFlow process died prematurely with exit code {proc.returncode}")
            stdout, _ = proc.communicate()
            print(f"Process output:\n{stdout}")
            sys.exit(1)

        r, _, _ = select.select([proc.stdout], [], [], 0.1)
        if r:
            line = proc.stdout.readline()
            if line:
                print(f"[MapFlow] {line.strip()}")
                # MapFlow typically prints "Starting MapFlow" or "McpServer" when ready
                # We'll consider it ready after it starts outputting logs and stays alive for a bit
                if "Starting MapFlow" in line or "MCP" in line or "Starting" in line:
                    is_ready = True
                    # Small buffer time to ensure it's fully up after log
                    time.sleep(1.0)
                    break

    if not is_ready:
        print("Warning: Did not see an explicit 'Starting' log. Proceeding anyway assuming it's up.")
        time.sleep(1)

    commands = test_script.get('commands', [])
    test_name = test_script.get('name', 'unknown_test')

    # Send commands via stdio json-rpc (assuming the MCP server reads from stdin when launched, or we configure it to)
    for i, cmd in enumerate(commands):
        # Abort if process died
        if proc.poll() is not None:
            print(f"MapFlow process died before command could be sent. Exit code {proc.returncode}")
            stdout, _ = proc.communicate()
            print(f"Process output:\n{stdout}")
            sys.exit(1)

        request = {
            "jsonrpc": "2.0",
            "id": i + 1,
            "method": cmd['method'],
            "params": cmd.get('params', {})
        }
        req_str = json.dumps(request)
        print(f"Sending: {req_str}")
        try:
            proc.stdin.write(req_str + "\n")
            proc.stdin.flush()
        except BrokenPipeError:
            print(f"Failed to send command {cmd['method']}: Broken pipe (MapFlow process likely crashed)")
            stdout, _ = proc.communicate()
            print(f"Process output:\n{stdout}")
            sys.exit(1)
        except Exception as e:
            print(f"Failed to send command {cmd['method']}: {e}")
            break
        time.sleep(0.5) # Wait for processing

    # Determine expected output path
    output_dir = os.environ.get("MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR")
    if not output_dir:
        output_dir = os.path.join(os.getcwd(), "tests", "artifacts")
    expected_output = os.path.join(output_dir, f"{test_name}_actual.png")
    timeout = 10
    start = time.time()
    found = False
    print(f"Waiting for {expected_output} to be generated...")
    while time.time() - start < timeout:
        if os.path.exists(expected_output):
            found = True
            print(f"Capture generated: {expected_output}")
            break
        time.sleep(1)

    print("Closing MapFlow...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    if not found:
        print(f"Test failed: Output {expected_output} not generated.", file=sys.stderr)
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
