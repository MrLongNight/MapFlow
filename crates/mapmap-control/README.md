# MapFlow Control (`mapmap-control`)

External control protocols for MapFlow (OSC, MIDI, etc.).

## Features

- **OSC**: Open Sound Control server and client for remote control.
- **MIDI**: Input/Output handling with mapping support (e.g., Ecler NUO4 profile).
- **Web Server**: (Optional) HTTP/WebSocket interface for control.
- **Cue System**: Cue list management for show automation.

## Features Flags

- `osc`: Enable OSC support (default).
- `midi`: Enable MIDI support (default).
- `dmx`: Enable DMX support (planned).

## Usage

```rust
use mapmap_control::osc::OscServer;

// Start OSC server
let server = OscServer::new(8000)?;
```
