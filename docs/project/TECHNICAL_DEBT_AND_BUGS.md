# MapFlow: Technical Debt, Bugs and Roadmap Gaps

This document tracks the current state of MapFlow's implementation, identifying critical architectural issues, incomplete features, and technical debt.

---

## 🛑 Critical Architectural Issues & Hacks

| Issue | Status | Impact | File/Location |
| :--- | :--- | :--- | :--- |
| **GPU Upload Thread** | 🔴 Placeholder | UI micro-stutters during high-res playback. Uploads currently block the main logic thread. | `mapmap-media/src/pipeline.rs` |
| **wgpu Lifetime Hack** | 🔴 Unsafe Hack | Uses `unsafe transmute` to force `'static` lifetime on `RenderPass`. High risk of UB if not fixed. | `crates/mapmap/src/app/loops/render.rs` |
| **UI App Pointer Hack** | 🔴 Unsafe Hack | Uses `*mut App` raw pointer to bypass borrow checker during egui UI layout. | `crates/mapmap/src/app/loops/render.rs` |
| **Video Sync Pipeline** | 🟡 In Progress | Frames are decoded but not always synced with node previews, leading to magenta/black outputs. | `crates/mapmap/src/app/loops/render.rs` |

---

## 🎨 Feature Gaps: Code vs. UI

Features that are partially or fully implemented in the backend but lack a UI representation.

### 🎥 Media & Rendering
- **HAP Video:** `hap_decoder.rs` exists but is not selectable in the Node Editor.
- **NDI Support:** NDI Send implementation is a placeholder (`ndi/mod.rs`); no UI to configure NDI outputs.
- **Shader Graph Nodes:** Math, Oscillator, and logic nodes exist in `shader_graph.rs` but lack visual representation/wiring in the UI.
- **LUT Support:** Core supports LUT loading, but there is no "LUT Effect" node in the effect chain UI.
- **MPV Decoder:** Exists as a shell in `mapmap-media` but only generates gray placeholder frames.

### 🌐 IO & Control
- **SRT Streaming:** Connection and frame sending logic are unimplemented stubs (`srt.rs`).
- **OSC Triggers:** UI lacks an OSC input field for Cue triggers despite core support.
- **MIDI Feedback:** Backend supports sending MIDI back to devices, but UI only maps inputs.
- **Philips Hue:** Entertainment areas fetching is coded but missing from the "Area Select" dropdown.
- **Ableton Link:** Currently just a placeholder wrapper (`link.rs`).

---

## 🛠️ Significant Technical Debt (TODOs)

### 🏗️ Architecture & Core Logic
- **Undo/Redo:** Only supports node positions. Needs to cover parameters, connections, and layer mutations.
- **Bezier Interpolation:** fallback to linear/grid. Smooth Bezier easing is missing in `animation.rs` and `module.rs`.
- **Mesh Import:** `module.rs` TODO for loading meshes from OBJ/SVG files.
- **Shader Codegen:** Missing scale, rotation, and translation parameter injection.
- **Graph Validation:** `shader_graph.rs` lacks cycle detection and type-safety checks for connections.

### 🧼 Code Cleanup & Quality
- **Dead Code:** Significant amounts of legacy Qt-migration logic marked with `#[allow(dead_code)]` in `window_manager.rs` and `mesh_editor.rs`.
- **Error Handling:** Heavy use of `panic!` in production-adjacent code (e.g., `mcp/server.rs`, `converter.rs`) instead of proper `Result` handling.
- **Placeholders:** Multiple "Phase 1" placeholders in `pipeline.rs`, `lib.rs` (FFI), and `web/routes.rs`.

---

## 🐛 Known Bugs

| Bug | Status | Root Cause |
| :--- | :--- | :--- |
| **Output Magenta Patterns** | 🟡 Partial Fix | `PaintTextureCache` missing implementation for video/source loading. |
| **Media Thumbnails** | 🔴 Missing | Background thumbnail generation not implemented in `media_browser.rs`. |
| **Media Duration** | 🔴 Missing | Duration extraction is a TODO in `media_browser.rs`. |
| **Theme Switching** | 🔴 Missing | `settings.rs` lacks implementation to apply theme changes globally. |
| **Node Renaming** | 🔴 Missing | Rename/Duplicate actions in `module_sidebar.rs` are stubs. |

---

*Last Updated: 2026-02-21*
