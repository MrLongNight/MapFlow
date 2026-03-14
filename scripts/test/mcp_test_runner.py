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
    # Since MapFlow does not have a dedicated headless mode arg yet we just run it
    proc = subprocess.Popen([mapflow_exe], env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

    # Wait for the server to spin up
    time.sleep(2)

    commands = test_script.get('commands', [])
    test_name = test_script.get('name', 'unknown_test')

    # Send commands via stdio json-rpc (assuming the MCP server reads from stdin when launched, or we configure it to)
    # Looking at the codebase, MapFlow might start an MCP server on a port or stdio depending on config.
    # Usually MCP implies JSON-RPC over stdio.
    # Let's send the commands.

    for i, cmd in enumerate(commands):
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
        except Exception as e:
            print(f"Failed to send command {cmd['method']}: {e}")
            break
        time.sleep(0.5) # Wait for processing

    # Wait for the capture file to be generated
    expected_output = f"tests/artifacts/{test_name}_actual.png"
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
