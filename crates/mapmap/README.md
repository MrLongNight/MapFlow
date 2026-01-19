# MapFlow (Application)

The main entry point and binary crate for the MapFlow projection mapping suite.

## Overview

This crate glues together all the specialized crates (`mapmap-core`, `mapmap-render`, `mapmap-ui`, etc.) to form the complete application. It initializes the engine, sets up the windowing system, and runs the main event loop.

## Documentation

Full project documentation is available in the [`docs/`](../../docs/) directory.

- [Getting Started](../../docs/01-GETTING-STARTED/)
- [User Guide](../../docs/04-USER-GUIDE/)
- [Architecture](../../docs/03-ARCHITECTURE/)

## Running MapFlow

### Prerequisites

Ensure you have the necessary system dependencies installed (see root README).

### Build & Run

```bash
# Run in release mode (recommended for performance)
cargo run --release

# Run with FFmpeg support (video playback)
cargo run --release --features ffmpeg

# Run with full audio support
cargo run --release --features ffmpeg,audio
```

## Configuration

MapFlow stores user configuration (window positions, last used audio device, etc.) in the system's user data directory:

- **Linux**: `~/.local/share/mapflow/`
- **Windows**: `%APPDATA%\mapflow\`
- **macOS**: `~/Library/Application Support/mapflow/`
