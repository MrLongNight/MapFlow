# Build Instructions

This document provides comprehensive build instructions for MapFlow (Rust Edition) on all supported platforms.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Linux Build](#linux-build)
- [macOS Build](#macos-build)
- [Windows Build](#windows-build)
- [Building for Development](#building-for-development)
- [Building for Release](#building-for-release)
- [Running Tests](#running-tests)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Rust Toolchain

MapFlow requires **Rust 1.75 or later**. Install it using rustup:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then restart your terminal

# Verify installation
rustc --version
cargo --version
```

### Git

Ensure you have Git installed to clone the repository:

```bash
# Verify Git installation
git --version
```

---

## Linux Build

### System Requirements

- **OS:** Ubuntu 20.04+, Debian 11+, Fedora 35+, or equivalent
- **CPU:** x86_64 with SSE2 support
- **GPU:** Vulkan 1.2 compatible (NVIDIA GTX 900+, AMD GCN 2.0+, Intel HD 4000+)
- **RAM:** 4GB minimum, 8GB recommended
- **Disk Space:** 2GB for build artifacts

### Install Dependencies

#### Ubuntu/Debian

```bash
# Update package lists
sudo apt-get update

# Install build essentials
sudo apt-get install -y build-essential pkg-config

# Install X11/Wayland development libraries
sudo apt-get install -y \
  libxcb1-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libx11-dev \
  libwayland-dev \
  libxkbcommon-dev

# Install font rendering libraries (required for ImGui)
sudo apt-get install -y \
  libfontconfig1-dev \
  libfreetype6-dev

# Install audio libraries (for future audio support)
sudo apt-get install -y libasound2-dev

# REQUIRED for real video playback: Install FFmpeg development libraries
sudo apt-get install -y \
  libavcodec-dev \
  libavformat-dev \
  libavutil-dev \
  libswscale-dev \
  libavdevice-dev \
  libavfilter-dev
```

#### Fedora

```bash
# Install build essentials
sudo dnf install -y gcc gcc-c++ pkg-config

# Install X11/Wayland development libraries
sudo dnf install -y \
  libxcb-devel \
  libX11-devel \
  wayland-devel \
  libxkbcommon-devel

# Install font rendering libraries
sudo dnf install -y \
  fontconfig-devel \
  freetype-devel

# Install audio libraries
sudo dnf install -y alsa-lib-devel

# Optional: Install FFmpeg development libraries
sudo dnf install -y \
  ffmpeg-devel \
  ffmpeg-free-devel
```

#### Arch Linux

```bash
# Install base development tools
sudo pacman -Syu base-devel

# Install required libraries
sudo pacman -S \
  libxcb \
  libx11 \
  wayland \
  libxkbcommon \
  fontconfig \
  freetype2 \
  alsa-lib

# Optional: Install FFmpeg
sudo pacman -S ffmpeg
```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Build in debug mode (faster compile, slower runtime)
cargo build

# Build in release mode (slower compile, faster runtime)
cargo build --release

# Run the application
cargo run --release
```

---

## macOS Build

### System Requirements

- **OS:** macOS 11 (Big Sur) or later
- **CPU:** Intel x86_64 or Apple Silicon (ARM64)
- **GPU:** Metal-compatible GPU
- **RAM:** 4GB minimum, 8GB recommended
- **Disk Space:** 2GB for build artifacts

### Install Dependencies

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install FFmpeg (optional, for video playback)
brew install ffmpeg
```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Build in release mode
cargo build --release

# Run the application
cargo run --release
```

---

## Windows Build

### System Requirements

- **OS:** Windows 10 version 1809 or later
- **CPU:** x86_64 with SSE2 support
- **GPU:** DirectX 12 compatible
- **RAM:** 4GB minimum, 8GB recommended
- **Disk Space:** 3GB for build artifacts

### Install Dependencies

1. **Install Visual Studio 2022** (Community Edition is free)
   - Download from: https://visualstudio.microsoft.com/downloads/
   - During installation, select:
     - "Desktop development with C++"
     - Windows 10/11 SDK

2. **Install Rust**
   - Download from: https://rustup.rs/
   - Run `rustup-init.exe` and follow instructions
   - Choose "default installation"

3. **Optional: Install FFmpeg** (for video playback)
   - Download from: https://www.gyan.dev/ffmpeg/builds/
   - Extract to `C:\ffmpeg`
   - Add `C:\ffmpeg\bin` to PATH

### Build Steps

Open Command Prompt or PowerShell:

```cmd
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Build in release mode
cargo build --release

# Run the application
cargo run --release
```

---

## Building for Development

For faster iteration during development:

```bash
# Build in debug mode (default)
cargo build

# Run in debug mode
cargo run

# Enable FFmpeg features
cargo run --features ffmpeg

# Watch for changes and rebuild automatically
cargo watch -x run
```

---

## Building for Release

For production or distribution:

```bash
# Build optimized release binary
cargo build --release

# The binary will be at:
# Linux/macOS: ./target/release/mapmap
# Windows: .\target\release\mapmap.exe

# Run release build
./target/release/mapmap  # Linux/macOS
.\target\release\mapmap.exe  # Windows
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p mapmap-core
cargo test -p mapmap-render

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

---

## Troubleshooting

### Linux: "error: linker 'cc' not found"

Install build essentials:
```bash
sudo apt-get install build-essential
```

### Linux: "could not find library -lxcb"

Install X11 development libraries:
```bash
sudo apt-get install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### macOS: "xcrun: error: invalid active developer path"

Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Windows: "link.exe not found"

Install Visual Studio 2022 with C++ desktop development workload.

### FFmpeg not found

Make sure FFmpeg development libraries are installed:

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**macOS:**
```bash
brew install ffmpeg
```

**Windows:**
Download pre-built FFmpeg libraries from https://www.gyan.dev/ffmpeg/builds/

### Slow debug builds

Use release mode or enable some optimizations in debug:
```bash
cargo build --release
# or
CARGO_PROFILE_DEV_OPT_LEVEL=1 cargo build
```

### GPU driver issues

Ensure your GPU drivers are up-to-date:
- **NVIDIA:** Download from https://www.nvidia.com/drivers
- **AMD:** Download from https://www.amd.com/support
- **Intel:** Usually auto-updated with Windows Update

---

## Additional Resources

- **Documentation:** See [docs/](docs/) directory
- **Architecture:** [docs/03-ARCHITECTURE/ARCHITECTURE.md](docs/03-ARCHITECTURE/ARCHITECTURE.md)
- **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)
- **Issues:** https://github.com/MrLongNight/MapFlow/issues
