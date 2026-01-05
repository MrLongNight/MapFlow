# mapmap-core

**mapmap-core** contains the fundamental domain logic, data structures, and state management for MapFlow. It is designed to be completely independent of the rendering backend (wgpu) and the UI framework (egui), ensuring clean separation of concerns.

## Key Features

*   **State Management**: `AppState` serves as the single source of truth for the application.
*   **Layer System**: Hierarchical composition of visual layers with blending modes and transformations.
*   **Mapping & Mesh**: Geometry definitions for projection mapping, including Bézier warps and keystoning.
*   **Audio Analysis**: Core logic for FFT analysis (`AudioAnalyzerV2`), beat detection, and frequency band energy calculation.
*   **Module System**: Node-based logic for the "Module Canvas", defining sources, filters, and outputs.
*   **Animation**: Keyframe animation system and interpolation logic.
*   **Effects Logic**: Data structures for effect parameters and chains (rendering logic is in `mapmap-render`).

## Usage

This crate is the foundation for all other MapFlow crates.

```rust
use mapmap_core::state::AppState;
use mapmap_core::layer::LayerManager;

// Initialize state
let mut state = AppState::default();

// Access managers
let layer_count = state.layer_manager.layers().count();
```

## Modules

*   `state`: Global application state container.
*   `layer`: Layer composition and management.
*   `mesh`: Geometry and warping data structures.
*   `audio`: Audio analysis and reactive logic.
*   `mapping`: Projector mapping configurations.
*   `module`: Node graph logic.
