# UI Architecture

## Canvas System (Phase 4 Refactor)

The `ModuleCanvas` is the central component for the visual programming interface.
It has been refactored to remove dependencies on external node editor libraries (`egui_node_editor`)
in favor of a custom, lightweight implementation tailored to MapFlow's needs.

### Key Components

- **Canvas Types**: Located in `crates/mapmap-ui/src/canvas/types.rs`.
  - `MediaPlaybackCommand`: Shared enum for media control.
  - `MediaPlayerInfo`: Shared struct for playback state.
- **Trigger System**: Located in `crates/mapmap-core/src/trigger_system.rs`.
  - Handles logic for `AudioFFT`, `Random`, `Fixed`, and `Beat` triggers.
  - Maintains persistent state for timers and random intervals.

### Module Structure

The UI is built using `egui` and follows a retained-mode style where the `ModuleCanvas`
struct holds the state of interaction (panning, selection, dragging) while the `MapFlowModule`
(from core) holds the data model.
