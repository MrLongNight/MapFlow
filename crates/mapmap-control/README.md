# mapmap-control

External control protocols for MapFlow (OSC, MIDI).

## Overview
This crate enables MapFlow to be controlled by external hardware and software. It manages input devices, parses messages, and routes them to the application logic.

## Supported Protocols

- **OSC (Open Sound Control)**:
  - UDP-based server and client.
  - Flexible address routing.
  - Real-time parameter feedback.

- **MIDI**:
  - Input/Output via `midir`.
  - Device discovery and hot-plugging support.
  - Controller profiles (e.g., Ecler NUO 4).
  - MIDI Clock synchronization.

## Architecture
Control messages are normalized into internal events that drive the `AppState` or `ModuleSystem`. This decoupling allows any input source to control any parameter.
