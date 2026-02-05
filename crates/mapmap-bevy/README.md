# MapFlow Bevy Integration

This crate provides a bridge between MapFlow's orchestration and the [Bevy](https://bevyengine.org/) game engine for high-performance 3D rendering and audio reactivity.

> **Status: 🟡 Experimental / Alpha**

## Overview

`mapmap-bevy` enables MapFlow to use Bevy as a generative content source. It runs a headless Bevy app instance, synchronizes audio analysis data from MapFlow core, and extracts the rendered frames to be used as a video source in the MapFlow pipeline.

## Features

- **Headless Runner:** `BevyRunner` manages a minimal Bevy app instance tailored for embedded execution.
- **Audio Reactivity:**
  - `AudioInputResource`: Synchronizes FFT, RMS, and Peak data from `mapmap-core` to Bevy resources.
  - `AudioReactive`: Component to easily map audio energy (Bass, Mid, High, etc.) to transform properties (Scale, Rotate, Position) or material properties.
- **Procedural Content:**
  - `BevyHexGrid`: Parametric hexagonal grid generation.
  - `BevyParticles`: Particle system integration.
  - `BevyAtmosphere`: Sky/Atmosphere rendering.
- **Frame Extraction:** `BevyRenderOutput` captures the rendered frame from wgpu and makes it available to the main MapFlow render loop.

## Usage

This crate is primarily used internally by `mapmap` to spawn the Bevy thread.

```rust
// Internal usage pattern
let mut runner = BevyRunner::new();
runner.update(&audio_data);
if let Some((data, width, height)) = runner.get_image_data() {
    // upload to texture...
}
```
