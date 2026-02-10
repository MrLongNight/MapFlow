# Project Phases (0-7)

> **⚠️ DEPRECATED:** This document is kept for historical reference only.
> For the most up-to-date roadmap and feature status, please verify [ROADMAP.md](../../ROADMAP.md) in the project root.

This document outlines the complete project roadmap for the MapFlow Rust rewrite, from the initial core engine development to the final polish and release.

## Phase Overview

*   **Phase 0: Foundation & Core Engine** - Setting up the Rust project, winit, and the basic rendering pipeline.
*   **Phase 1: Multi-Projector Support** - Implementing output management for multiple projectors.
*   **Phase 2: Effects & Shaders** - Building the WGSL shader system and a library of effects.
*   **Phase 3: Control & Integration** - OSC/MIDI control, FFmpeg integration for media playback.
*   **Phase 4: Asset & Preset Management** - Creating a library for effects, transforms, and project templates.
*   **Phase 5: Advanced UI (ImGui)** - Initial UI implementation using ImGui for core controls.
*   **Phase 6: Advanced UI (egui)** - A complete UI overhaul using the egui framework for a professional authoring experience, including a node editor, timeline, and asset browser.
*   **Phase 7: Performance, Polish & Release** - Profiling, stress testing, bug fixing, and preparation for the v1.0 release.

---

## Current Status

The project is currently in **Phase 7: Advanced Show Control**.

✅ **Phase 6: Advanced UI (egui Migration) – COMPLETED** (2025-12-23)

## Phase 6: Advanced UI (egui Migration) – ✅ COMPLETED

The goal of this phase was to migrate the legacy ImGui interface to a professional, node-based authoring environment using `egui`.

### Migration Status – ALL COMPLETED ✅

