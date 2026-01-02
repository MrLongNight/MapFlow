# MapFlow Core (`mapmap-core`)

The core logic and data structures for MapFlow. This crate contains the application state, business logic, and platform-independent systems.

## Features

- **Layer System**: Hierarchical layer management with transformations, opacity, and blending.
- **Mapping System**: Mesh-based mapping with bezier warping and keystone correction.
- **Audio Analysis**: Real-time audio analysis (FFT) with beat detection (`analyzer_v2.rs`).
- **Animation**: Keyframe-based animation system.
- **Project State**: Central `AppState` struct managing the entire application configuration.

## Usage

```rust
use mapmap_core::layer::Layer;
use mapmap_core::state::AppState;

let mut state = AppState::default();
let layer = Layer::new("My Layer");
state.layer_manager.add_layer(layer);
```
