# mapmap-ui

The User Interface layer for MapFlow, built with `egui`.

## Overview
This crate implements the visual interface of MapFlow. It uses `egui` for immediate mode GUI rendering, integrated with `wgpu` and `winit`. It handles all user interactions, panel rendering, and theme management.

## Key Panels

- **Dashboard**: Main control center for global parameters.
- **Module Canvas**: Node-based editor for routing signals and effects.
- **Timeline**: Keyframe animation editor.
- **Media Browser**: Asset management and file selection.
- **Mapping Panel**: Controls for projection mapping and warping.
- **Layer Panel**: Hierarchy and composition management.

## Architecture

- **AppUI**: Central struct managing the state of all UI panels.
- **Panels**: Self-contained modules implementing specific UI functionality (e.g., `controller_overlay_panel.rs`, `osc_panel.rs`).
- **Widgets**: Custom reusable components like `AudioMeter`, specialized sliders, and node graph elements.
- **Theming**: "Cyber Dark" theme definition and custom visual styles.
