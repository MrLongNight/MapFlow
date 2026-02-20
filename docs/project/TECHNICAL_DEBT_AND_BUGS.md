# Technical Debt, Bugs and Missing UI Features

This document tracks the current state of MapFlow's implementation, identifying gaps between code and UI, known bugs, and technical debt.

## Critical Bugs

### 1. Video Rendering Pipeline
- **Status:** **FIXED** (2026-02-20)
- **Symptoms:** Media nodes show no content; Output projectors show magenta/pink patterns.
- **Root Cause:** 
    - Decoded frames were uploaded to `TexturePool` but not synchronized with the `node_previews` map used by the UI.
    - Synchronous uploads on logic thread caused micro-stutters.
- **Solution:**
    - Implemented `FramePipeline` with background decoding and GPU upload threads.
    - Added node preview synchronization in `render.rs`.
    - Integrated asynchronous pipeline handles into the main render loop.

### 2. Preview Panel Empty
- **Status:** **FIXED** (2026-02-20)
- **Root Cause:** The `PreviewPanel` state was never updated with the `egui::TextureId` from the render engine's preview cache.
- **Fix:** Added logic in `render.rs` to propagate `egui_tex_id` to `app.ui_state.preview_panel`.

## Features in Code but NOT in UI

### Media & IO
- **HAP Video Support:** `hap_decoder.rs` exists in `mapmap-media` but is not selectable in the `ModuleCanvas` source node dropdown.
- **NDI Output:** The core supports NDI sending, but there is no UI toggle to enable NDI broadcast for a specific output projector.
- **Spout/Syphon:** Code structure exists, but UI integration for selecting Spout senders is minimal/placeholders.
- **SRT Streaming:** SRT implementation in `mapmap-io` is not exposed in the UI settings or output nodes.

### Rendering
- **Shader Graph:** Many nodes exist in `shader_graph.rs` (e.g., Math nodes, Oscillators) that are not yet available in the visual graph editor.
- **LUT Support:** 3D LUT loading is implemented in `mapmap-core`, but the Effect Chain UI lacks a "Load LUT" effect node.

### Control
- **Advanced MIDI Feedback:** The code supports sending MIDI messages back to controllers, but the UI only allows mapping inputs.
- **Philips Hue Grouping:** Entertainment area fetching is in the code but not fully integrated into the "Area Select" dropdown in UI.

## Significant Technical Debt (TODOs)

### Architecture
- **VRAM Management:** The `TexturePool` lacks an automated garbage collection for unused textures (e.g., from deleted nodes).
- **Undo/Redo Breadth:** Currently only supports node positions. Needs expansion to all state mutations (connections, parameters).

### Features
- **Bezier Interpolation:** `animation.rs` lacks smooth Bezier easing.
- **Mesh File Import:** `module.rs` TODO: Load mesh from OBJ/SVG files.
- **MPV Integration:** `mpv_decoder.rs` is just a shell; needs full Render API integration for fallback decoding.

## Cleanup Completed
- **Asynchronous GPU Pipeline:** IMPLEMENTED. Background threads now handle decode and upload.
- **sysinfo redundancy:** Removed `new_all()` calls which were causing startup lags.
- **Windows Startup:** Fixed Hue connection timeout and Bevy asset watcher hangs.
