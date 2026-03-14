#!/usr/bin/env python3
import sys
import os
import subprocess
try:
    from PIL import Image, ImageChops
except ImportError:
    print("Warning: PIL not installed. Pixel match will fall back to exact comparison or fail.", file=sys.stderr)

def pixel_diff(expected_path, actual_path, diff_path, tolerance=2, max_mismatch_ratio=0.001):
    try:
        expected = Image.open(expected_path).convert("RGBA")
        actual = Image.open(actual_path).convert("RGBA")
    except Exception as e:
        print(f"Error opening images: {e}", file=sys.stderr)
        return False

    if expected.size != actual.size:
        print(f"Size mismatch: expected {expected.size}, got {actual.size}", file=sys.stderr)
        return False

    diff = ImageChops.difference(expected, actual)
    width, height = expected.size
    total_pixels = width * height
    max_mismatched = int(total_pixels * max_mismatch_ratio)

    mismatched = 0
    diff_image = Image.new("RGBA", (width, height))
    diff_pixels = diff_image.load()
    actual_pixels = actual.load()

    # Simple thresholding
    diff_data = diff.getdata()
    for idx, pixel in enumerate(diff_data):
        x = idx % width
        y = idx // width
        # Compare RGBA channels
        if any(c > tolerance for c in pixel):
            mismatched += 1
            diff_pixels[x, y] = (255, 0, 0, 255)
        else:
            diff_pixels[x, y] = actual_pixels[x, y]

    if mismatched > max_mismatched:
        print(f"Pixel mismatch: {mismatched} > {max_mismatched}", file=sys.stderr)
        diff_image.save(diff_path)
        return False

    return True

def ffmpeg_ssim(expected_path, actual_path):
    cmd = [
        "ffmpeg", "-i", actual_path, "-i", expected_path,
        "-filter_complex", "ssim", "-f", "null", "-"
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True)
        # Parse SSIM output
        for line in result.stderr.splitlines():
            if "SSIM" in line:
                print(f"FFmpeg comparison: {line}")
                if "All:" in line:
                    score = float(line.split("All:")[1].split()[0])
                    if score >= 0.99:
                        return True
                    else:
                        print(f"SSIM score too low: {score} < 0.99", file=sys.stderr)
                        return False
    except Exception as e:
        print(f"FFmpeg comparison failed: {e}", file=sys.stderr)
    return False

def ai_fallback(expected_path, actual_path):
    print("AI fallback not fully implemented. Consider adding Gemini Vision API integration here.", file=sys.stderr)
    return False

def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <expected_path> <actual_path> [diff_path]")
        sys.exit(1)

    expected_path = sys.argv[1]
    actual_path = sys.argv[2]
    diff_path = sys.argv[3] if len(sys.argv) > 3 else "diff.png"

    if not os.path.exists(expected_path):
        print(f"Expected path does not exist: {expected_path}", file=sys.stderr)
        sys.exit(1)

    if not os.path.exists(actual_path):
        print(f"Actual path does not exist: {actual_path}", file=sys.stderr)
        sys.exit(1)

    # Priority 1: Pixel based diff
    print("Running pixel-based diff...")
    if pixel_diff(expected_path, actual_path, diff_path):
        print("Pixel comparison passed.")
        sys.exit(0)

    # Priority 2: FFmpeg SSIM (good for slight compression artifacts or animations)
    print("Running FFmpeg SSIM comparison...")
    if ffmpeg_ssim(expected_path, actual_path):
        print("FFmpeg SSIM comparison passed.")
        sys.exit(0)

    # Priority 3: AI Fallback
    print("Running AI fallback comparison...")
    if ai_fallback(expected_path, actual_path):
        print("AI comparison passed.")
        sys.exit(0)

    print("All comparisons failed.")
    sys.exit(1)

if __name__ == "__main__":
    main()
