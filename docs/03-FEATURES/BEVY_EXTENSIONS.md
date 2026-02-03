# Bevy Extensions Integration

This document outlines the integration status of various Bevy ecosystem crates into MapFlow, providing enhanced 3D rendering capabilities.

## Integrated Extensions

The following extensions have been successfully integrated and are available for use in the "Bevy Scene" source type.

| Extension | Version | Status | Description | UI Availability |
| :--- | :--- | :--- | :--- | :--- |
| **[hexx](https://github.com/ManevilleF/hexx)** | `0.17` | ✅ Active | Hexagonal grid utilities. Included in the default Demo Scene. | Visible as a hexagonal grid floor in the default Bevy scene. |
| **[bevy_mod_outline](https://github.com/komadori/bevy_mod_outline)** | `0.8` | ✅ Active | Mesh outlining. Applied to the demo cube. | Visible as an orange outline around the central cube. |
| **[bevy_enoki](https://github.com/Lommix/bevy_enoki)** | `0.2` | ⚠️ Pending | Particle system. Dependency added, but code currently disabled due to API changes (`SpawnerBundle`). | **Disabled** (Code commented out in `systems.rs`). Needs API adjustment to be enabled. |
| **[bevy_rectray](https://github.com/mintlu8/bevy_rectray)** | `0.1` | ⬜ Planned | Ray casting for UI/Interaction. Compatible with Bevy 0.14. | Not yet implemented. Planned for interactive scene elements. |

## Incompatible / Skipped Extensions

The following extensions were evaluated but found incompatible or problematic with Bevy 0.14 at this time.

| Extension | Issue | Recommendation |
| :--- | :--- | :--- |
| **[bevy-vfx-bag](https://github.com/torsteingrindvik/bevy-vfx-bag)** | Stuck on Bevy 0.10. | Avoid or fork for significant rewrite. |
| **[bevy-ui-gradients](https://github.com/ickshonpe/bevy-ui-gradients)** | Incompatible with Bevy 0.14 UI rendering changes. | Use native Bevy gradients (0.17+) or MapFlow's internal UI overlay. |

## Evaluation Pending

| Extension | Status | Notes |
| :--- | :--- | :--- |
| **[bevy_smooth_pixel_camera](https://crates.io/crates/bevy_smooth_pixel_camera)** | ❓ Risk | Rated for Bevy 0.13. Might require a fork or patch. |
| **[bevy_text_animation](https://crates.io/crates/bevy_text_animation)** | ❓ Unknown | Compatibility with 0.14 needs verification. |

## How to Use Bevy features in UI

1.  **Add Bevy Source**:
    *   Right-click in the Node Canvas.
    *   Select **Add Node** -> **Sources** -> **🎮 Bevy Scene**.
2.  **View Output**:
    *   Connect the Bevy Node to an Output or view its preview.
    *   You will see the demo scene containing the properties of the enabled extensions (Hex grid, Outlined Cube).
3.  **Modify Scene**:
    *   The scene logic is located in `crates/mapmap-bevy/src/systems.rs`. Edits here reflect immediately after recompilation.

## Developer Notes
*   **Enoki**: To enable particles, update `crates/mapmap-bevy/src/systems.rs` to use correct `bevy_enoki::prelude` types for version 0.2.
*   **Repo**: Dependencies are managed in `crates/mapmap-bevy/Cargo.toml`.
