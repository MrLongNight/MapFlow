# MapFlow UI

The user interface layer for MapFlow, built with `egui`.

## Overview

This crate contains all the UI components, panels, and widgets that make up the MapFlow application. It manages the interaction between the user and the core application state.

**Status:** Phase 6 (UI Migration) Complete. ImGui has been fully removed in favor of `egui` for a modern, unified look and better integration with `wgpu`.

## Key Components

### 🎛️ Main Control
- **Dashboard**: The main control center for playback, performance monitoring (FPS, CPU/GPU), and master controls.
- **MenuBar**: Top-level application menu (File, Edit, View, Help).

### 🎬 Editors
- **Module Canvas**: The heart of MapFlow. A node-based editor for routing signals, effects, and media. Supports:
    - **Sources:** Media Files, Shaders, Live Input (Spout/NDI).
    - **Effects:** Blur, Glitch, Colorize, etc.
    - **Layers:** Composition and blending.
    - **Outputs:** Projectors and Screens.
- **Timeline V2**: Keyframe animation editor for automating parameters over time.
- **Mesh Editor**: Specialized editor for Bezier warping and keystone correction.
- **Node Editor**: (Advanced) Shader Graph editor for creating custom effects.

### 📂 Asset Management
- **Media Browser**: File explorer with thumbnail previews for managing video and image assets. Supports drag-and-drop to the Module Canvas.

### 🔧 Configuration Panels
- **Inspector Panel**: Context-sensitive property editor. Adapts to show settings for the selected Layer, Output, or Module.
- **Output Panel**: Configuration for physical outputs, resolution, and refresh rates.
- **Mapping Panel**: Tools for managing the mapping hierarchy.
- **Edge Blend Panel**: Controls for multi-projector edge blending and gamma correction.
- **Audio Panel**: Visualization of audio analysis (FFT, Waveform) and device selection.
- **Oscillator Panel**: Controls for the Oscillator Distortion effect.
- **Cue Panel**: List view for show cues (Phase 7).
- **Controller Overlay**: Visual feedback for MIDI controllers (e.g., Ecler NUO4).

## Architecture

The UI is structured around a central `AppUI` state object which holds the state of all panels.

- **State Management**: `AppUI` (in `lib.rs`) holds `menu_bar`, `dashboard`, `module_canvas`, etc.
- **Action System**: Interaction with the core application is handled via a `UIAction` enum, which decouples the UI from the application logic. The main loop collects these actions and applies them to the `mapmap-core` state.
- **Assets**: Icons are managed by `IconManager`.

## Themes

MapFlow features a customizable theming system.

### 🌑 Cyber Dark Theme
Designed for low-light environments typical of live performances.
- **High Contrast**: Readable text against dark backgrounds.
- **Color Coding**: Visual cues for different node types and states.
- **Reduced Eye Strain**: Avoids pure white/black extremes.
- **Accessibility**: Scalable UI elements.
