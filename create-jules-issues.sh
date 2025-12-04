#!/bin/bash
# Script to create all 8 Jules Development Issues
# Run this with: bash create-jules-issues.sh

set -e

echo "🚀 Creating Jules Development Issues..."
echo "========================================"
echo ""

# Issue 1: Multi-Window Rendering
echo "📝 Creating Issue 1: Multi-Window Rendering..."
gh issue create \
  --title "Implement Multi-Window Rendering" \
  --label "jules-task,priority: critical,phase-2: multi-projector" \
  --body "## Multi-Window Rendering Implementation

**Phase:** Phase 2 - Multi-Projector System  
**Priority:** Critical  
**Status:** 60% complete

### Description
Complete multi-window rendering with synchronized output. This is critical for professional multi-projector setups.

### Tasks
- [ ] Implement window-per-output architecture
- [ ] Synchronize frame presentation across windows
- [ ] Handle window resize and display changes
- [ ] Test with multiple physical displays
- [ ] Performance optimization for multi-window scenarios

### Acceptance Criteria
- [ ] Multiple output windows can be created and managed
- [ ] Frame synchronization works across all outputs
- [ ] Handles display changes (connect/disconnect) gracefully
- [ ] Performance: 60fps on 2+ outputs at 1920x1080
- [ ] Tests pass for 2, 4, and 6+ output scenarios

