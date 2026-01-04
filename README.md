<div align="left">
  <img src="resources/app_icons/MapFlow_Logo_HQ-Full-M.webp" alt="MapFlow Logo"/>
</div>

<!-- Dynamic CI/CD Status Badges -->
[![Build & Test](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build%26test.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build%26test.yml)
[![Security Scan](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml)
[![Release](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09_create-releases.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09_create-releases.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# MapFlow

> **Modern, High-Performance Projection Mapping Suite**

MapFlow is a professional-grade, open-source projection mapping system being completely rewritten in Rust. Originally a C++/Qt application (MapMap), MapFlow is being transformed into a modern, high-performance tool capable of competing with commercial solutions like Resolume Arena.

## 🎯 Vision

Projection mapping (also known as video mapping and spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. MapFlow aims to provide a professional, open-source alternative for artists, designers, and technical professionals who need powerful projection mapping capabilities without the cost of commercial software.

## 🚀 Project Status

**Current Phase: Phase 1 (Core Engine) - 🚧 IN PROGRESS**

MapFlow is a complete rewrite in Rust. The project is in its early stages, focusing on building a solid foundation for a high-performance, memory-safe, and modern projection mapping tool.

### From Legacy to Rust:
- **Memory Safety:** Eliminates entire classes of crashes common in live performance software.
- **Modern Graphics:** Utilizes `wgpu` for access to Vulkan, Metal, and DX12, moving beyond legacy OpenGL.
- **High Performance:** Built for 60fps+ at 4K with multiple outputs, leveraging Rust's zero-cost abstractions and fearless concurrency.
- **Cross-Platform:** Designed to run on Linux, macOS, and Windows.

## 🗺️ Roadmap

The development of MapFlow is planned in several phases, starting with the core engine and gradually adding more advanced features.

**Phase 1: Core Engine**
-   [ ] Layer system with transforms, opacity, and blend modes
-   [ ] Hardware-accelerated video decoding
-   [ ] Advanced playback controls (speed, direction, loop modes)
-   [ ] Still image and animated format support (PNG, JPG, GIF)

**Phase 2: Professional Multi-Projector System**
-   [ ] Multi-output rendering with synchronization
-   [ ] Bezier-based mesh warping
-   [ ] Edge blending and color calibration shaders

**Phase 3: Effects Pipeline**
-   [ ] GPU compute effects
-   [ ] Audio reactivity
-   [ ] Custom shader support

**Further Phases will include:**
-   **Control Systems:** MIDI, OSC, DMX, and web API support.
-   **Professional Video I/O:** NDI, DeckLink, Spout/Syphon integration.
-   **Advanced UI:** A polished, intuitive user interface for authoring and performance.

For a detailed breakdown, see the [ROADMAP](ROADMAP.md).

## 🛠️ Quick Start

### Prerequisites

**Rust Toolchain:**
```bash
# Install Rust 1.75 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**System Dependencies:**

**Ubuntu/Debian:**
```bash
sudo apt-get install -y \
  build-essential pkg-config \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libfontconfig1-dev libfreetype6-dev libasound2-dev
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows:**
- Install [Visual Studio 2022](https://visualstudio.microsoft.com/) with C++ development tools.

### Build and Run

```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Build (optimized release)
cargo build --release

# Run the application
cargo run --release

# Run tests
cargo test
```

## 🏗️ Architecture

MapFlow is organized as a Cargo workspace with specialized crates to ensure a clean separation of concerns:

```
crates/
├── mapmap-core/      # Domain model (Paint/Mapping/Shape)
├── mapmap-render/    # Graphics abstraction (wgpu backend)
├── mapmap-media/     # Video decode and playback
├── mapmap-ui/        # ImGui integration
└── mapmap/           # Main application binary
```

### Technology Stack

- **Language:** Rust 2021
- **Graphics:** `wgpu` (Vulkan/Metal/DX12 abstraction)
- **UI:** `egui` or `ImGui` (to be decided)
- **Media:** FFmpeg
- **Windowing:** `winit`

## 🤝 Contributing

This project is in active development. Contributions will be welcome as the core architecture stabilizes. Please see the [Contributing Guidelines](docs/02-CONTRIBUTING/CONTRIBUTING.md) for more details.

## 📄 License

MapFlow is licensed under the **GNU General Public License v3.0** (GPL-3.0). See [LICENSE](LICENSE) for the full license text.

## 🙏 Acknowledgments

- **Original MapMap Team** - For creating the foundational C++/Qt application and its concepts.
- **John Janik** - For the significant work on the initial Rust rewrite fork which inspired this project.
- **The Rust Community** - For building an amazing language and ecosystem.

# MapFlow

<!-- Dynamic CI/CD Status Badges -->
[![Build & Test](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build-and-test.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-01_build-and-test.yml)
[![Security Scan](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml)
[![Release](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09B_create-release.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-09B_create-release.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

> **Modern, High-Performance Projection Mapping Suite**

MapFlow is a professional-grade, open-source projection mapping system being completely rewritten in Rust. Originally a C++/Qt application (MapMap), MapFlow is being transformed into a modern, high-performance tool capable of competing with commercial solutions like Resolume Arena.

## 🎯 Vision

Projection mapping (also known as video mapping and spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. MapFlow aims to provide a professional, open-source alternative for artists, designers, and technical professionals who need powerful projection mapping capabilities without the cost of commercial software.

## 🚀 Project Status

**Current Phase: Phase 1 (Core Engine) - 🚧 IN PROGRESS**

MapFlow is a complete rewrite in Rust. The project is in its early stages, focusing on building a solid foundation for a high-performance, memory-safe, and modern projection mapping tool.

### From Legacy to Rust:
- **Memory Safety:** Eliminates entire classes of crashes common in live performance software.
- **Modern Graphics:** Utilizes `wgpu` for access to Vulkan, Metal, and DX12, moving beyond legacy OpenGL.
- **High Performance:** Built for 60fps+ at 4K with multiple outputs, leveraging Rust's zero-cost abstractions and fearless concurrency.
- **Cross-Platform:** Designed to run on Linux, macOS, and Windows.

## 🗺️ Roadmap

The development of MapFlow is planned in several phases, starting with the core engine and gradually adding more advanced features.

**Phase 1: Core Engine**
-   [ ] Layer system with transforms, opacity, and blend modes
-   [ ] Hardware-accelerated video decoding
-   [ ] Advanced playback controls (speed, direction, loop modes)
-   [ ] Still image and animated format support (PNG, JPG, GIF)

**Phase 2: Professional Multi-Projector System**
-   [ ] Multi-output rendering with synchronization
-   [ ] Bezier-based mesh warping
-   [ ] Edge blending and color calibration shaders

**Phase 3: Effects Pipeline**
-   [ ] GPU compute effects
-   [ ] Audio reactivity
-   [ ] Custom shader support

**Further Phases will include:**
-   **Control Systems:** MIDI, OSC, DMX, and web API support.
-   **Professional Video I/O:** NDI, DeckLink, Spout/Syphon integration.
-   **Advanced UI:** A polished, intuitive user interface for authoring and performance.

For a detailed breakdown, see the [ROADMAP](ROADMAP.md).

## 🛠️ Quick Start

### Prerequisites

**Rust Toolchain:**
```bash
# Install Rust 1.75 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**System Dependencies:**

**Ubuntu/Debian:**
```bash
sudo apt-get install -y \
  build-essential pkg-config \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libx11-dev libfontconfig1-dev libfreetype6-dev libasound2-dev
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows:**
- Install [Visual Studio 2022](https://visualstudio.microsoft.com/) with C++ development tools.

### Build and Run

```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Build (optimized release)
cargo build --release

# Run the application
cargo run --release

# Run tests
cargo test
```

## 🏗️ Architecture

MapFlow is organized as a Cargo workspace with specialized crates to ensure a clean separation of concerns:

```
crates/
├── mapmap-core/      # Domain model (Paint/Mapping/Shape)
├── mapmap-render/    # Graphics abstraction (wgpu backend)
├── mapmap-media/     # Video decode and playback
├── mapmap-ui/        # ImGui integration
└── mapmap/           # Main application binary
```

### Technology Stack

- **Language:** Rust 2021
- **Graphics:** `wgpu` (Vulkan/Metal/DX12 abstraction)
- **UI:** `egui` or `ImGui` (to be decided)
- **Media:** FFmpeg
- **Windowing:** `winit`

## 🤝 Contributing

This project is in active development. Contributions will be welcome as the core architecture stabilizes. Please see the [Contributing Guidelines](docs/02-CONTRIBUTING/CONTRIBUTING.md) for more details.

## 📄 License

MapFlow is licensed under the **GNU General Public License v3.0** (GPL-3.0). See [LICENSE](LICENSE) for the full license text.

## 🙏 Acknowledgments

- **Original MapMap Team** - For creating the foundational C++/Qt application and its concepts.
- **John Janik** - For the significant work on the initial Rust rewrite fork which inspired this project.
- **The Rust Community** - For building an amazing language and ecosystem.
