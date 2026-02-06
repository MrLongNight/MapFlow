# mapmap-bevy

**Bevy Engine Integration for MapFlow.**

This crate provides a bridge between MapFlow's orchestration and the [Bevy](https://bevyengine.org/) game engine
for high-performance 3D rendering and audio reactivity.

## Features

- **Headless Runner**: Runs Bevy in a headless mode synchronized with MapFlow's loop.
- **Audio Reactivity**: Maps audio analysis data to Bevy resources (`AudioInputResource`).
- **3D Scenes**: Supports standard Bevy scenes, lights, and materials.
- **Render Output**: Extracts rendered frames for use in MapFlow's compositor.

## Usage

This crate is used internally by `mapmap` to drive 3D visual modules.

```rust
use mapmap_bevy::BevyRunner;

// Initialize the runner
let mut runner = BevyRunner::new();

// In the application loop:
// runner.update(&audio_data);
// let frame = runner.get_image_data();
```
