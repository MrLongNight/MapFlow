# VjMapper Project Roadmap

> **Complete Development Plan and Open Items Tracking**

This document provides a comprehensive overview of VjMapper's development roadmap, current status, and all open points organized logically and chronologically.

## Quick Status Overview

**Current Phase:** Phase 2 (Professional Multi-Projector System) - 85% Complete  
**Latest Milestone:** Foundation & Core Engine Complete  
**Next Major Milestone:** Multi-Output Rendering Implementation

---

## Table of Contents

1. [Project Vision](#project-vision)
2. [Development Phases Overview](#development-phases-overview)
3. [Current Status (Phase 0-2)](#current-status-phase-0-2)
4. [Open Items by Phase](#open-items-by-phase)
5. [Phase 3-7 Roadmap](#phase-3-7-roadmap)
6. [Technical Debt & Known Issues](#technical-debt--known-issues)
7. [Success Metrics](#success-metrics)

---

## Project Vision

VjMapper is a professional-grade, open-source projection mapping system built in Rust, designed to provide powerful projection mapping capabilities without the cost of commercial software. The project aims to compete with commercial solutions like Resolume Arena while remaining open-source and community-driven.

**Key Goals:**
- Professional multi-projector support with edge blending
- Real-time video playback with hardware acceleration
- GPU-accelerated effects and shader system
- MIDI/OSC/DMX control integration
- Professional video I/O (NDI, DeckLink, Spout, Syphon)
- Modern, intuitive user interface

---

## Development Phases Overview

| Phase | Focus | Status | Completion |
|-------|-------|--------|------------|
| **Phase 0** | Foundation & Setup | ✅ Complete | 100% |
| **Phase 1** | Core Engine | ✅ Complete | 86% (Mostly Done) |
| **Phase 2** | Multi-Projector System | 🚧 In Progress | 85% |
| **Phase 3** | Effects Pipeline | 📋 Planned | 0% |
| **Phase 4** | Control Systems | 📋 Planned | 0% |
| **Phase 5** | Professional Video I/O | 📋 Planned | 0% |
| **Phase 6** | Advanced UI | 📋 Planned | 0% |
| **Phase 7** | Polish & Release | 📋 Planned | 0% |

---

## Current Status (Phase 0-2)

### ✅ Phase 0: Foundation (100% Complete)

**Completed Items:**
- ✅ Modern graphics via wgpu (Vulkan/Metal/DX12)
- ✅ Safe, high-performance Rust implementation
- ✅ ImGui-based live operator interface
- ✅ Modular architecture with 7 specialized crates
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ Comprehensive CI/CD pipeline
- ✅ Project structure and build system
- ✅ Basic rendering pipeline
- ✅ Window management with winit

### 🚧 Phase 1: Core Engine (86% Complete)

**Completed Items:**
- ✅ Layer system with transforms, opacity, and blend modes
- ✅ Advanced playback controls (backwards, ping-pong, play once)
- ✅ Master controls for speed and opacity
- ✅ Quick resize modes (Fill, Fit, Stretch, Original)
- ✅ Hardware-accelerated rendering
- ✅ Multi-threaded media pipeline
- ✅ Professional ImGui control interface
- ✅ Performance monitoring and real-time stats

**Open Items:**
- 🚧 Still image support (PNG, JPG, TIFF) - Partial implementation
- 🚧 Animated format support (GIF, image sequences) - Partial implementation
- 🚧 ProRes codec support - Not started
- 🚧 Hardware-accelerated video decode optimization - Needs work

### 🚧 Phase 2: Multi-Projector System (85% Complete)

**Completed Items:**
- ✅ Bezier-based mesh warping system
- ✅ Edge blending and color calibration shaders
- ✅ Monitor detection and output management foundation
- ✅ UI panels for multi-output configuration
- ✅ GPU post-processing pipeline
- ✅ Canvas region filtering
- ✅ 2x2 projector array preset

**Open Items:**
- 🚧 Multi-window rendering implementation - Core work needed
- 🚧 Frame synchronization across outputs - Not started
- 🚧 Advanced geometric correction - Partial implementation
- 🚧 Per-output warping and mapping - Needs refinement
- 🚧 Output management persistence - Save/load not implemented

---

## Open Items by Phase

### Priority 1: Critical Path Items (Phase 2 Completion)

#### Multi-Window Rendering
- **Status:** 🚧 In Progress (60% done)
- **Owner:** Unassigned
- **Priority:** Critical
- **Description:** Complete multi-window rendering with synchronized output
- **Tasks:**
  - [ ] Implement window-per-output architecture
  - [ ] Synchronize frame presentation across windows
  - [ ] Handle window resize and display changes
  - [ ] Test with multiple physical displays
  - [ ] Performance optimization for multi-window scenarios

#### Frame Synchronization
- **Status:** 📋 Not Started
- **Owner:** Unassigned
- **Priority:** Critical
- **Description:** Ensure frame-perfect sync across all output windows
- **Tasks:**
  - [ ] Design sync mechanism (VSync, manual sync, etc.)
  - [ ] Implement frame timing system
  - [ ] Add frame drop detection and recovery
  - [ ] Test with 2+, 4+, 6+ outputs
  - [ ] Profile performance impact

#### Build System Issues
- **Status:** 🚧 Failing
- **Owner:** Unassigned
- **Priority:** High
- **Description:** Fix Build_Rust GitHub Actions workflow
- **Tasks:**
  - [x] Add missing system dependencies (fontconfig, freetype, X11, FFmpeg)
  - [ ] Fix FreeType linker errors (FT_Set_Default_Properties)
  - [ ] Verify build on Ubuntu 20.04+
  - [ ] Test on other platforms (macOS, Windows)
  - [ ] Document build requirements

### Priority 2: Phase 1 Polish Items

#### Still Image Support
- **Status:** 🚧 Partial
- **Owner:** Unassigned
- **Priority:** High
- **Description:** Complete support for PNG, JPG, TIFF still images
- **Tasks:**
  - [ ] Image loading and caching system
  - [ ] Texture upload optimization
  - [ ] Memory management for large images
  - [ ] Format conversion pipeline
  - [ ] UI integration for image properties

#### Animated Format Support
- **Status:** 🚧 Partial
- **Owner:** Unassigned
- **Priority:** Medium
- **Description:** Support GIF and image sequences
- **Tasks:**
  - [ ] GIF decoder integration
  - [ ] Image sequence loader
  - [ ] Frame timing and playback
  - [ ] Memory-efficient buffering
  - [ ] UI controls for animation playback

#### ProRes Codec Support
- **Status:** 📋 Not Started
- **Owner:** Unassigned
- **Priority:** Medium
- **Description:** Add professional ProRes codec support
- **Tasks:**
  - [ ] Research FFmpeg ProRes support
  - [ ] Implement decoder
  - [ ] Test with various ProRes variants
  - [ ] Performance benchmarking
  - [ ] Documentation

### Priority 3: Phase 2 Enhancements

#### Advanced Geometric Correction
- **Status:** 🚧 Partial
- **Owner:** Unassigned
- **Priority:** Medium
- **Description:** Enhanced mesh warping and correction tools
- **Tasks:**
  - [ ] Keystone correction UI
  - [ ] Grid-based warping
  - [ ] Corner pinning improvements
  - [ ] Save/load warp presets
  - [ ] Per-output warp configuration

#### Output Management Persistence
- **Status:** 📋 Not Started
- **Owner:** Unassigned
- **Priority:** Medium
- **Description:** Save and load output configurations
- **Tasks:**
  - [ ] Project file format design
  - [ ] Serialization of output settings
  - [ ] Deserialization and validation
  - [ ] Migration from older formats
  - [ ] UI for project management

---

## Phase 3-7 Roadmap

### Phase 3: Effects Pipeline (Months 10-12) - 📋 Planned

**Goals:**
- GPU compute effects system
- Audio reactivity
- Custom shader support
- Effect library

**Key Features:**
- Shader graph system with visual editor
- Parameter animation
- Audio-reactive effects (FFT, beat detection)
- LUT color grading
- Effect presets and library

**Success Metrics:**
- 10+ built-in effects
- <5ms effect processing latency
- Real-time audio reactivity
- Custom shader hot-reload

### Phase 4: Control Systems (Months 13-15) - 📋 Planned

**Goals:**
- MIDI input/output
- OSC server/client
- DMX output (Art-Net/sACN)
- Web API and WebSocket
- Cue system

**Key Features:**
- MIDI device auto-detection and mapping
- OSC pattern matching and routing
- DMX universe management
- WebSocket for remote control
- Cue list with transitions
- Keyboard shortcuts and macros

**Success Metrics:**
- <16ms MIDI/OSC latency
- 4+ DMX universes at 44Hz
- Web interface with low latency
- Cue system with smooth transitions

### Phase 5: Professional Video I/O (Months 16-18) - 📋 Planned

**Goals:**
- NDI integration
- DeckLink SDI support
- Spout/Syphon texture sharing
- Professional streaming

**Key Features:**
- NDI sender/receiver
- DeckLink capture and playback
- Spout (Windows) and Syphon (macOS) support
- Virtual camera output
- Professional streaming protocols

**Success Metrics:**
- NDI at full resolution and framerate
- 4K SDI input/output
- Zero-copy texture sharing
- Low-latency streaming

### Phase 6: Advanced UI (Months 19-21) - 📋 Planned

**Goals:**
- Modern UI framework
- Node editor
- Timeline editor
- Asset browser

**Key Features:**
- Migrate from ImGui to egui
- Visual node-based editor
- Multi-track timeline
- Asset library with previews
- Dockable panels
- Dark/light themes

**Success Metrics:**
- Fluid 60fps UI
- Intuitive workflow
- Professional appearance
- Accessibility features

### Phase 7: Performance, Polish & Release (Months 22-24) - 📋 Planned

**Goals:**
- Performance optimization
- Bug fixing
- Documentation
- v1.0 release preparation

**Key Features:**
- Profiling and optimization
- Stress testing with real-world scenarios
- Complete user documentation
- Video tutorials
- Installer/packaging for all platforms
- Website and marketing materials

**Success Metrics:**
- 4K @ 60fps multi-output
- <50ms total system latency
- Zero critical bugs
- Complete documentation
- Positive user feedback

---

## Technical Debt & Known Issues

### Build System
- ❗ **CRITICAL:** Build_Rust workflow failing due to FreeType linker errors
- ❗ **CRITICAL:** Missing system dependencies in CI/CD
- ⚠️ Warning: ashpd v0.8.1 future incompatibility with Rust 2024

### Performance
- ⚠️ Video decode pipeline optimization needed
- ⚠️ Memory usage monitoring and optimization
- ⚠️ GPU memory management improvements

### Code Quality
- ⚠️ Test coverage needs improvement (<50% currently)
- ⚠️ Documentation gaps in some crates
- ⚠️ Some clippy warnings need addressing

### Features
- 🐛 Still image support incomplete
- 🐛 GIF animation not working
- 🐛 ProRes codec not supported
- 🐛 Multi-window sync not implemented

---

## Success Metrics

### Performance Targets

**Achieved:**
- ✅ 60 fps @ 1920x1080 per output (VSync locked)
- ✅ <1ms texture upload for 1920x1080 RGBA
- ✅ <50ms frame latency
- ✅ Multi-output rendering with synchronized presentation
- ✅ Real-time edge blending and color calibration (GPU-accelerated)

**Remaining:**
- 🎯 4K @ 60 fps with hardware decode
- 🎯 10+ concurrent video streams
- 🎯 <16ms control latency (MIDI/OSC)
- 🎯 NDI/Spout/Syphon integration

### Quality Targets

**Current:**
- ⚠️ Build: Failing (linker errors)
- ✅ Core functionality: Working
- ⚠️ Test coverage: ~40%
- ⚠️ Documentation: ~60% complete

**Goals:**
- 🎯 Build: Passing on all platforms
- 🎯 Test coverage: >80%
- 🎯 Documentation: 100% complete
- 🎯 Zero critical bugs
- 🎯 Performance targets met

---

## Next Immediate Actions

### This Week
1. **Fix Build_Rust workflow** (Critical)
   - Add all required system dependencies
   - Fix FreeType linking issue
   - Test on multiple platforms

2. **Complete Multi-Window Rendering** (Critical)
   - Implement window-per-output
   - Add frame synchronization
   - Test with physical displays

3. **Update Documentation** (High)
   - Fix BUILD.md references
   - Update CONTRIBUTING.md
   - Create this ROADMAP.md

### This Month (Phase 2 Completion)
1. Complete multi-window rendering
2. Implement frame synchronization
3. Polish edge blending and color calibration
4. Add output configuration save/load
5. Comprehensive testing with real projectors
6. Document Phase 2 features

### Next Month (Phase 3 Start)
1. Design shader graph system
2. Implement audio analysis pipeline
3. Create initial effect library
4. Begin custom shader support

---

## Timeline

```
Phase 0: Foundation           [████████████████████] 100% Complete
Phase 1: Core Engine         [███████████████░░░░░]  86% Complete  
Phase 2: Multi-Projector     [█████████████░░░░░░░]  85% Complete (In Progress)
Phase 3: Effects Pipeline    [░░░░░░░░░░░░░░░░░░░░]   0% Planned
Phase 4: Control Systems     [░░░░░░░░░░░░░░░░░░░░]   0% Planned
Phase 5: Pro Video I/O       [░░░░░░░░░░░░░░░░░░░░]   0% Planned
Phase 6: Advanced UI         [░░░░░░░░░░░░░░░░░░░░]   0% Planned
Phase 7: Polish & Release    [░░░░░░░░░░░░░░░░░░░░]   0% Planned
```

**Estimated Completion:**
- Phase 2: ~2 weeks (December 2024)
- Phase 3: Q1 2025 (3 months)
- Phase 4: Q2 2025 (3 months)
- Phase 5: Q3 2025 (3 months)
- Phase 6: Q4 2025 (3 months)
- Phase 7: Q1 2026 (3 months)
- **v1.0 Release Target:** March 2026

---

## Contributing

Want to help? Check out our [open items](#open-items-by-phase) and pick something to work on!

- 🔴 Critical items need immediate attention
- 🟡 High priority items are important for phase completion
- 🟢 Medium priority items can be done in parallel

See [CONTRIBUTING.md](docs/02-CONTRIBUTING/CONTRIBUTING.md) for guidelines.

---

## Questions or Suggestions?

Open an issue at https://github.com/MrLongNight/VjMapper/issues

---

**Last Updated:** 2024-12-04  
**Document Version:** 1.0  
**Project Version:** 0.1.0 (Phase 2)
