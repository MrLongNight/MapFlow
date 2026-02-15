# Refactoring Plan: `main.rs` Decomposition & Architecture

This document serves as the master plan for refactoring the MapMap application. It is broken down into specific, actionable
"Jules Tasks" that are self-contained and verifiable.

## 🎯 Strategic Goals

1. **Modularization**: Eliminate the monolithic `main.rs` (>2000 lines).
2. **Separation of Concerns**: UI, Logic, and State should be in distinct crates or modules.
3. **Testability**: Smaller modules allow for unit testing.
4. **Maintainability**: Easier navigation for developers.

---

## 🤖 Jules Task List

### Phase 1: UI Modularization (High Priority)

*Goal: Extract immediate mode UI code out of the main loop.*

#### [Task 1.1] Setup UI Module & Extract Settings

- **Objective**: Create the foundational `ui` module structure and move the "Settings" window.
- **Input**: `crates/mapmap/src/main.rs` (Settings window block)
- **Actions**:
  1. Create `crates/mapmap/src/ui/mod.rs` and `crates/mapmap/src/ui/settings.rs`.
  2. Define `pub fn show(ctx: &egui::Context, state: &mut AppState, ...)` in `settings.rs`.
  3. Move the `egui::Window::new("Settings")` logic from `main.rs` to this function.
  4. Update `main.rs` to import `mod ui` and call `ui::settings::show()`.
- **Validation**:
  - `cargo check -p mapmap` passes.
  - Settings window opens and functions identical to before.

#### [Task 1.2] Extract Sidebar & Master Controls

- **Objective**: Move the left sidebar (containing Master Gain, BPM, Audio Viz) to a dedicated module.
- **Input**: `crates/mapmap/src/main.rs` (`egui::SidePanel::left`)
- **Actions**:
  1. Create `crates/mapmap/src/ui/sidebar.rs`.
  2. Extract the `SidePanel::left` logic into a public render function.
  3. Pass necessary state (AudioContext, ProjectState) as arguments.
  4. Replace the block in `main.rs` with the function call.
- **Validation**: Sidebar functions correctly (Gain slider, BPM toggle work).

#### [Task 1.3] Extract Timeline & Bottom Panel

- **Objective**: Isolate the Timeline/Sequencer UI.
- **Input**: `crates/mapmap/src/main.rs` (`egui::BottomPanel`)
- **Actions**:
  1. Create `crates/mapmap/src/ui/timeline.rs`.
  2. Move `BottomPanel` logic.
  3. Ensure `Transport` controls (Play/Pause) are correctly wired to state.
- **Validation**: Timeline renders at bottom, playback controls update state.

#### [Task 1.4] Extract Module Canvas (Center Panel)

- **Objective**: Move the main workspace (Node Graph/Canvas) logic.
- **Input**: `crates/mapmap/src/main.rs` (`egui::CentralPanel`)
- **Actions**:
  1. Create `crates/mapmap/src/ui/canvas.rs`.
  2. Move `CentralPanel` logic.
  3. Crucial: Ensure `Module` iteration and rendering logic is preserved or passed in cleanly.
- **Validation**: Nodes appear on canvas, interactions (drag/drop) still work.

---

### Phase 2: Core State & App Logic

*Goal: Slim down `App` struct and `main()` function.*

#### [Task 2.1] Define AppState Struct

- **Objective**: Separate state definition from the application runner.
- **Input**: `crates/mapmap/src/main.rs` (`struct App`)
- **Actions**:
  1. Create `crates/mapmap/src/state.rs`.
  2. Move `struct App` (rename to `AppState`?) and its fields here.
  3. Create a separate `App` wrapper in `main.rs` that holds `AppState` and implements `winit::ApplicationHandler`.
- **Validation**: Application compiles, state checks work.

#### [Task 2.2] Extract Event Handling

- **Objective**: Move massive match statements for Winit events out of `main.rs`.
- **Input**: `crates/mapmap/src/main.rs` (`impl ApplicationHandler for App`)
- **Actions**:
  1. Create `crates/mapmap/src/inputs.rs`.
  2. Create functions like `handle_keyboard_input`, `handle_mouse_input`.
  3. Call these functions from the main event loop.
- **Validation**: Keyboard shortcuts and mouse input work as expected.

#### [Task 2.3] Systems Extraction (Audio/Physics)

- **Objective**: Move update logic (audio processing, physics steps) to "Systems".
- **Input**: `crates/mapmap/src/main.rs` (`App::update` method)
- **Actions**:
  1. Create `crates/mapmap/src/systems/audio.rs` and `crates/mapmap/src/systems/physics.rs`.
  2. Move the respective logic blocks from `App::update` to these modules.
- **Validation**: Audio analysis updates visuals, physics simulations run.

---

### Phase 3: Rendering Pipeline

*Goal: Decouple WGPU logic.*

#### [Task 3.1] Extract Renderer

- **Objective**: Move raw WGPU calls (render pass, encoder) to a renderer struct.
- **Input**: `crates/mapmap/src/main.rs` (`App::render` WGPU sections)
- **Actions**:
  1. Create `crates/mapmap/src/renderer.rs`.
  2. Define `struct MapFlowRenderer` holding Device, Queue, Surface.
  3. Move initialization and `render_frame` logic here.
- **Validation**: Graphics render correctly, `main.rs` contains minimal WGPU code.

---

## 🛠️ Execution Protocol for Jules

1. **Checkout**: Create a fresh branch per task (e.g., `refactor/task-1.1-settings`).
2. **Execute**: Follow "Actions" strictly.
3. **Verify**: Run `cargo check` and `cargo test` (if applicable).
4. **Confirm**: User manual verification of UI functionality.
5. **Merge**: Squash merge to keep history clean.
