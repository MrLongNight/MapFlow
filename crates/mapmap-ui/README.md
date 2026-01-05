# mapmap-ui

**mapmap-ui** provides the user interface for MapFlow, built with `egui`. It contains all the panels, widgets, and interaction logic.

## Key Features

*   **egui Integration**: Uses `egui` for immediate mode GUI rendering.
*   **Panels**:
    *   `Dashboard`: Main overview and playback control.
    *   `ModuleCanvas`: Node-based editor for routing and effects.
    *   `Timeline`: Keyframe animation editor.
    *   `MappingPanel`: Geometry and warping controls.
    *   `AudioPanel`: Audio visualization and settings.
*   **Custom Widgets**:
    *   `AudioMeter`: Retro and Digital VU meters.
    *   `Knob`: Rotary control for parameters.
*   **Theming**: Cyber-dark theme implementation (`theme.rs`).
*   **Internationalization**: Localization support using `fluent`.

## Usage

The UI crate exposes an `AppUI` struct that manages the UI state and a `render` function called by the main application loop.

```rust
// In main loop
// ui.render(&mut ctx, &mut app_state);
```

## Modules

*   `dashboard`: Main control dashboard.
*   `module_canvas`: Node editor UI.
*   `timeline_v2`: Animation timeline.
*   `theme`: Visual style definitions.
