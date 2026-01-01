# mapmap-core

The core logic and data structures for MapFlow.

## Overview
This crate contains the fundamental data models and state management logic for the MapFlow application. It is designed to be independent of the rendering and UI layers, ensuring clean separation of concerns.

## Key Modules

- **State Management**: `AppState`, `LayerManager`, `MappingManager`, `PaintManager`
- **Data Models**: `Layer`, `Mapping`, `Mesh`, `Paint`
- **Audio Analysis**: `AudioAnalyzerV2` (FFT-based frequency analysis)
- **Math**: `BezierPatch` for warping, geometric primitives
- **Animation**: `AnimationSystem`, `Keyframe`, `Timeline`

## Features

- **Hierarchical Layer System**: Layers can be grouped and transformed.
- **Advanced Projection Mapping**: Mesh warping with Bezier patches and keystoning.
- **Audio Reactivity**: Analyze audio streams and drive parameters.
- **Module System**: Node-based logic for advanced signal flow.
