# MapFlow Render (`mapmap-render`)

The GPU rendering engine for MapFlow, built on `wgpu`.

## Features

- **Compositor**: Multi-layer composition with blend modes.
- **Mesh Renderer**: Bezier-based mesh warping and texture mapping.
- **Effect Chain**: Post-processing pipeline with support for custom shaders.
- **Output Management**: Edge blending, color calibration, and multi-monitor support.
- **Shaders**: WGSL shader management and hot-reloading.

## Usage

This crate provides the `RenderBackend` which manages the `wgpu` device, queue, and surface.

```rust
use mapmap_render::backend::RenderBackend;

// Initialize backend
let backend = RenderBackend::new(&window).await?;
```
