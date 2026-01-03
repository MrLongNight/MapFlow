# MapFlow Control

Handles external control protocols and APIs for MapFlow.

## Features

- **OSC (Open Sound Control)**: Server and client for remote control.
- **MIDI**: Input and output handling, MIDI Learn, and controller profiles (e.g., Ecler NUO 4).
- **HTTP API**: Optional REST/WebSocket API for web-based control.
- **DMX**: Future support for Art-Net and sACN.

## Usage

This crate is the backend for all external control. To use it, you initialize the `ControlManager` and handle the events it emits.

```rust
use mapmap_control::{ControlManager, ControlTarget, ControlValue};

// Initialize the manager
let mut manager = ControlManager::new();

// Set a value programmatically (e.g., from a test or internal logic)
// This simulates receiving a control signal for Layer 0's opacity
manager.set_value(
    ControlTarget::LayerOpacity(0),
    ControlValue::Float(0.75)
).unwrap();
```

## Configuration

Enable features in your `Cargo.toml` as needed:

```toml
[dependencies]
mapmap-control = { version = "0.1", features = ["midi", "osc"] }
```
