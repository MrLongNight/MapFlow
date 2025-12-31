# Implementation Plan: Assignment System

> **Status:** PROPOSED
> **Priority:** HIGH (Core Feature)
> **Dependencies:** `mapmap-core`, `mapmap-ui`, `mapmap-control`

---

## 🎯 Objective
Implement a centralized **Assignment System** to map generic control sources (MIDI, OSC, etc.) to application parameters (Layer Opacity, Effect Params, etc.), replacing ad-hoc bindings.

## 📋 Scope

1.  **Core Data Structures**: Define `Assignment`, `ControlSource`, `ControlTarget`.
2.  **Assignment Manager**: Logic to store, manage, and process assignments.
3.  **UI Panel**: Interface to view, create, and edit assignments (filtering, learning).
4.  **Integration**: Updates in the main loop to apply control values to targets.

---

## 🏗️ Architecture

### 1. Data Structures (`mapmap-core/src/assignment.rs`)

```rust
pub struct Assignment {
    pub id: Uuid,
    pub active: bool,
    pub source: ControlSource,
    pub target: ControlTarget,
    pub transform: ValueTransform, // Invert, Min/Max, Curve
}

pub enum ControlSource {
    Midi { channel: Option<u8>, cc: u8 }, // Channel optional = Omni
    Osc { address: String, arg_index: usize },
    // DMX, Gamepad, etc. later
}

pub enum ControlTarget {
    LayerOpacity(LayerId),
    LayerTransform(LayerId, TransformProperty), // PosX, PosY, Scale...
    EffectParameter(LayerId, usize, String), // Layer, EffectIndex, ParamName
    MasterOpacity,
    // ...
}
```

### 2. Logic (`mapmap-core/src/assignment/manager.rs`)

- `AssignmentManager` struct holding a `Vec<Assignment>`.
- `process(&self, input_state: &InputState) -> Vec<ParameterUpdate>`
- Methods: `add()`, `remove()`, `find_by_source()`, `find_by_target()`.

### 3. UI (`mapmap-ui/src/assignment_panel.rs`)

- **Table View**: List all assignments (Source icon -> Target name).
- **Controls**:
    - "Learn" toggle (waits for next MIDI/OSC input).
    - Source/Target manual selectors (Dropdowns/Drag&Drop).
    - Value range sliders.
    - Delete button.

---

## 📅 Implementation Steps

### Phase 1: Core Definitions (Day 1)
- [ ] Create `mapmap-core/src/assignment/` module.
- [ ] Implement `ControlSource` and `ControlTarget` enums.
- [ ] Implement `Assignment` struct with serialization (serde).
- [ ] Register module in `lib.rs`.

### Phase 2: Manager & Integration (Day 2)
- [ ] Implement `AssignmentManager`.
- [ ] Add `assignment_manager` to `AppState`.
- [ ] Integrate into `main.rs` update loop (read inputs -> update params).

### Phase 3: UI Implementation (Day 3)
- [ ] Create `AssignmentPanel` in `egui`.
- [ ] Implement listing and checking active state.
- [ ] Implement "Add" functionality (Basic).

### Phase 4: Learn & Refine (Day 4)
- [ ] Implement "MIDI Learn" logic (global learn mode).
- [ ] Add filtering/sorting to the panel.
- [ ] Persist assignments in Project File.

---

## 🔗 References
- Issue #128 (Assignment System)
- `mapmap-control::midi`
- `mapmap-core::parameter`
