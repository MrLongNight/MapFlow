# MapFlow: Technical Debt, Bugs and Roadmap Gaps

This document tracks the current state of MapFlow's implementation, identifying critical architectural issues, incomplete features, and technical debt.

---

## 🛑 Critical Architectural Issues & Hacks

| Issue | Status | Impact | File/Location |
| :--- | :--- | :--- | :--- |
| **"God Object" module_canvas** | 🟡 In Progress | `module_canvas/mod.rs` has grown to **~7,000 lines**. Refactoring started: Splitting into draw, interaction, integration, and state. | `crates/mapmap-ui/src/editors/module_canvas/mod.rs` |
| **Monolithic core/module.rs** | 🟡 In Progress | `module.rs` contains over **3,500 lines**. Refactoring started: Splitting into types, config, and manager. | `crates/mapmap-core/src/module.rs` |
| **GPU Upload Blockage** | 🔴 Critical | The `FramePipeline` in `mapmap-media` is not yet integrated into the main `update_media_players` loop. Currently, texture uploads happen on the main thread, causing micro-stutters. | `crates/mapmap/src/orchestration/media.rs` |
| **wgpu Lifetime Hack** | 🔴 Unsafe Hack | Uses `unsafe transmute` to force `'static` lifetime on `RenderPass`. High risk of UB. | `crates/mapmap/src/app/loops/render.rs` |
| **UI App Pointer Hack** | 🔴 Unsafe Hack | Uses `*mut App` raw pointer to bypass the Rust borrow checker during egui UI layout. | `crates/mapmap/src/app/loops/render.rs` |

---

## 🏗️ Refactoring Strategy (Architecture Audit 2026-02-25)

### 1. module_canvas Decomposition
- **Goal:** Split into logical sub-modules.
- **Status:** Directory structure prepared. Logic separation identified.
- **Modules:** `draw.rs`, `interaction.rs`, `integration/`, `state.rs`.

### 2. core/module.rs Splitting
- **Goal:** Separate data definitions from graph logic.
- **Status:** Initial `types.rs` and `manager.rs` separation planned.
- **Modules:** `types/`, `config.rs`, `manager.rs`.

---

## 🎨 Feature Gaps: Code vs. UI (Updated)

- **Bevy Node Controls:** UI labels indicate controls for Bevy 3D/Particles nodes are "not yet implemented".
- **HAP Video Alpha:** Alpha support is hardcoded to `None` in `hap_decoder.rs`.
- **NDI Support:** Send implementation is a placeholder; no UI configuration.
- **Shader Graph Nodes:** Core supports logic nodes, but UI lacks visual representation/wiring.
- **LUT Support:** No "LUT Effect" node in the effect chain UI despite core support.
- **MPV Decoder:** Shell exists, but actual API integration is missing (generates gray frames).
- **SRT Streaming:** Missing `libsrt` integration; connection/sending logic are just stubs.
- **OSC Triggers:** UI lacks OSC input field for Cue triggers.
- **Philips Hue:** Pairing logic and Area Selection fetching are missing.

---

## 🛠️ Significant Technical Debt (TODOs)

### 🏗️ Architecture & Core Logic
- **Undo/Redo Coverage:** Currently only node positions. Needs to cover parameters, connections, and layer mutations.
- **Trigger Smoothing:** `TriggerMappingMode::Smoothed` (attack/release) is a TODO in `module.rs`.
- **Individual Layer Speed:** Returns master speed; individual control not yet implemented in `layer.rs`.
- **Mesh Import:** Missing core logic for loading meshes from file (OBJ/SVG) in `module.rs`.
- **Shader Codegen:** Missing scale, rotation, and translation parameter injection.
- **Graph Validation:** `shader_graph.rs` lacks cycle detection and type-safety checks.
- **MCP Shared State:** MCP server cannot yet read/access the shared project state.
- **Spout Sync:** Spout output is missing from the main synchronization loop in `orchestration/outputs.rs`.

### 🧼 Code Cleanup & Quality
- **Panic Policy:** Replace `panic!` with `Result` in `mapmap-mcp/server.rs`, `web/handlers.rs`, and MIDI mapping.
- **Safety Documentation:** Every `unsafe` block must have a `// SAFETY:` comment (especially in FFmpeg/NDI/Spout).
- **Dead Code:** Significant amounts of legacy Qt-migration logic in `window_manager.rs` and `mesh_editor.rs`.
- **Media Thumbnails:** Background thumbnail generation and duration extraction missing in `media_browser.rs`.

---

## 🐛 Known Bugs

| Bug | Status | Root Cause |
| :--- | :--- | :--- |
| **Output Magenta Patterns** | 🟡 Partial Fix | `PaintTextureCache` falls back to test pattern because real loading is missing. |
| **NDI Buffer Padding** | 🟡 In Progress | Hardcoded 256-byte alignment in NDI readback may fail on some hardware. |
| **Theme Switching** | 🔴 Missing | Global theme application requires restart; `settings.rs` implementation missing. |
| **Node Renaming** | ✅ Fixed | Rename/Duplicate actions enabled in `module_sidebar.rs`. |

---

*Last Updated: 2026-02-25 (by ClawMaster PM 🦀)*
