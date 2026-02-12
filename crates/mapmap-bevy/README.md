# MapFlow Bevy Integration

The **MapFlow Bevy** crate provides integration with the [Bevy](https://bevyengine.org/) game engine,
enabling high-performance 3D rendering, particle systems, and advanced visual effects within the MapFlow ecosystem.

## Overview

This crate serves as the bridge between MapFlow's core architecture and Bevy's ECS (Entity Component System) and rendering capabilities.
It allows MapFlow to leverage Bevy's robust 3D features while maintaining its own application lifecycle.

## Features

- **Bevy ECS Integration**: Seamlessly run Bevy systems alongside MapFlow logic.
- **3D Rendering**: Full PBR (Physically Based Rendering) support via Bevy's renderer.
- **Particle Systems**: GPU-accelerated particle effects (Custom Mesh-Based implementation).
- **3D Text**: Dynamic 3D text generation with customizable font size, color, and alignment.
- **Atmospheric Rendering**: Realistic sky and atmosphere rendering using [`bevy_atmosphere`](https://crates.io/crates/bevy_atmosphere).
- **Post-Processing**: Outline effects and other post-fx via [`bevy_mod_outline`](https://crates.io/crates/bevy_mod_outline).
- **WGPU Interop**: Shares `wgpu` context resources for efficient frame composition.

## Usage

This crate is primarily used by the main `mapmap` application to power 3D modules and visualizers. It is not intended to be used as a standalone application.

## Dependencies

- `bevy` (Rendering, Core Pipeline, PBR, GLTF, Scene)
- `bevy_atmosphere` (Skybox/Atmosphere)
- `bevy_mod_outline` (Visual outlines)
- `mapmap-core` (Core data structures)
