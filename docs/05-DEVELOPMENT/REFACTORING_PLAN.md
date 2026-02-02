# Refactoring Plan: `main.rs` Decomposition

This document outlines the strategy for decomposing the monolithic `crates/mapmap/src/main.rs` file into modular, maintainable components. The goal is to separate concerns (UI, Logic, Rendering, State) and adhere to Clean Code principles.

## 1. Analysis of Current State (`main.rs`)

Currently, `main.rs` handles:
-   **Application Lifecycle**: Initialization (`App::new`), Event Loop (`handle_event`), and Cleanup.
-   **State Management**: Holds `App` struct with all application state (Windows, Audio, Modules, UI State).
-   **Update Logic**: Audio processing, Module evaluation, Physics updates (`App::update`).
-   **Rendering Pipeline**: WGPU command encoding, Effect chains, Mesh warping (`App::render`).
-   **UI Layout & Logic**: Immediate mode GUI definition (egui) for all panels and windows within `App::render`.

## 2. Visual Overview (UI Layout)

### A. High-Level Layout Structure (Container Hierarchy)

This diagram shows the primary layout areas *without* specific element details.

```ascii
+---------------------------------------------------------------+
| [Top Panel] Menu Bar (File, Edit, View, Window, Help)         |
+---------------------------------------------------------------+
| [Left Panel]      | [Central Panel]       | [Right Panel]     |
|                   |                       |                   |
|                   |                       |                   |
| Sidebar           | Module Canvas         | Inspector         |
| (Resizable)       | (Node Graph / View)   | (Properties)      |
|                   |                       |                   |
|                   |                       |                   |
|                   |                       |                   |
+-------------------|                       |                   |
| [Left Panel Btm]  |                       |                   |
| Preview           |                       |                   |
+-------------------+-----------------------+-------------------+
| [Bottom Panel] Timeline / Sequencer                           |
+---------------------------------------------------------------+
```

### B. Detailed UI Components (With Elements)

This diagram details the specific widgets and functionalities within each area.

```ascii
+---------------------------------------------------------------+
| [Menu Bar] File > Settings | View > Panels | Help > About     |
+-----------------------+-----------------------+---------------+
| [Unified Sidebar]     | [Module Canvas]       | [Inspector]   |
| > Master Controls     |                       |               |
|   [BPM] [Gain]        |  +---+   +---+        | > Layer 1     |
|                       |  |Src|-->|Efct|       |   [Opacity]   |
| > Media Browser       |  +---+   +---+        |   [Blend]     |
|   [Folder Tree]       |                       |               |
|   [Files List]        |  (Node Editor View)   | > Transform   |
|                       |                       |   [Pos X/Y]   |
| > Audio Analysis      |                       |   [Scale]     |
|   [Spectrum Vis]      |                       |               |
|   [Device Select]     |                       | > Effects     |
|                       |                       |   [Blur Lvl]  |
+-----------------------|                       |               |
| [Preview Panel]       |                       |               |
| [Thumb 1] [Thumb 2]   |                       |               |
+-----------------------+-----------------------+---------------+
| [Timeline]                                                    |
| [ |> ] [ || ] [ [] ]  <-- Transport Controls                  |
| [Keyframe Editor -------------------------------------------] |
+---------------------------------------------------------------+

[Floating Windows / Overlays]
+-------------------+   +-------------------+
| [Settings Window] |   | [Effect Chain]    |
| > Project         |   | List of active    |
| > Audio/MIDI      |   | effects & params  |
| > Graphics        |   +-------------------+
+-------------------+
```

## 3. Refactoring Roadmap (Step-by-Step)

The refactoring will be executed in phases to ensure stability at each step.

### **Phase 1: UI Modularization (Current Focus)**
*Goal: Move UI definition code out of `main.rs` into `crates/mapmap/src/ui/`.*

*   **Step 1.1: Setup & Settings Window**
    *   Create `crates/mapmap/src/ui/` and `mod.rs`.
    *   Extract the "Settings" window (Project, Audio, Graphics tabs) into `ui/settings.rs`.
    *   *Deliverable:* `ui::settings::show(...)` called in `main.rs`.

*   **Step 1.2: Main Layout (Sidebar & Panels)**
    *   Extract the Sidebar (Media, Audio, Master) into `ui/sidebar.rs`.
    *   Extract the Inspector into `ui/inspector.rs` (if not already in its own module).
    *   Extract the Timeline into `ui/timeline.rs`.
    *   *Deliverable:* `ui::layout::render(...)` manages the high-level docking logic.

*   **Step 1.3: Module Canvas & Nodes**
    *   Move the central panel logic (Module Canvas) into `ui/canvas.rs`.

### **Phase 2: App Structure & Initialization**
*Goal: Slim down the `App` struct and `main` function.*

*   **Step 2.1: App State Separation**
    *   Move `struct App` and its fields into `crates/mapmap/src/app_state.rs` or `context.rs`.
    *   Move `App::new` initialization logic into a builder or factory pattern in `crates/mapmap/src/startup.rs`.

### **Phase 3: Logic & Update Loop**
*Goal: Separate business logic from presentation.*

*   **Step 3.1: Event Handling**
    *   Extract `handle_event` into `crates/mapmap/src/events.rs`.
    *   Create specific handlers for Keyboard, Mouse, and Window events.

*   **Step 3.2: Update Systems**
    *   Move `App::update` logic (Audio analysis, Physics) into `crates/mapmap/src/systems/update.rs`.

### **Phase 4: Render Pipeline**
*Goal: Decouple WGPU rendering from Application logic.*

*   **Step 4.1: Render Loop Extraction**
    *   Move the WGPU render pass logic (Effect Chain application, Mesh warping) into `crates/mapmap/src/renderer/mod.rs`.

## 4. Implementation Strategy for Phase 1 (Step 1.1)

1.  **Create Directory**: `crates/mapmap/src/ui/`
2.  **Create Module Entry**: `crates/mapmap/src/ui/mod.rs` (pub mod settings;)
3.  **Create File**: `crates/mapmap/src/ui/settings.rs`
    *   Function: `pub fn show(ctx: &egui::Context, state: &mut AppState, ui_state: &mut AppUI, ...)`
    *   Content: The `egui::Window::new("Settings")...` block from `main.rs`.
4.  **Update `main.rs`**:
    *   Import `mod ui;`
    *   Replace code block with `ui::settings::show(...)`.
