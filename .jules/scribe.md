# Scribe Journal

## Current State (2026-01-20)
- **Roadmap Sync**: Updated `ROADMAP.md` to match the single source of truth `ROADMAP_2.0.md` (v2.0, Stand 2026-01-20). This resolves the confusion between the two files.
- **Documentation Coverage**:
  - Enabled `#![warn(missing_docs)]` in `crates/mapmap-control/src/lib.rs`.
  - Started systematically fixing missing documentation warnings in `mapmap-control`.
  - Fixed docs for `error.rs`, `manager.rs`, `midi/mod.rs`, `midi/clock.rs`, `midi/controller_element.rs`, and `midi/mapping.rs`.
  - Reduced warnings from 367 to 301.
  - Remaining areas in `mapmap-control` needing docs: `midi/profiles.rs`, `midi/midi_learn.rs`, `dmx/*`, `hue/*`, `cue/*`, `shortcuts/*`.
# Scribe Journal - 2026-01-20

## Documentation Cleanup & Restructuring

I have undertaken a cleanup of the documentation structure to resolve conflicts and align with the project standards.

### Actions Taken
- **Resolved Conflicts**: Fixed merge conflicts in `.jules/scribe.md` and `crates/mapmap-mcp/README.md`.
- **Restructured Docs**:
  - Renamed `docs/04-USER-GUIDE` to `docs/02-USER-GUIDE` to resolve the numbering conflict with `04-API`.
  - Moved `docs/02-CONTRIBUTING/CONTRIBUTING.md` and `CODE-OF-CONDUCT.md` to the project root.
  - Renamed `docs/02-CONTRIBUTING/` to `docs/05-DEVELOPMENT/` to serve as the Developer Guide.
  - Renumbered remaining folders (`05-ROADMAP` -> `06`, `06-TUTORIALS` -> `07`, `07-TECHNICAL` -> `08`) to ensure a sequential order without gaps or duplicates.
  - Removed `docs/08-CHANGELOG/` to enforce `CHANGELOG.md` in the root as the Single Source of Truth.
  - Updated `docs/INDEX.md` to reflect the new structure.
- **Synced Roadmap**: Updated `ROADMAP.md` to match `ROADMAP_2.0.md`.

---

# Scribe Journal - 2026-01-09

## Discrepancy in Documentation Structure

I noticed a significant discrepancy between the documentation structure described in typical "Inventory" prompts/templates (listing `01-OVERVIEW`, `04-API`) and the actual filesystem structure (`01-GETTING-STARTED`, `04-USER-GUIDE`).

### Observations
- The prompt templates often cite `docs/01-OVERVIEW/` as the first item.
- The actual repository has `docs/01-GETTING-STARTED/`.
- `docs/INDEX.md` correctly reflects the filesystem.
- This mismatch can lead to confusion during automated reviews or when referencing documentation paths in new files.

### Decision
- I have chosen to link to the **actual filesystem paths** (`01-GETTING-STARTED`) in `crates/mapmap/README.md` to avoid broken links.
- I am documenting this here to clarify why the implementation might appear to diverge from the prompt's "inventory".

---

# Scribe Journal - 2026-01-08

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
