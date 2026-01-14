# MapFlow Control

The control system integration layer for MapFlow, providing interfaces for external hardware and protocols.

## Features

- **MIDI**: Extensive MIDI support including input/output, learn mode, and clock synchronization. Includes a built-in profile for Ecler NUO 4.
- **OSC**: Open Sound Control server and client for integration with TouchOSC, Lemur, and other creative coding tools.
- **Cue System**: Automated show control with cues, crossfades, and triggers.
- **Web API**: (Optional) REST API and WebSocket interface for remote control.
- **DMX**: (Planned) Art-Net and sACN support.

## Usage

```rust,no_run
use mapmap_control::{ControlTarget, ControlValue};

// Example: Creating a control target for layer opacity
let target = ControlTarget::LayerOpacity(0);
let value = ControlValue::Float(0.75);
```

## Feature Flags

| Flag       | Description                                      | Default |
|------------|--------------------------------------------------|---------|
| `midi`     | Enables MIDI support via `midir`                 | Yes     |
| `osc`      | Enables OSC support via `rosc`                   | Yes     |
| `http-api` | Enables Web API support via `axum`               | No      |
| `full`     | Enables all features                             | No      |
