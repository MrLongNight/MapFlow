# MapFlow Installation Guide

## ⚠️ Important Note

MapFlow is a complete rewrite of the legacy "MapMap" application. The C++/Qt version instructions are obsolete.

## Supported Platforms

- **Linux**: Ubuntu 22.04+, Debian 12+, Fedora 38+ (x86_64)
- **Windows**: Windows 10/11 (x86_64)
- **macOS**: macOS 12+ (Apple Silicon & Intel) - *Experimental*

## Installation Methods

### 1. Download Release Binaries (Recommended)

Pre-built binaries for Windows and Linux are available on the GitHub Releases page:

[**Download Latest Release**](https://github.com/MrLongNight/MapFlow/releases)

1. Download the appropriate file for your OS (`.zip` for Windows, `.deb` for Debian/Ubuntu).
2. **Windows**: Extract the zip archive and run `mapflow.exe`.
3. **Linux**: Install the package:
   ```bash
   sudo dpkg -i mapflow_*.deb
   sudo apt-get install -f  # Fix missing dependencies if any
   mapflow
   ```

### 2. Build from Source

For detailed build instructions, please refer to [BUILD.md](BUILD.md).

Quick summary for Rust developers:

```bash
# Clone
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Install system dependencies (Ubuntu example)
sudo apt-get install build-essential pkg-config libxcb1-dev libasound2-dev libavcodec-dev libavformat-dev libavutil-dev libswscale-dev

# Run
cargo run --release --features ffmpeg,audio
```
