# mapmap-render

**mapmap-render** handles all GPU interactions using `wgpu`. It implements the rendering pipeline, from texture upload to final compositing and display.

## Key Features

*   **WGPU Backend**: Abstraction over `wgpu` devices, queues, and surfaces.
*   **Compositor**: Handles layer blending and composition.
*   **Renderers**:
    *   `MeshRenderer`: Renders warped geometry for projection mapping.
    *   `EffectChainRenderer`: Applies post-processing effects (shaders) to textures.
    *   `EdgeBlendRenderer`: Handles soft-edge blending for multi-projector setups.
    *   `ColorCalibrationRenderer`: Applies per-output color correction and LUTs.
*   **Shader Management**: WGSL shader loading and hot-reloading support.
*   **Texture Management**: Efficient texture upload and pooling.

## Architecture

This crate receives render commands and data from `mapmap-core` and executes them on the GPU. It does not depend on the UI.

```rust
// Example: Creating a renderer (simplified)
// let renderer = MeshRenderer::new(&device, texture_format);
// renderer.render(&encoder, &view, &mesh_data);
```

## Modules

*   `backend`: WGPU setup and context management.
*   `compositor`: Layer stacking and blending.
*   `mesh_renderer`: Geometry rendering.
*   `effect_chain_renderer`: Post-processing pipeline.
*   `shaders`: WGSL shader source management.
