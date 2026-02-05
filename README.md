<div align="center">
  <img src="resources/app_icons/MapFlow_Logo_HQ-Full-M.png" alt="MapFlow Logo" width="500"/>
</div>

<!-- Dynamic CI/CD Status Badges -->
[![Build & Test](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build%26test.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build%26test.yml)
[![Security Scan](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml)
[![Release](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09_create-releases.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09_create-releases.yml)
[![CI-10: Backup Main Branch](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-10_backup-main.yml/badge.svg?branch=main)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-10_backup-main.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# MapFlow

> **Modern, High-Performance Projection Mapping Suite**

MapFlow is a professional-grade, open-source projection mapping system rewritten in Rust. Originally a C++/Qt application (MapMap), MapFlow has been transformed into a modern, high-performance tool capable of competing with commercial solutions like Resolume Arena.

## 🎯 Vision

Projection mapping (also known as video mapping and spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. MapFlow aims to provide a professional, open-source alternative for artists, designers, and technical professionals who need powerful projection mapping capabilities without the cost of commercial software.

## 🚀 Project Status

**Current Phase: Phase 6 (UI Migration) - ✅ COMPLETED**

MapFlow is now a fully functional Rust application. The migration from ImGui to `egui` is complete, and the core engine delivers high performance.

### Key Features:
- **Memory Safety:** Eliminates entire classes of crashes common in live performance software.
- **Modern Graphics:** Utilizes `wgpu` for access to Vulkan, Metal, and DX12.
- **High Performance:** Built for 60fps+ at 4K with multiple outputs.
- **Cross-Platform:** Runs on Linux, macOS, and Windows.
- **Audio Reactivity:** Built-in FFT analysis and beat detection.
- **Node-Based Workflow:** Advanced "Module Canvas" for visual signal routing.

## 🗺️ Roadmap Overview

For a detailed status, see the [ROADMAP](ROADMAP_2.0.md).

**Phase 1: Core Engine**
-   [x] Layer system with transforms, opacity, and blend modes
-   [x] Hardware-accelerated video decoding (FFmpeg)
-   [x] Advanced playback controls
-   [x] Image/GIF support

**Phase 2: Professional Multi-Projector System**
-   [x] Multi-output rendering
-   [x] Bezier-based mesh warping
-   [x] Edge blending and color calibration

**Phase 3: Effects Pipeline**
-   [x] GPU compute effects (Blur, Glitch, etc.)
-   [x] Audio reactivity (FFT, Beat)
-   [x] Shader Graph system

**Future Phases:**
-   **Multi-PC:** Distributed rendering via NDI (In Progress).
-   **Lighting:** DMX/Art-Net integration.

## 🛠️ Quick Start

### Prerequisites

**Rust Toolchain:**
```bash
# Install Rust 1.75 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**System Dependencies (Ubuntu/Debian):**
```bash
sudo apt-get install -y \
  build-essential pkg-config \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libfontconfig1-dev libfreetype6-dev libasound2-dev \
  libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libavdevice-dev
```

### Build and Run

```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Run in release mode with full features (Video + Audio)
cargo run --release --features ffmpeg,audio
```

## 🏗️ Architecture

MapFlow is organized as a Cargo workspace:

```
crates/
├── mapmap-core/      # Domain model and logic
├── mapmap-render/    # Graphics engine (wgpu)
├── mapmap-media/     # Video playback (FFmpeg)
├── mapmap-ui/        # User Interface (egui)
├── mapmap-control/   # MIDI/OSC/Automation
├── mapmap-io/        # NDI/Spout/DeckLink
├── mapmap-bevy/      # 3D/Generative Engine (Bevy)
├── mapmap-mcp/       # AI Context Protocol
└── mapmap/           # Main application entry point
```

### Technology Stack

- **Language:** Rust 2021
- **Graphics:** `wgpu`
- **UI:** `egui`
- **Media:** FFmpeg
- **Windowing:** `winit`

## 🤝 Contributing

See the [Contributing Guidelines](CONTRIBUTING.md).

## 📄 License

MapFlow is licensed under the **GNU General Public License v3.0** (GPL-3.0). See [LICENSE](LICENSE).

## 🙏 Acknowledgments

- **Original MapMap Team** - For the foundational concepts.
- **The Rust Community** - For the amazing ecosystem.
