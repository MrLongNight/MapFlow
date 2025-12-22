# Project Phases (0-7)

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

The project is currently in **Phase 6: Advanced UI (egui)**.

## Phase 6: Advanced UI (egui Migration)

The goal of this phase is to migrate the legacy ImGui interface to a professional, node-based authoring environment using `egui`.

### Migration Status

- [x] **Dashboard Controls** (Quick-access parameters, `dashboard.rs`)
- [x] **Media Browser** (Asset management, `media_browser.rs`)
- [x] **Mesh Editor** (Projection mapping mesh editing, `mesh_editor.rs`)
- [x] **Node Editor** (Visual programming, `node_editor.rs`)
- [x] **Timeline V2** (Keyframe animation, `timeline_v2.rs`)
- [x] **Theming** (Custom styling, `theme.rs`)

### Pending Migration (Legacy ImGui Components)

The following components are still using ImGui (found in `crates/mapmap-ui/src/lib.rs`) and need to be rewritten in `egui`:

- [ ] **Layer Manager** (`render_layer_panel`)
- [ ] **Paint Manager** (`render_paint_panel`)
- [ ] **Mapping Manager** (`render_mapping_panel`)
- [ ] **Transform Controls** (`render_transform_panel`)
- [ ] **Output Configuration** (`render_output_panel`)
- [ ] **Edge Blend & Color Calibration** (`render_edge_blend_panel`, `render_color_calibration_panel`)
- [ ] **Audio Visualization** (`render_audio_panel`)
- [ ] **Oscillator Control** (`render_oscillator_panel`)
- [ ] **Main Menu & Toolbar** (`render_menu_bar`, `render_controls`)
- [ ] **Shader Graph Editor** (`shader_graph_editor.rs`)

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
