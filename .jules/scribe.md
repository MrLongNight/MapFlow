# Scribe Journal

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
