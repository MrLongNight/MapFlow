# Technical Debt, Bugs and Missing UI Features

This document tracks the current state of MapFlow's implementation, identifying gaps between code and UI, known bugs, and technical debt.

## Critical Bugs (Actively Being Fixed)

### 1. Video Rendering Pipeline (Partial Fix Applied)
- **Status:** **IN PROGRESS**
- **Symptoms:** Media nodes show no content; Output projectors show magenta/pink patterns.
- **Root Cause:** 
    - Decoded frames were uploaded to `TexturePool` but not synchronized with the `node_previews` map used by the UI.
    - `wgpu` texture format mismatch between decoder (RGBA) and surface (BGRA) handled incorrectly in some places.
    - `PaintTextureCache` default to test patterns instead of waiting for decoder frames.
- **Fixes Applied:**
    - Added node preview synchronization in `render.rs`.
    - Added detailed logging in `media.rs`.
    - Implemented 2s timeout for Hue connection to prevent startup hang.

### 2. Preview Panel Empty
- **Status:** **FIXED**
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
- **GPU Upload Thread:** `pipeline.rs` has a placeholder for the upload thread. Currently, uploads happen on the logic thread, causing micro-stutters during high-resolution playback.
- **VRAM Management:** The `TexturePool` lacks an automated garbage collection for unused textures (e.g., from deleted nodes).
- **Undo/Redo Breadth:** Currently only supports node positions. Needs expansion to all state mutations (connections, parameters).

### Features
- **Bezier Interpolation:** `animation.rs` lacks smooth Bezier easing.
- **Mesh File Import:** `module.rs` TODO: Load mesh from OBJ/SVG files.
- **MPV Integration:** `mpv_decoder.rs` is just a shell; needs full Render API integration for fallback decoding.

## Cleanup Needed
- **sysinfo redundancy:** Removed `new_all()` calls which were causing startup lags.
- **Dead Code:** Multiple `#[allow(dead_code)]` markers in `window_manager.rs` and `lib.rs` need review.
