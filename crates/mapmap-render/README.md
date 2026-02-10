# MapFlow Render

The low-level graphics rendering engine for MapFlow, built on top of `wgpu`.

## Overview

MapFlow Render provides a robust abstraction over modern graphics APIs (Vulkan, Metal, DX12), handling the complex details of the rendering pipeline so the core application can focus on logic.

## Key Modules

- **backend**: The core `wgpu` backend initialization and device management.
- **compositor**: Handles the blending of multiple layers into a final composition.
- **mesh_renderer**: Renders warped meshes with texture mapping.
- **edge_blend_renderer**: Applies soft-edge blending for multi-projector setups.
- **color_calibration_renderer**: Per-output color correction and gamma adjustment.
- **effect_chain_renderer**: Post-processing effect pipeline.
- **shader_graph_integration**: Integration with the node-based shader graph system.
- **hot_reload**: Real-time shader hot-reloading for rapid development.

## Architecture

Rendering in MapFlow is pipeline-based. The `Compositor` takes a scene description and executes a series of render passes:

1. **Layer Rendering**: Individual layers are rendered to intermediate textures.
2. **Composition**: Layers are blended together.
3. **Output Mapping**: The composition is mapped onto 3D meshes for projection.
4. **Post-Processing**: Edge blending and color calibration are applied.
5. **Presentation**: The final result is presented to the window surface.
