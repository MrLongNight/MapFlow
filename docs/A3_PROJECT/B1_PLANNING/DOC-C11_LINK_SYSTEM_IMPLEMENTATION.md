# Universal Link System Implementation Plan

This document details the implementation of the Universal Link System for Layers, Masks, Effects, and Blends in the Module Canvas, as well as the Audio Trigger Node enhancements.

## 1. Audio Trigger Node Enhancements

### Goal
Allow each output channel of the Audio Trigger Node to be optionally inverted (Logic NOT).

### Implementation Details
- **File**: `crates/mapmap-core/src/module.rs`
- **Struct**: `AudioTriggerOutputConfig`
- **Changes**:
  - Add `inverted_outputs: HashMap<String, bool>` or a generic boolean flags structure.
  - Since outputs are dynamic, we might need a robust way to store this.
  - Alternative: Add `inverted` flag to the signal generation logic, but the config needs to store user preference.
  - **Proposed Change**: Add `pub inverted_bands: [bool; 9]` and `pub inverted_volume: [bool; 2]` etc., or a simpler `inverted_signals: HashSet<String>` storing names of inverted sockets.
- **UI**:
  - Update `AudioFFT` panel to show a small "Invert" checkbox next to each output toggle.

---

## 2. Universal Link System

### Goal
Enable Master/Slave linking between nodes to synchronize visibility/activity states.

### Core Concepts

#### Link Mode
Property per node determining its role:
- **Off**: Standard behavior.
- **Master**: Controls other nodes. Exposes `Link Output`. Accepts `Trigger Input` to control itself.
- **Slave**: Controlled by a Master. Exposes `Link Input`.

#### Link Behavior (Slave only)
- **Same As Master**: Slave visibility = Master visibility.
- **Inverted**: Slave visibility = !Master visibility.

### Data Structures

**File**: `crates/mapmap-core/src/module.rs`

New Enums:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkMode {
    Off,
    Master,
    Slave,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkBehavior {
    SameAsMaster,
    Inverted,
}
```

**Common Link Data** (to be added to relevant Node structs/enums):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeLinkData {
    pub mode: LinkMode,
    pub behavior: LinkBehavior,
    pub trigger_input_enabled: bool, // For the optional Trigger Input
}
```

### Affected Nodes

1.  **Layer Nodes** (`LayerAssignmentType`)
    - `SingleLayer`, `Group`, `AllLayers` variants will receive `link_data: NodeLinkData`.
2.  **Mask Nodes** (`MaskType`)
    - `File`, `Shape`, `Gradient` variants will receive `link_data`.
3.  **Effect/Blend Nodes** (`ModulizerType`)
    - `Effect`, `BlendMode` variants will receive `link_data`.

### Socket Logic (`ModulePart::add_part` / `generate_outputs`)

- **Trigger Input**:
  - If `link_data.trigger_input_enabled` is true (or always for Master), add `Trigger In` socket.
- **Link Output** (Master):
  - Add `Link Out` socket (Type: `Trigger` or new `Link` type? `Trigger` type `f32` is sufficient: 1.0 = Active, 0.0 = Inactive).
- **Link Input** (Slave):
  - Add `Link In` socket.

### Connection Rules
- `Link Out` can only connect to `Link In`.
- Need to distinguish "Control/Link" triggers from generic "Trigger" signals if we use the same socket type.
- **Proposal**: Use `ModuleSocketType::Link` to ensure strict typing (Master -> Slave only).

### Runtime Logic

In `mapmap-core/src/module_eval.rs` or `mapmap-ui/src/module_canvas.rs`:
- **Master Evaluation**:
  - Read `Trigger Input` value.
  - Determine `is_active` state.
  - Set Node visibility/bypass based on `is_active`.
  - Write `is_active` (1.0 or 0.0) to `Link Out`.
- **Slave Evaluation**:
  - Read `Link In` value.
  - Apply `LinkBehavior` (Invert if needed).
  - Set Node visibility/bypass.
  - Ignore local `Trigger Input` if acting as Slave? Or combine? (Usually Slave ignores local control).

---

## 3. Implementation Steps

1.  **Update Core Data Structures** (`crates/mapmap-core/src/module.rs`)
    - Define `LinkMode`, `LinkBehavior`, `NodeLinkData`.
    - Update `LayerAssignmentType`, `MaskType`, `ModulizerType`.
    - Update `AudioTriggerOutputConfig` for inversion.
    - Add `ModuleSocketType::Link`? Or reuse Trigger with naming convention? -> **Decision: Add `ModuleSocketType::Link` for safety.**

2.  **Update Module Construction** (`crates/mapmap-core/src/module.rs`)
    - Update `add_part` to initialize default `link_data`.
    - Implement `update_sockets` logic to dynamically show/hide Link/Trigger sockets based on Mode.

3.  **Update UI Panels** (`crates/mapmap-ui/src/module_canvas/panels/`)
    - Add Inspector controls for Link Mode/Behavior.
    - Add "Invert" checkboxes for Audio Trigger.

4.  **Update Runtime/Evaluation**
    - Ensure logical propagation of signals.

## 4. Work Packages

- **WP1**: Audio Trigger Inversion (Standalone)
- **WP2**: Core Struct Updates (LinkMode, SocketType)
- **WP3**: Layer Node Linking
- **WP4**: Mask/Effect/Blend Node Linking
- **WP5**: UI Integration
