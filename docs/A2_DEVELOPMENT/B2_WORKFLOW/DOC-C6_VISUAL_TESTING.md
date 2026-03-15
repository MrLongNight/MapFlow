# MapFlow Visual Testing Validation Hierarchy

The MapFlow visual end-to-end testing pipeline validates the correct rendering of effects, shaders, and media pipelines by using the Model Context Protocol (MCP) to script MapFlow interactions and capture the final rendered output.

## Multi-Stage Validation Engine

Due to the dynamic nature of graphics programming (GPU differences, rounding errors, minor artifacting in compression), strict pixel-perfect tests are often brittle. We employ a three-tier validation engine:

### Priority 1: Pixel Diff (`PIL.ImageChops`)
- **What**: Fast, pixel-by-pixel deterministic comparison.
- **Why**: Instantly catches severe rendering breakages (e.g. blackout screens, checkerboard mismatches).
- **Tool**: Python's `PIL.ImageChops.difference`.

### Priority 2: FFmpeg SSIM
- **What**: Structural Similarity Index Measure (SSIM) computed via FFmpeg.
- **Why**: Handles slight video compression artifacts, animated frames, and fractional shader variations that humans cannot distinguish but that cause exact pixel diffs to fail.
- **Tool**: `ffmpeg -i actual.png -i reference.png -filter_complex ssim -f null -`. A score of `>= 0.99` is required.

### Priority 3: AI Fallback (Future)
- **What**: Evaluates subjective visual elements.
- **Why**: Helps determine if an effect remains "artifact-free" or visually pleasing even if structural similarity shifts.
- **Tool**: Integration with Gemini Vision API.

## Workflow: Gold Standard Regeneration

When a change purposefully alters the visual output (e.g., updating a shader's core logic), the reference material (Gold Standard) must be updated.

1. Run the test harness and identify the failure:
   ```bash
   cargo test --test visual_mcp_tests
   ```
2. Inspect the generated artifact at `tests/artifacts/{test_name}_actual.png` and the generated `tests/artifacts/{test_name}_diff.png`.
3. If the actual image is correct, overwrite the reference:
   ```bash
   cp tests/artifacts/{test_name}_actual.png tests/reference_images/{test_name}.png
   ```
4. Commit the new reference image to version control.
