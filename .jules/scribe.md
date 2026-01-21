# Scribe Journal

## 2026-01-09 - Discrepancy in Documentation Structure

I noticed a significant discrepancy between the documentation structure described in typical "Inventory" prompts/templates (listing `01-OVERVIEW`, `04-API`) and the actual filesystem structure (`01-GETTING-STARTED`, `04-USER-GUIDE`).

### Observations
- The prompt templates often cite `docs/01-OVERVIEW/` as the first item.
- The actual repository has `docs/01-GETTING-STARTED/`.
- `docs/INDEX.md` correctly reflects the filesystem.
- This mismatch can lead to confusion during automated reviews or when referencing documentation paths in new files.

### Decision
- I have chosen to link to the **actual filesystem paths** (`01-GETTING-STARTED`) in `crates/mapmap/README.md` to avoid broken links.
- I am documenting this here to clarify why the implementation might appear to diverge from the prompt's "inventory".

## 2026-01-08 - Journal Start

## Current State (2026-01-14)
- **Cleanup**: Verified that `CHANGELOG.md` (root) is the superset of `docs/08-CHANGELOG/CHANGELOG.md` (which was stale, missing entries from 2026-01-13 and 2026-01-14). Safely deleted the stale duplicate to ensure a Single Source of Truth.
- **Crate Docs**: Verified `README.md` and `lib.rs` for all major crates (`mapmap-ui`, `mapmap-render`, `mapmap-control`, `mapmap-media`, `mapmap-io`, `mapmap-core`). They are in excellent shape.
- **Rustdoc Coverage**:
  - Enabled `#![warn(missing_docs)]` in `mapmap-core`.
  - Documented the public API in `lib.rs` (e.g., `Vertex`, `Quad`, `Project`).
  - Documented `shader_graph.rs` (Node system, Enums, Structs) to resolve compiler warnings.
- **Pattern Observation**: The `docs/XX-TOPIC/README.md` pattern linking to root documents works well.

## Next Steps
- Continue adding Rustdoc to other inner modules of `mapmap-core` (e.g. `layer.rs`, `mapping.rs`).
- Verify `ROADMAP.md` matches the actual code implementation for "In Progress" features.
