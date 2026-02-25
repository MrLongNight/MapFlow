# API Reference

This directory serves as the entry point for MapFlow's API documentation.

## Generating Documentation

MapFlow follows standard Rust documentation practices. To generate and view the full API reference locally:

```bash
# Generate docs for all workspace members
cargo doc --workspace --no-deps --open
```

To include documentation for features that are behind flags (like `audio`, `ndi`, `spout`):

```bash
cargo doc --workspace --no-deps --all-features --open
```

## Core Modules Overview

MapFlow is organized into a workspace of specialized crates. Here is a high-level overview of the API structure:

### 🧠 Core & Logic
* **[`mapmap-core`](../../../crates/mapmap-core/README.md)**: The domain model. Defines `Project`, `Layer`, `Module`, and the core evaluation logic. Independent of rendering and UI.
* **[`mapmap-control`](../../../crates/mapmap-control/README.md)**: Handles external control inputs (OSC, MIDI, WebSocket) and routing them to internal parameters.
* **[`mapmap-mcp`](../../../crates/mapmap-mcp/README.md)**: Implements the Model Context Protocol server for AI assistant integration.

### 🎨 Rendering & Media
* **[`mapmap-render`](../../../crates/mapmap-render/README.md)**: The `wgpu`-based rendering engine. Manages the GPU context, shaders, and render loops.
* **[`mapmap-media`](../../../crates/mapmap-media/README.md)**: Media decoding (FFmpeg/mpv) and playback state management.
* **[`mapmap-bevy`](../../../crates/mapmap-bevy/README.md)**: Integration bridge for the Bevy game engine (used for 3D particles and scenes).

### 🖥️ User Interface & IO
* **[`mapmap-ui`](../../../crates/mapmap-ui/README.md)**: The `egui`-based user interface. Connects user actions to core state changes.
* **[`mapmap-io`](../../../crates/mapmap-io/README.md)**: Hardware IO and specialized protocols (NDI, Spout, Hue, etc.).

### 🚀 Application
* **[`mapmap`](../../../crates/mapmap/README.md)**: The main binary crate. Orchestrates startup, loop management, and ties all crates together.

## Development Tips

* **Use `cargo check`**: For rapid feedback during development.
* **Clippy is your friend**: `cargo clippy` is enforced in CI.
* **Tests**: Run `cargo test` regularly. Core logic is heavily tested in `mapmap-core`.
