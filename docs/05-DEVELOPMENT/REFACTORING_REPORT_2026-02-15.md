# Refactoring Status Report (2026-02-15)

## 1. Executive Summary

The previous refactoring efforts to reduce `main.rs` were successful, reducing it from >2000 lines to ~1500 lines. However, the complexity has shifted to new areas, creating new "God Classes" that require immediate attention.

**Conclusion:** Refactoring is **STRONGLY RECOMMENDED**.

## 2. Identified Hotspots (Top 5)

| Metric | File path | Lines | Status |
| :--- | :--- | :--- | :--- |
| **Critical** | `crates/mapmap-ui/src/editors/module_canvas.rs` | **6,730** | **URGENT** |
| High | `crates/mapmap-core/src/module.rs` | 3,099 | Needs review |
| Medium | `crates/mapmap-core/src/module_eval.rs` | 1,695 | Manageable |
| Medium | `crates/mapmap/src/main.rs` | 1,524 | Improving |
| Medium | `crates/mapmap-ui/src/panels/effect_chain_panel.rs` | 1,379 | Watch list |

## 3. Analysis of Critical Files

### 3.1 `module_canvas.rs` (6730 lines)
This file likely absorbed all the logic extracted from `main.rs` related to the Node Graph. It currently violates the Single Responsibility Principle by mixing:
- **Data Models**: Structs like `MyNodeData`, `MyDataType`.
- **UI Rendering**: `bottom_ui` and other rendering functions.
- **Interaction Logic**: Handling connections, deletions, node building.
- **Node Implementations**: Specific logic for Media, Effects, etc.

**Recommendation**: Split into a `module_canvas` sub-crate or directory.
- `src/editors/canvas/types.rs`: Data definitions.
- `src/editors/canvas/nodes/`: Individual node logic (e.g. `media_node.rs`, `effect_node.rs`).
- `src/editors/canvas/ui.rs`: Rendering helpers.

### 3.2 `module.rs` (3099 lines)
This core file handles the graph data structure.
**Recommendation**: Extract sub-components.
- `src/module/part.rs`: `ModulePart` and definitions.
- `src/module/graph.rs`: The `MapFlowModule` graph logic.
- `src/module/trigger.rs`: `TriggerConfig` and related logic.

## 4. Proposed Action Plan (Phase 4)

We should treat this as "Phase 4" of the existing refactoring strategy (or a new independent plan).

### [Task 4.1] Decompose `module_canvas.rs`
1.  Create directory `crates/mapmap-ui/src/editors/graph/`.
2.  Extract `MyNodeData` and enums to `types.rs`.
3.  Extract node-specific UI logic to `nodes/mod.rs`.
4.  Keep high-level editor logic in `lib.rs` (or `mod.rs`) of that folder.

### [Task 4.2] Decompose `module.rs`
1.  Split `crates/mapmap-core/src/module.rs` into a folder module.
2.  Separate data structures from logic.

---
**Status**: Waiting for user approval to proceed with creating `docs/05-DEVELOPMENT/REFACTORING_PLAN_PHASE_4.md`.