- [x] **Dashboard Controls** (Quick-access parameters, `dashboard.rs`)
- [x] **Media Browser** (Asset management, `media_browser.rs`)
- [x] **Mesh Editor** (Projection mapping mesh editing, `mesh_editor.rs`)
- [x] **Node Editor** (Visual programming, `node_editor.rs`)
- [x] **Timeline V2** (Keyframe animation, `timeline_v2.rs`)
- [x] **Theming** (Custom styling, `theme.rs`)
- [x] **Layer Manager** (`layer_panel.rs`) – COMPLETED 2025-12-22
- [x] **Paint Manager** (`paint_panel.rs`) – COMPLETED 2025-12-22
- [x] **Mapping Manager** (`mapping_panel.rs`) – COMPLETED 2025-12-23 (PR #97)
- [x] **Transform Controls** (`transform_panel.rs`) – COMPLETED 2025-12-22
- [x] **Output Configuration** (`output_panel.rs`) – COMPLETED 2025-12-23
- [x] **Edge Blend & Color Calibration** (`edge_blend_panel.rs`) – COMPLETED 2025-12-23
- [x] **Audio Visualization** (`audio_panel.rs`) – COMPLETED 2025-12-22
- [x] **Oscillator Control** (`oscillator_panel.rs`) – COMPLETED 2025-12-23
- [x] **Main Menu & Toolbar** (`menu_bar.rs`) – COMPLETED 2025-12-22
- [x] **Shader Graph Editor** (`node_editor.rs`) – COMPLETED 2025-12-23
- [x] **OSC Panel** (`osc_panel.rs`) – COMPLETED 2025-12-23
- [x] **Cue Panel** (`cue_panel.rs`) – COMPLETED 2025-12-23
- [x] **ImGui Removal** (Code Cleanup) – COMPLETED 2025-12-23

### Remaining UI Tasks (Phase 6.5)

- [ ] **Docking Layout & Unified Inspector**
- [x] **Icon System** (Streamline Ultimate Integration) - Integrated in Dashboard & Media Browser
- [ ] **All UI Strings for i18n** (Extract and translate)

---

## Phase 7: Performance, Polish & Release

### Packaging & Distribution

- [x] **App Icon Embedding**
  - Uses `winres` to embed `mapmap.ico` into the Windows executable.
- [ ] **Windows Installer (WiX)**
  - Basic configuration (`main.wxs`) exists.
  - Needs verification of DLL bundling (FFmpeg) and shortcut creation.
- [ ] **Linux Packaging (.deb)**
  - Needs `cargo-deb` configuration in `Cargo.toml` or `debian/` control files.
- [ ] **AppImage / Flatpak** (Optional)
  - Evaluate for broader Linux compatibility.

---

## Phase 8: Multi-PC Architecture (NEW)

> **Detailed Documentation:** [`docs/03-ARCHITECTURE/MULTI-PC-FEASIBILITY.md`](../03-ARCHITECTURE/MULTI-PC-FEASIBILITY.md)

This phase enables distributed output across multiple PCs, supporting professional multi-projector installations.

### Phase Overview

| Sub-Phase | Option | Description | Duration |
|-----------|--------|-------------|----------|
| **8.1** | Option A: NDI Streaming | Video streaming via NDI protocol | 3 weeks |
| **8.2** | Option C: Legacy Client | H.264/RTSP for old hardware | 2 weeks |
| **8.3** | Option D: Raspberry Pi | ARM64 budget player | 1-2 weeks |
| **8.4** | Option B: Distributed Rendering | Multi-GPU cluster rendering | 5-6 weeks |

### 8.1 Option A: NDI Video Streaming (Recommended)

The master PC renders all content and streams the finished video to player clients.

- [ ] **NDI Integration** (`mapmap-ndi/`)
  - [ ] Create new crate `mapmap-ndi`
  - [ ] Integrate `grafton-ndi` Rust bindings
  - [ ] Implement NDI Sender (wgpu Texture → NDI)
  - [ ] Implement NDI Receiver (NDI → Fullscreen)
  - [ ] Multi-source discovery (NDI Finder)
  - [ ] Latency optimization (<100ms target)

- [ ] **Player Mode** (`--player-ndi`)
  - [ ] Refactor `main.rs` for multi-mode support
  - [ ] Headless player without Editor UI
  - [ ] Auto-connect to master source
  - [ ] Fullscreen rendering on selected output
  - [ ] Optional status overlay

- [ ] **Installer Updates**
  - [ ] Add "MapFlow Player (NDI)" shortcut
  - [ ] NDI Runtime dependency check

### 8.2 Option C: Legacy Slave Client

For very old hardware (2010+ era), using hardware-accelerated H.264 decoding.

- [ ] **H.264/RTSP Streaming** (`mapmap-legacy/`)
  - [ ] Create new crate `mapmap-legacy`
  - [ ] H.264 Encoder (x264 software / NvEnc hardware)
  - [ ] RTSP Server for stream distribution
  - [ ] Hardware decoder support (DXVA, VA-API, VideoToolbox)
  - [ ] SDL2-based fullscreen player

- [ ] **Player Mode** (`--player-legacy`)
  - [ ] Minimal dependencies (no wgpu required)
  - [ ] FFmpeg hardware decoding
  - [ ] Configurable stream URL

### 8.3 Option D: Raspberry Pi Player (Optional)

Budget-friendly player using Raspberry Pi hardware.

- [ ] **ARM64 Cross-Compilation**
  - [ ] Set up `aarch64-unknown-linux-gnu` target
  - [ ] Configure cross-compilation toolchain
  - [ ] Create CI/CD pipeline for ARM64 builds

- [ ] **Software Options**
  - [ ] Document Dicaffeine NDI Player setup
  - [ ] Custom ARM64 MapFlow build (optional)
  - [ ] VLC RTSP fallback

- [ ] **Deployment**
  - [ ] Raspberry Pi OS Image (pre-configured)
  - [ ] Systemd auto-start service
  - [ ] Read-only filesystem (optional)

### 8.4 Option B: Distributed Rendering (Future)

Clients render independently, receiving only control commands.

- [ ] **Control Protocol** (`mapmap-sync/`)
  - [ ] OSC-based control messaging
  - [ ] Timecode synchronization (NTP-based)
  - [ ] Frame-sync via hardware genlock (optional)
  - [ ] Asset distribution (NFS/S3)

- [ ] **Distributed Render Client**
  - [ ] Local wgpu rendering
  - [ ] Scene replication from master
  - [ ] Independent resolution per client

### Hardware Requirements Summary

| Role | Option A (NDI) | Option B (Dist) | Option C (Legacy) | Option D (Pi) |
|------|----------------|-----------------|-------------------|---------------|
| **Master CPU** | 8+ cores | 4+ cores | 8+ cores | N/A |
| **Master GPU** | RTX 3060+ | Any | RTX 3060+ | N/A |
| **Client CPU** | 4+ cores | 8+ cores | Dual-Core | Pi 4/5 |
| **Client GPU** | Intel HD 4000+ | RTX 3060+ | Intel HD 2000+ | VideoCore VI |
| **Network** | Gigabit | Gigabit | 100 Mbps | Gigabit |

### Success Criteria

- [ ] **Option A MVP**: NDI stream from MapFlow to second PC, fullscreen display, <100ms latency
- [ ] **Option C MVP**: RTSP stream to Intel HD 2000 PC, 1080p30 playback
- [ ] **Option D MVP**: Raspberry Pi 4 playing 720p60 NDI stream via Dicaffeine
- [ ] **Option B MVP**: Two PCs rendering synchronized content via timecode
