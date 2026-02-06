# mapmap-bevy

**Bevy Engine Integration for MapFlow**

This crate provides a bridge between MapFlow's orchestration and the [Bevy Game Engine](https://bevyengine.org/) for high-performance 3D rendering and audio reactivity. It allows MapFlow to leverage Bevy's ECS, 3D renderer (PBR), and plugin ecosystem to generate dynamic visual content.

## ✨ Key Features

*   **Headless Runner**: Runs a customized Bevy `App` loop optimized for off-screen rendering.
*   **Audio Reactivity**: Maps MapFlow's audio analysis data (FFT bands, beat detection) to Bevy components (transform, material emission).
*   **Procedural Generation**: Includes systems for generating 3D structures like HexGrids and Particle systems.
*   **Frame Readback**: Efficiently captures the rendered frame from the GPU and exposes it to MapFlow's media pipeline.
*   **Scene Synchronization**: Updates Bevy entities based on the MapFlow module graph state.

## 📦 Dependencies

*   `bevy` (v0.14) - Core engine (default features disabled to reduce bloat).
*   `mapmap-core` - Shared data structures (Audio analysis, Module definitions).
*   `wgpu` - Graphics backend interaction.
*   `bevy_atmosphere`, `bevy_mod_outline`, `bevy_enoki` - Visual extensions.

## 🚀 Usage

The main entry point is the `BevyRunner` struct, which manages the Bevy app instance.

```rust
use mapmap_bevy::BevyRunner;
use mapmap_core::audio_reactive::AudioTriggerData;

fn main() {
    // Initialize the runner
    let mut runner = BevyRunner::new();

    // Mock audio data (usually comes from mapmap-core audio analyzer)
    let audio_data = AudioTriggerData::default();

    // Update the Bevy world with new data
    runner.update(&audio_data);

    // Retrieve the rendered frame (e.g., for display or further processing)
    if let Some((data, width, height)) = runner.get_image_data() {
        println!("Rendered frame: {}x{} ({} bytes)", width, height, data.len());
    }
}
```

## Architecture

1.  **Initialization**: `BevyRunner::new()` sets up a minimal Bevy app with PBR, Render, and custom plugins.
2.  **Scene Setup**: A headless camera is spawned targeting a specific `Image` asset (Render Target).
3.  **Update Loop**: `runner.update()` injects audio data into the `AudioInputResource` and ticks the Bevy scheduler.
4.  **Systems**:
    *   `audio_reaction_system`: Modulates entities based on audio energy.
    *   `hex_grid_system`: Rebuilds meshes when configuration changes.
    *   `frame_readback_system`: Copies the GPU texture to a CPU buffer.
5.  **Output**: `runner.get_image_data()` locks the shared buffer and returns the pixel data.
