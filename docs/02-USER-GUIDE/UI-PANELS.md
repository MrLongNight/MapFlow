# UI Panels Guide

This guide provides a detailed overview of the user interface panels in MapFlow. MapFlow's UI is designed to be modular and flexible, allowing you to arrange panels to suit your workflow.

## Overview

MapFlow's interface is built around a docking system. You can drag panel tabs to rearrange them, or use the **View** menu to show/hide specific panels.

The default layout typically includes:
- **Left Sidebar**: Media Browser
- **Center**: Module Canvas / Dashboard
- **Right Sidebar**: Inspector
- **Bottom**: Timeline

## 1. Dashboard

The Dashboard is your main control center during a performance.

*   **Playback Controls**: Play, Pause, Stop, and Speed control for the global transport.
*   **Performance Stats**: Real-time monitoring of FPS, Frame Time, CPU, and GPU usage.
*   **Master Controls**: Global opacity and speed sliders.
*   **Toolbar**: Quick access to common tools and modes (Edit, Map, Perform).

## 2. Module Canvas

The Module Canvas is the heart of MapFlow's node-based workflow. Here you connect different modules to create your visual signal flow.

*   **Nodes**: Represent functional units (Media Players, Effects, Layers, Outputs).
*   **Connections**: Wires connecting outputs of one node to inputs of another.
*   **Interaction**:
    *   **Right-Click**: Open the "Add Node" menu.
    *   **Drag Wire**: Create a connection.
    *   **Click Node**: Select it to view properties in the Inspector.

### Common Node Types
*   **Source**: Media File, Generator, Live Input.
*   **Effect**: Blur, Color Correction, Distortion.
*   **Layer**: Compositing nodes.
*   **Output**: Projector, Syphon/Spout, NDI.

## 3. Media Browser

The Media Browser allows you to browse and import content from your local file system.

*   **Navigation**: Browse folders and drives.
*   **Preview**: Hover over supported files to see a preview (where available).
*   **Import**: Drag files from the browser onto the Module Canvas to create a Media Node.
*   **Supported Formats**:
    *   **Video**: MP4, MOV, MKV, WebM (H.264, H.265, VP8/9, HAP).
    *   **Image**: PNG, JPG, BMP, GIF.

## 4. Inspector

The Inspector is a context-sensitive panel that shows the properties of the currently selected object.

*   **Node Properties**: When a node is selected in the Canvas, its specific parameters (e.g., Opacity, Speed, Effect Strength) are shown here.
*   **Layer Properties**: Transform (Position, Scale, Rotation), Blend Mode, and Opacity.
*   **Mapping Properties**: When editing a mesh, vertex coordinates and warping controls appear here.

## 5. Timeline

The Timeline is used for sequencing and automation.

*   **Tracks**: Each layer or parameter can have its own track.
*   **Keyframes**: Set values at specific points in time to animate parameters.
*   **Transport**: Scrub through time to preview animations.
*   **Modes**: Switch between different playback modes (Loop, One-Shot).

## 6. Mapping Panel

The Mapping Panel is dedicated to projection mapping tasks.

*   **Mesh Editor**: Select and edit warping meshes.
*   **Keystone**: Basic 4-point correction.
*   **Grid Warp**: Advanced multi-point warping for curved surfaces.
*   **Masking**: Draw masks to hide unwanted parts of the projection.

## 7. Output Panel

Manage your physical and virtual outputs.

*   **Displays**: Assign MapFlow outputs to physical monitors or projectors.
*   **Resolution**: Set custom resolutions and refresh rates.
*   **Test Patterns**: Display grids and color bars for alignment.
*   **Edge Blending**: Configure overlaps for multi-projector setups.
*   **Color Calibration**: Adjust gamma and color balance per output.

## 8. Audio Panel

Monitor and configure audio input.

*   **Waveform/Spectrum**: Visual feedback of the incoming audio signal.
*   **Input Selection**: Choose your audio interface and channel.
*   **Gain**: Adjust input levels.
*   **Triggers**: Configure audio-reactive thresholds (Bass, Mid, High).

## 9. Controller Overlay

A visual reference for your connected MIDI controller.

*   **Visualization**: Shows the state of knobs, faders, and buttons on your hardware.
*   **Feedback**: Highlights active controls and shows current values.
*   **MIDI Learn**: Toggle learn mode to map UI elements to hardware controls.
