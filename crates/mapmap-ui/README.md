# MapFlow UI

The user interface layer for MapFlow, built with `egui`.

## Overview

This crate contains all the UI components, panels, and widgets that make up the MapFlow application.
It manages the interaction between the user and the core application state.

## Key Components

- **Dashboard**: The main control center for playback and performance monitoring.
- **Module Canvas**: A node-based editor for routing signals, effects, and media.
- **Timeline V2**: Keyframe animation editor.
- **Media Browser**: File explorer for managing assets.
- **Inspector**: Context-sensitive property editor.
- **Mapping Panel**: Tools for adjusting projection mapping meshes.
- **Responsive Design**: The UI automatically adapts to different screen sizes and aspect ratios, optimizing panel layout for mobile or compact views.

## Architecture

The UI is structured around a central `AppUI` state object which holds the state of all panels.
Interaction with the core application is handled via a `UIAction` enum, which decouples the UI from the application logic.

## Themes

MapFlow features a customizable theming system ("Cyber Dark") designed for low-light environments typical of live performances,
offering high contrast and reduced eye strain.
