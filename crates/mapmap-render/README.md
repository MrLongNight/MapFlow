# mapmap-render

The WGPU-based rendering engine for MapFlow.

## Overview
This crate handles all GPU operations, including rendering pipeline setup, shader management, and compositing. It uses `wgpu` to provide cross-platform hardware acceleration.

## Key Features

- **Compositor**: Blends multiple layers using various blend modes (Add, Multiply, Screen, etc.).
- **Mesh Renderer**: Handles vertex transformation and warping for projection mapping.
- **Effect Chain**: Post-processing pipeline with support for custom shaders.
- **Video Playback Integration**: Efficient texture upload and color space conversion (YUV -> RGB).
- **Oscillator Simulation**: GPU-based simulation for visual effects.
- **Edge Blending**: Soft-edge blending for multi-projector setups.

## Shaders
Shaders are written in WGSL and located in the `shaders/` directory at the project root. This crate manages loading and hot-reloading of these shaders.
