# Scribe Journal

## Current State (2026-01-20)
- **Roadmap Sync**: Updated `ROADMAP.md` to match the single source of truth `ROADMAP_2.0.md` (v2.0, Stand 2026-01-20). This resolves the confusion between the two files.
- **Documentation Coverage**:
  - Enabled `#![warn(missing_docs)]` in `crates/mapmap-control/src/lib.rs`.
  - Started systematically fixing missing documentation warnings in `mapmap-control`.
  - Fixed docs for `error.rs`, `manager.rs`, `midi/mod.rs`, `midi/clock.rs`, `midi/controller_element.rs`, and `midi/mapping.rs`.
  - Reduced warnings from 367 to 301.
  - Remaining areas in `mapmap-control` needing docs: `midi/profiles.rs`, `midi/midi_learn.rs`, `dmx/*`, `hue/*`, `cue/*`, `shortcuts/*`.

## Current State (2026-01-14)
- **Cleanup**: Verified that `CHANGELOG.md` (root) is the superset of `docs/08-CHANGELOG/CHANGELOG.md` (which was stale, missing entries from 2026-01-13 and 2026-01-14). Safely deleted the stale duplicate to ensure a Single Source of Truth.
- **Crate Docs**: Verified `README.md` and `lib.rs` for all major crates (`mapmap-ui`, `mapmap-render`, `mapmap-control`, `mapmap-media`, `mapmap-io`, `mapmap-core`). They are in excellent shape.
- **Rustdoc Coverage**:
  - Enabled `#![warn(missing_docs)]` in `mapmap-core`.
  - Documented the public API in `lib.rs` (e.g., `Vertex`, `Quad`, `Project`).
  - Documented `shader_graph.rs` (Node system, Enums, Structs) to resolve compiler warnings.
- **Pattern Observation**: The `docs/XX-TOPIC/README.md` pattern linking to root documents works well.

## Next Steps
- Continue adding Rustdoc to remaining modules in `mapmap-control` (`dmx`, `hue`, `cue`, `shortcuts`).
- Enable `#![warn(missing_docs)]` in `mapmap-ui` and address missing docs there (large task).
