# Technical Debt, Bugs and Missing UI Features

This document tracks the current state of MapFlow's implementation, identifying gaps between code and UI, known bugs, and technical debt.

## Fixed Issues (2026-02-20)

### 1. Video Rendering Pipeline
- **Status:** **FIXED**
- **Fix:** Implemented `FramePipeline` with background decoding and GPU upload threads. Synchronized `node_previews` and `PreviewPanel` in `render.rs`.

### 2. Preview Panel Empty
- **Status:** **FIXED**
- **Fix:** Propagated `egui_tex_id` from render cache to `app.ui_state.preview_panel`.

### 3. Windows Startup Performance
- **Status:** **FIXED**
- **Fix:** 
    - Disabled Bevy asset watcher (prevented startup hang).
    - Implemented 2s timeout for Philips Hue connection.
    - Removed `sysinfo::System::new_all()` (expensive call).

### 4. GPU Upload Thread
- **Status:** **FIXED**
- **Fix:** Moved texture uploads to dedicated background threads in `mapmap-media`.

### 5. VRAM Management (Garbage Collection)
- **Status:** **FIXED**
- **Fix:** Implemented time-based Garbage Collection in `TexturePool`. Textures not used for 30s are automatically freed.

## Features in Code but NOT in UI (Next Priorities)

### Media & IO
- [ ] **HAP Video Support:** `hap_decoder.rs` exists in `mapmap-media` but is not selectable in the `ModuleCanvas` source node dropdown. (Jules working on this)
- [ ] **NDI Output:** The core supports NDI sending, but there is no UI toggle to enable NDI broadcast for a specific output projector. (Jules working on this)
- [ ] **Spout/Syphon:** Code structure exists, but UI integration for selecting Spout senders is minimal/placeholders.
- [ ] **SRT Streaming:** SRT implementation in `mapmap-io` is not exposed in the UI settings or output nodes.

### Rendering
- [ ] **Shader Graph:** Many nodes exist in `shader_graph.rs` (e.g., Math nodes, Oscillators) that are not yet available in the visual graph editor.
- [ ] **LUT Support:** 3D LUT loading is implemented in `mapmap-core`, but the Effect Chain UI lacks a "Load LUT" effect node. (Jules working on this)

### Control
- [ ] **Advanced MIDI Feedback:** The code supports sending MIDI messages back to controllers, but the UI only allows mapping inputs.
- [ ] **Philips Hue Grouping:** Entertainment area fetching is in the code but not fully integrated into the "Area Select" dropdown in UI.

## Significant Technical Debt (TODOs)

### Architecture
- [ ] **Undo/Redo Breadth:** Currently only supports node positions. Needs expansion to all state mutations (connections, parameters).

### Features
- [ ] **Bezier Interpolation:** `animation.rs` lacks smooth Bezier easing.
- [ ] **Mesh File Import:** `module.rs` TODO: Load mesh from OBJ/SVG files.
- [ ] **MPV Integration:** `mpv_decoder.rs` is just a shell; needs full Render API integration for fallback decoding.

## Cleanup Needed
- [ ] **Dead Code:** Multiple `#[allow(dead_code)]` markers in `window_manager.rs` and `lib.rs` need review.
