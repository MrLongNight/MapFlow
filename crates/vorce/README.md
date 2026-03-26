# Vorce (Application)

The main entry point and binary crate for the Vorce projection mapping suite.

## Overview

This crate glues together all the specialized crates (`vorce-core`, `vorce-render`, `vorce-ui`, etc.) to form the complete application.
It initializes the engine, sets up the windowing system, and runs the main event loop.

## Documentation

Full project documentation is available in the [`docs/`](../../docs/) directory.

- [Getting Started](../../docs/A4_USER/B1_MANUAL/DOC-C2_QUICKSTART.md)
- [User Guide](../../docs/A4_USER/B1_MANUAL/DOC-C0_README.md)
- [Architecture](../../docs/A1_SYSTEM/B1_ARCHITECTURE/DOC-C1_OVERVIEW.md)

## Running Vorce

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

Vorce stores user configuration (window positions, last used audio device, etc.) in the system's user data directory:

- **Linux**: `~/.local/share/vorce/`
- **Windows**: `%APPDATA%\vorce\`
- **macOS**: `~/Library/Application Support/vorce/`
