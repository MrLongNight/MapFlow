# Refactoring Plan: Phase 4 - The "God Class" Dismantling

This plan targets the two largest files in the codebase, `module_canvas.rs` and `module.rs`, which have accumulated excessive responsibility.

## 🎯 Objectives

1.  **Reduce `module_canvas.rs`** (>6700 lines) by extracting node-specific UI logic into a sub-module structure.
2.  **Reduce `module.rs`** (>3000 lines) by separating data models from the graph management logic.
3.  **Improve Compilation Times**: Smaller files allow for better incremental compilation.
4.  **Enhance Navigability**: Developers (and AI) can find relevant code faster.

## 🤖 Jules Task List (Total: 8 Tasks)

To minimize risk and manage complexity effectively, we will break this down into **8 distinct Jules Tasks**. Each task is self-contained and verifiable.

### Component A: `module.rs` Decomposition (3 Tasks)

#### [Task 4.1.1] Module Scaffolding & Trigger Extraction
- **Files**: `crates/mapmap-core/src/module.rs` -> `crates/mapmap-core/src/module/mod.rs` + `trigger.rs`
- **Output**:
    - Directory `crates/mapmap-core/src/module/` created.
    - File `crates/mapmap-core/src/module.rs` moved to `mod.rs`.
    - `TriggerConfig`, `TriggerTarget`, `TriggerMappingMode` moved to `module/trigger.rs`.
    - `mod.rs` re-exports everything publicly.
- **Verification**: `cargo check -p mapmap-core`

#### [Task 4.1.2] Extract ModulePart
- **Input**: `crates/mapmap-core/src/module/mod.rs`
- **Output**:
    - `ModulePart` struct and its impl blocks moved to `crates/mapmap-core/src/module/part.rs`.
    - `mod.rs` imports and re-exports `ModulePart`.
- **Verification**: `cargo check -p mapmap-core`, `cargo test -p mapmap-core`

#### [Task 4.1.3] Graph Logic Consolidation
- **Input**: `crates/mapmap-core/src/module/mod.rs`
- **Output**:
    - Rename/Structure remaining logic in `mod.rs` (the actual `MapFlowModule` graph) to ensure it's clean.
    - (Optional) Move graph logic to `graph.rs` if `mod.rs` is still large, keeping `mod.rs` as just an entry point.
- **Verification**: `cargo check -p mapmap-core`, `cargo test -p mapmap-core`

---

### Component B: `module_canvas.rs` Decomposition (5 Tasks)

#### [Task 4.2.1] Canvas Scaffolding & Types
- **Files**: `crates/mapmap-ui/src/editors/module_canvas.rs` -> `crates/mapmap-ui/src/editors/module_canvas/mod.rs` + `types.rs`
- **Output**:
    - Directory `crates/mapmap-ui/src/editors/module_canvas/` created.
    - File moved to `mod.rs`.
    - `MyDataType`, `MyValueType`, `MyResponse`, `MyNodeTemplate` moved to `types.rs`.
- **Verification**: `cargo check -p mapmap-ui`

#### [Task 4.2.2] Extract Node Registry & Base Traits
- **Input**: `crates/mapmap-ui/src/editors/module_canvas/mod.rs`
- **Output**:
    - Create `nodes/mod.rs`.
    - Define traits or common logic for node UI rendering if applicable.
    - Ensure `mod.rs` can access `nodes` module.
- **Verification**: `cargo check -p mapmap-ui`

#### [Task 4.2.3] Extract Media Node UI
- **Input**: `crates/mapmap-ui/src/editors/module_canvas/mod.rs`
- **Output**:
    - Move all `MediaNode` specific UI logic (drawing the node, inputs/outputs) to `nodes/media.rs`.
    - Update `mod.rs` `bottom_ui` match arm to delegate to `nodes::media::render(...)`.
- **Verification**: `cargo check -p mapmap-ui`, Manual check of Media Nodes.

#### [Task 4.2.4] Extract Effect Node UI
- **Input**: `crates/mapmap-ui/src/editors/module_canvas/mod.rs`
- **Output**:
    - Move all `EffectNode` specific UI logic to `nodes/effect.rs`.
    - Update `mod.rs` delegate.
- **Verification**: `cargo check -p mapmap-ui`, Manual check of Effect Nodes.

#### [Task 4.2.5] Extract Remaining Nodes (Trigger, Layer, Output)
- **Input**: `crates/mapmap-ui/src/editors/module_canvas/mod.rs`
- **Output**:
    - Move `TriggerNode` to `nodes/trigger.rs`.
    - Move `LayerNode` to `nodes/layer.rs`.
    - Move `OutputNode` to `nodes/output.rs`.
    - Finalize `mod.rs` to have minimal logic, mostly delegation.
- **Verification**: `cargo check -p mapmap-ui`, Full manual regression test.

## 🛡️ Execution Protocol

1.  **Sequential Execution**: Tasks 4.1.x can run in parallel with 4.2.x, but sub-tasks (e.g., 4.2.1 -> 4.2.2) must be sequential.
2.  **Commit per Task**: Each task should result in a clean, compiling commit.
