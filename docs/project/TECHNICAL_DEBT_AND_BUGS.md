# Technical Debt, Bugs and Missing UI Features

This document tracks the current state of MapFlow's implementation, identifying gaps between code and UI, known bugs, and technical debt.

## Critical Bugs (Actively Being Fixed)

### 1. Video Rendering Pipeline (Partial Fix Applied)
- **Status:** **IN PROGRESS**
- **Symptoms:** Media nodes show no content; Output projectors show magenta/pink patterns.
- **Root Cause:**
    - Decoded frames were uploaded to `TexturePool` but not synchronized with the `node_previews` map used by the UI in all cases.
    - `PaintTextureCache` still contains placeholders for loading textures from video decoders and images (`paint_texture_cache.rs`).
    - The GPU upload thread in `pipeline.rs` is currently a placeholder (Phase 1), causing uploads to happen on the logic thread.
- **Fixes Applied:**
    - Added node preview synchronization in `render.rs`.
    - Implemented 2s timeout for Hue connection to prevent startup hang.
    - Refactored `main.rs` to use `ApplicationHandler` for better event stability.

### 2. Preview Panel Empty
- **Status:** **FIXED**
- **Root Cause:** The `PreviewPanel` state was never updated with the `egui::TextureId` from the render engine's preview cache.
- **Fix:** Added logic in `render.rs` to propagate `egui_tex_id` to `app.ui_state.preview_panel`.

## Features in Code but NOT in UI

### Media & IO
- **HAP Video Support:** `hap_decoder.rs` exists in `mapmap-media` but is not selectable in the `ModuleCanvas` source node dropdown.
- **SRT Streaming:** SRT implementation in `mapmap-io` is not exposed in the UI settings or output nodes.
- **NDI Output:** While code for NDI sending exists, it is marked as a placeholder in `mapmap-io/src/ndi/mod.rs` and lacks full verification/integration.

### Rendering
- **Shader Graph:** Many nodes exist in `shader_graph.rs` (e.g., Math nodes, Oscillators) that are not yet available in the visual graph editor.
- **LUT Support:** 3D LUT loading is implemented in `mapmap-core`, but the Effect Chain UI lacks a "Load LUT" effect node.

### Control
- **Advanced MIDI Feedback:** The code supports sending MIDI messages back to controllers, but the UI only allows mapping inputs.
- **Philips Hue Grouping:** Entertainment area fetching is in the code but not fully integrated into the "Area Select" dropdown in UI.

## Significant Technical Debt (TODOs)

### Architecture
- **GPU Upload Thread:** `pipeline.rs` has a placeholder for the upload thread. Currently, uploads happen on the logic thread, causing micro-stutters during high-resolution playback.
- **wgpu v27 Lifetime Hack:** In `render.rs`, an `unsafe transmute` is used to bypass lifetime issues with `egui-wgpu` and `wgpu` v27. This needs a proper architectural fix.
- **Undo/Redo Breadth:** Currently only supports node positions. Needs expansion to all state mutations (connections, parameters).

### Features
- **Bezier Interpolation:** `animation.rs` lacks smooth Bezier easing (currently falls back to linear).
- **Mesh File Import:** `module.rs` TODO: Load mesh from OBJ/SVG files.
- **MPV Integration:** `mpv_decoder.rs` is just a shell; needs full Render API integration for fallback decoding.

## Cleanup Needed
- **Dead Code:** Multiple `#[allow(dead_code)]` markers in `mesh_editor.rs` and `window_manager.rs` indicate legacy logic from the Qt-to-egui transition that should be pruned.
- **Unused Fields:** Several structures have fields that are currently unused or purely placeholders for future phases.
