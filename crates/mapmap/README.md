# mapmap

The main entry point for the MapFlow application.

## Overview
This is the binary crate that ties everything together. It initializes the application, sets up the window and event loop, and orchestrates the interaction between the core, rendering, and UI systems.

## Responsibilities

- **Application Lifecycle**: Startup, main loop, shutdown.
- **Window Management**: Uses `winit` to create and manage the application window.
- **Initialization**: Sets up logging, audio backend, MIDI/OSC servers, and the GPU context.
- **Event Handling**: Routes window and input events to the UI and Core systems.

## Usage

Run the application:
```bash
cargo run --release
```

Enable video playback (requires FFmpeg):
```bash
cargo run --release --features ffmpeg
```
