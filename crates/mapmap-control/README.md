# mapmap-control

**mapmap-control** handles external input and control protocols. It bridges the gap between hardware controllers/network protocols and the application state.

## Key Features

*   **OSC (Open Sound Control)**: Server and client for remote control (defaults to port 8000).
*   **MIDI**: Input and output handling for MIDI controllers.
    *   Includes a profile for the Ecler NUO 4.
    *   Supports MIDI Learn.
*   **Web API**: HTTP server (Axum) for RESTful control (optional feature).
*   **Input Abstraction**: Unifies different input sources into `ControlSource` signals.

## Features Flags

*   `osc`: Enables OSC support (default).
*   `midi`: Enables MIDI support (default).
*   `http-api`: Enables the embedded web server (optional).

## Usage

```rust
// Initialize control manager
// let mut control = ControlManager::new(&config);
// control.update(&mut app_state);
```