### Technical Details
- Files: \`crates/mapmap-render/src/output.rs\`, \`crates/mapmap/src/main.rs\`
- Use wgpu for multi-window support
- Implement VSync synchronization mechanism
- Consider using separate surfaces per output

### Related Documentation
- ROADMAP.md: \"Multi-Window Rendering\" section
- docs/03-ARCHITECTURE/rendering.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 1 created!"
echo ""

# Issue 2: Frame Synchronization
echo "📝 Creating Issue 2: Frame Synchronization..."
gh issue create \
  --title "Implement Frame Synchronization" \
  --label "jules-task,priority: critical,phase-2: multi-projector" \
  --body "## Frame Synchronization Across Outputs

**Phase:** Phase 2 - Multi-Projector System  
**Priority:** Critical  
**Status:** Not started

### Description
Ensure frame-perfect synchronization across all output windows for seamless multi-projector displays.

### Tasks
- [ ] Design sync mechanism (VSync, manual sync, etc.)
- [ ] Implement frame timing system
- [ ] Add frame drop detection and recovery
- [ ] Test with 2+, 4+, 6+ outputs
- [ ] Profile performance impact

### Acceptance Criteria
- [ ] Frame-perfect sync across all outputs (±1 frame)
- [ ] Automatic frame drop detection
- [ ] Recovery mechanism for out-of-sync situations
- [ ] Performance overhead <5ms
- [ ] Works reliably with different display refresh rates

### Technical Details
- Files: \`crates/mapmap-render/src/sync.rs\` (new file)
- Consider using: frame counters, presentation timestamps
- Implement backpressure mechanism
- Add metrics for monitoring sync quality

### Related Documentation
- ROADMAP.md: \"Frame Synchronization\" section

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 2 created!"
echo ""

# Issue 3: Build System Fix
echo "📝 Creating Issue 3: Build System Fix..."
gh issue create \
  --title "Fix Build System - FreeType Linker Errors" \
  --label "jules-task,priority: high,type: infrastructure" \
  --body "## Fix Build System Dependencies

**Phase:** Infrastructure  
**Priority:** High  
**Status:** Blocking

### Description
Resolve FreeType linker errors and ensure clean multi-platform builds. This is blocking development.

### Tasks
- [ ] Fix FreeType linker configuration
- [ ] Verify all system dependencies are properly linked
- [ ] Update build documentation
- [ ] Test build on Linux, macOS, Windows
- [ ] Add CI checks for common build failures

### Acceptance Criteria
- [ ] \`cargo build\` succeeds on all platforms without manual intervention
- [ ] No linker errors for system dependencies
- [ ] Clear error messages if dependencies are missing
- [ ] Updated build documentation in README.md
- [ ] CI validates builds on all platforms

### Technical Details
- Files: \`Cargo.toml\`, \`.github/workflows/Build_Rust.yml\`
- Known issue: FreeType linking fails on some systems
- May need: pkg-config, fontconfig, freetype system packages
- Consider: vendoring dependencies or better error messages

### Current Error
\`\`\`
error: linking with \`cc\` failed
  = note: /usr/bin/ld: cannot find -lfreetype
\`\`\`

### Related Documentation
- ROADMAP.md: \"Build System\" issues
- README.md: Build instructions

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 3 created!"
echo ""

# Issue 4: Still Image Support
echo "📝 Creating Issue 4: Still Image Support..."
gh issue create \
  --title "Complete Still Image Support (PNG, JPG, TIFF)" \
  --label "jules-task,priority: high,phase-1: core-features" \
  --body "## Complete Still Image Format Support

**Phase:** Phase 1 - Core Features  
**Priority:** High  
**Status:** Partially complete

### Description
Implement comprehensive support for still image formats including PNG, JPG, and TIFF with proper memory management and caching.

### Tasks
- [ ] Complete PNG support (color profiles, transparency)
- [ ] Add JPG/JPEG support with quality settings
- [ ] Implement TIFF support (multi-page, compression)
- [ ] Add image caching layer
- [ ] Memory-efficient loading for large images
- [ ] Handle format-specific metadata

### Acceptance Criteria
- [ ] PNG: Full support including alpha transparency
- [ ] JPG: Loading and display with proper color handling
- [ ] TIFF: Basic support for single and multi-page
- [ ] Large images (>100MB) load efficiently
- [ ] Image cache reduces reload time by >80%
- [ ] Tests for all formats and edge cases

### Technical Details
- Files: \`crates/mapmap-media/src/image.rs\`
- Use: image crate or custom loaders
- Implement: LRU cache for decoded images
- Consider: Progressive loading for large images
- Memory target: <500MB for typical project

### Related Documentation
- ROADMAP.md: \"Still Image Support\" section
- docs/02-FEATURES/media-support.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 4 created!"
echo ""

# Issue 5: Animated Format Support
echo "📝 Creating Issue 5: Animated Format Support..."
gh issue create \
  --title "Add Animated Format Support (GIF, Image Sequences)" \
  --label "jules-task,priority: medium,phase-1: core-features" \
  --body "## Animated Format Support Implementation

**Phase:** Phase 1 - Core Features  
**Priority:** Medium  
**Status:** Not started

### Description
Add support for animated image formats including GIF and image sequence playback.

### Tasks
- [ ] Implement GIF decoder with frame timing
- [ ] Add image sequence support (PNG/JPG sequences)
- [ ] Frame timing and playback control
- [ ] Memory-efficient frame caching
- [ ] Loop control (once, loop, bounce)

### Acceptance Criteria
- [ ] GIF: Full playback with correct timing
- [ ] Image sequences: Automatic detection and playback
- [ ] Frame rate control (1-60 fps)
- [ ] Smooth playback without frame drops
- [ ] Memory usage <200MB for typical sequences
- [ ] Tests for various formats and frame rates

### Technical Details
- Files: \`crates/mapmap-media/src/animated.rs\` (new file)
- GIF: Use gif crate or custom decoder
- Sequences: Pattern matching (frame_0001.png, etc.)
- Implement: Frame buffer and timing system
- Consider: Pre-loading next N frames

### Related Documentation
- ROADMAP.md: \"Animated Formats\" section
- docs/02-FEATURES/media-support.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 5 created!"
echo ""

# Issue 6: ProRes Codec Support
echo "📝 Creating Issue 6: ProRes Codec Support..."
gh issue create \
  --title "Add ProRes Codec Support" \
  --label "jules-task,priority: medium,phase-1: core-features" \
  --body "## ProRes Codec Support

**Phase:** Phase 1 - Core Features  
**Priority:** Medium  
**Status:** Not started

### Description
Add support for Apple ProRes codec variants, commonly used in professional video production.

### Tasks
- [ ] Integrate FFmpeg ProRes decoder
- [ ] Support ProRes variants (Proxy, LT, 422, HQ, 4444, 4444 XQ)
- [ ] Implement efficient frame decoding
- [ ] Add codec detection and metadata parsing
- [ ] Performance benchmarking and optimization

### Acceptance Criteria
- [ ] All ProRes variants decode correctly
- [ ] Performance: Real-time playback at 1920x1080 60fps
- [ ] Proper color space handling
- [ ] Alpha channel support for 4444/4444 XQ
- [ ] Memory usage optimized
- [ ] Tests for all variants and resolutions

### Technical Details
- Files: \`crates/mapmap-media/src/codecs/prores.rs\` (new file)
- Use: FFmpeg with ProRes decoder
- Binding: ffmpeg-next or custom FFI
- Consider: Hardware acceleration where available
- Test files: Use standard ProRes test sequences

### Related Documentation
- ROADMAP.md: \"Video Codec Support\" section
- docs/02-FEATURES/video-codecs.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 6 created!"
echo ""

# Issue 7: Advanced Geometric Correction
echo "📝 Creating Issue 7: Advanced Geometric Correction..."
gh issue create \
  --title "Advanced Geometric Correction Tools" \
  --label "jules-task,priority: medium,phase-2: multi-projector" \
  --body "## Advanced Geometric Correction Implementation

**Phase:** Phase 2 - Multi-Projector System  
**Priority:** Medium  
**Status:** 30% complete

### Description
Implement advanced geometric correction and warping tools for professional projection mapping.

### Tasks
- [ ] Keystone correction UI and calculations
- [ ] Grid-based mesh warping
- [ ] Bezier curve warping interface
- [ ] Save/load warp presets
- [ ] Real-time preview during adjustment

### Acceptance Criteria
- [ ] Keystone: 4-point and 8-point correction
- [ ] Mesh warp: Adjustable grid (4x4 to 16x16)
- [ ] Performance: <5ms warp computation at 1920x1080
- [ ] Presets: Save/load/share warp configurations
- [ ] UI: Intuitive visual editor
- [ ] Tests for warp calculations and edge cases

### Technical Details
- Files: \`crates/mapmap-render/src/warp.rs\`, \`crates/mapmap-ui/src/warp_editor.rs\`
- Algorithms: Perspective transformation, mesh interpolation
- GPU compute: Use compute shaders for real-time warping
- Format: JSON for preset storage

### Related Documentation
- ROADMAP.md: \"Geometric Correction\" section
- docs/02-FEATURES/warping.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 7 created!"
echo ""

# Issue 8: Output Configuration Persistence
echo "📝 Creating Issue 8: Output Configuration Persistence..."
gh issue create \
  --title "Implement Output Configuration Persistence" \
  --label "jules-task,priority: medium,phase-2: multi-projector" \
  --body "## Output Configuration Save/Load System

**Phase:** Phase 2 - Multi-Projector System  
**Priority:** Medium  
**Status:** Not started

### Description
Implement project file format to save and load complete output configurations including mappings, warps, and settings.

### Tasks
- [ ] Design project file format (JSON/TOML)
- [ ] Implement serialization for all config types
- [ ] Add save/load UI
- [ ] Version migration support
- [ ] Auto-save functionality
- [ ] Recovery from corrupted files

### Acceptance Criteria
- [ ] Save complete project state to file
- [ ] Load restores exact configuration
- [ ] Format supports future extensions
- [ ] Version migration from v1.0 format
- [ ] Auto-save every 5 minutes (optional)
- [ ] Clear error messages for corrupt files
- [ ] Tests for save/load round-trips

### Technical Details
- Files: \`crates/mapmap/src/project.rs\`, \`crates/mapmap-config/src/serialization.rs\`
- Format: JSON or TOML (JSON recommended for size)
- Schema: Version field for migration
- Consider: Incremental saves, compression
- Include: Outputs, surfaces, warps, media paths

### File Format Example
\`\`\`json
{
  \"version\": \"1.0\",
  \"created\": \"2024-12-04T12:00:00Z\",
  \"outputs\": [...],
  \"surfaces\": [...],
  \"warps\": [...]
}
\`\`\`

### Related Documentation
- ROADMAP.md: \"Project Files\" section
- docs/02-FEATURES/project-files.md

---
*Issue for Jules AI Agent - Created via script*"

echo "✅ Issue 8 created!"
echo ""

echo "========================================"
echo "✨ All 8 Jules Issues created successfully!"
echo ""
echo "View issues: gh issue list --label 'jules-task'"
echo "Or visit: https://github.com/MrLongNight/VjMapper/issues"
echo ""
echo "🤖 Jules can now start working on these tasks!"
