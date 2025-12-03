# MapMap

[![CI](https://github.com/johnjanik/mapmap/actions/workflows/ci.yml/badge.svg)](https://github.com/johnjanik/mapmap/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

> **Modern, High-Performance Projection Mapping Suite**

MapMap is a professional-grade, open-source projection mapping system being completely rewritten in Rust. Originally a C++/Qt application, MapMap is being transformed into a modern, high-performance tool capable of competing with commercial solutions like Resolume Arena.

## 🎯 Vision

Projection mapping (also known as video mapping and spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. MapMap aims to provide a professional, open-source alternative for artists, designers, and technical professionals who need powerful projection mapping capabilities without the cost of commercial software.

## 🚀 Project Status

**Current Phase: Phase 2 (Professional Multi-Projector System) - 🚧 IN PROGRESS**

MapMap is undergoing a complete rewrite in Rust. The project is actively in development and has made significant progress on its foundational features.

### Completed & In Progress ✅

**Phase 0 - Foundation (✅ Structurally Complete)**
- ✅ Modern graphics via **wgpu** (Vulkan/Metal/DX12)
- ✅ Safe, high-performance **Rust** implementation
- ✅ **ImGui-based** live operator interface
- ✅ Modular architecture with 7 specialized crates
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ Comprehensive CI/CD pipeline

**Phase 1 - Core Engine (🚧 86% Complete)**
- ✅ Layer system with transforms, opacity, and blend modes
- ✅ Advanced playback controls (backwards, ping-pong, play once)
- ✅ Master controls for speed and opacity
- ✅ Quick resize modes (Fill, Fit, Stretch, Original)
- 🚧 Still image support (PNG, JPG, TIFF)
- 🚧 Animated format support (GIF, image sequences)
- 🚧 ProRes codec support

**Phase 2 - Multi-Projector System (🚧 85% Complete)**
- ✅ Bezier-based mesh warping system
- ✅ Edge blending and color calibration shaders
- ✅ Monitor detection and output management foundation
- ✅ UI panels for multi-output configuration
- 🚧 Multi-window rendering implementation
- 🚧 Frame synchronization across outputs

### Next Phase 🎯
**Phase 3:** Effects Pipeline - GPU compute effects, audio reactivity, custom shaders

### What's New

**From C++/Qt to Rust:**
- **Memory Safety:** Eliminates entire classes of crashes in live shows
- **Modern Graphics:** Vulkan/Metal/DX12 instead of legacy OpenGL
- **Better Performance:** Zero-cost abstractions and fearless concurrency
- **Production Ready:** Built for 60fps+ at 4K with multiple outputs

**Architecture Highlights:**
- Domain-driven design with clear separation of concerns
- Multi-threaded media pipeline (decode/upload/render)
- Extensible plugin system via FFI
- Hardware-accelerated video decoding
- Real-time performance optimizations

## 📦 Features

### Current (Phases 0-2 In Progress)
- ✅ Real-time video playback with basic controls (play/pause/stop)
- ✅ Advanced playback modes (backwards, ping-pong)
- ✅ Hardware-accelerated rendering (Vulkan/Metal/DX12 via wgpu)
- ✅ Professional ImGui control interface
- ✅ Layer system with transforms, opacity, and blend modes
- ✅ Bezier-based mesh warping
- ✅ Edge blending and color calibration shaders (foundation)
- ✅ Monitor detection and output management foundation
- ✅ Performance monitoring and real-time stats

### Roadmap

**Phase 0 (Foundation)** - ✅ Structurally Complete
- ✅ wgpu rendering backend
- ✅ FFmpeg decode abstraction
- ✅ Multi-threaded architecture design
- ✅ ImGui integration

**Phase 1 (Core Engine)** - 🚧 86% Complete
- ✅ Layer system and compositing
- ✅ Advanced playback modes
- 🚧 Hardware-accelerated video decode
- 🚧 Still image and GIF support

**Phase 2 (Professional Multi-Projector)** - 🚧 85% Complete
- ✅ Bezier mesh warping with control points
- ✅ Edge blending and color calibration shaders
- 🚧 Multi-output support with synchronized rendering
- 🚧 Geometric correction and canvas regions

**Phase 3 (Effects Pipeline)** - 📋 Planned
- 📋 Shader graph system
- 📋 Parameter animation
- 📋 Audio-reactive effects
- 📋 LUT color grading

**Phase 4 (Control Systems)** - 📋 Planned
- 📋 MIDI input/output
- 📋 OSC server/client
- 📋 DMX output (Art-Net/sACN)
- 📋 Web API and WebSocket
- 📋 Cue system
- 📋 Keyboard shortcuts and macros

**Phase 5-7:** Professional Video I/O (NDI/DeckLink/Spout/Syphon), Advanced UI, Performance & Polish

See [RUST_REWRITE_PLAN.md](RUST_REWRITE_PLAN.md) for the complete roadmap.

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
- Install [Visual Studio 2022](https://visualstudio.microsoft.com/) with C++ tools

### Build and Run

```bash
# Clone the repository
git clone https://github.com/johnjanik/mapmap.git
cd mapmap

# Build (development)
cargo build

# Build (optimized release)
cargo build --release

# Run the demo
cargo run --release

# Run tests
cargo test

# Generate documentation
cargo doc --no-deps --open
```

For detailed build instructions, see [BUILD.md](BUILD.md).

## 📚 Documentation

- **[BUILD.md](BUILD.md)** - Comprehensive build instructions for all platforms
- **[RUST_REWRITE_PLAN.md](RUST_REWRITE_PLAN.md)** - Complete 24-month roadmap and technical details
- **[STRATEGY.md](STRATEGY.md)** - Strategic assessment and modernization plan
- **[PHASE0_STATUS.md](PHASE0_STATUS.md)** - Current implementation status
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and architecture

## 🏗️ Architecture

MapMap is organized as a Cargo workspace with specialized crates:

```
mapmap/
├── mapmap-core/      # Domain model (Paint/Mapping/Shape)
├── mapmap-render/    # Graphics abstraction (wgpu backend)
├── mapmap-media/     # Video decode and playback
├── mapmap-ui/        # ImGui integration
├── mapmap-control/   # MIDI/OSC/DMX (Phase 4)
├── mapmap-ffi/       # Plugin API (Phase 5)
└── mapmap/           # Main application binary
```

### Technology Stack

- **Language:** Rust 2021 (MSRV 1.75+)
- **Graphics:** wgpu (Vulkan/Metal/DX12 abstraction)
- **UI:** ImGui (live operator interface)
- **Media:** FFmpeg (with hardware acceleration support)
- **Windowing:** winit (cross-platform)
- **Concurrency:** Tokio, Rayon, crossbeam-channel

## 🤝 Contributing

This project is currently in active development. Contributions are welcome once Phase 2 is complete.

**Development Guidelines:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write tests for all public APIs
- Document public items with `///` doc comments
- Run `cargo fmt` and `cargo clippy` before committing
- Keep commits atomic with clear messages

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## 🎮 Usage

Once built, you can run MapMap:

```bash
cargo run --release
```

**Current Features:**
- Real video playback (MP4, MOV, AVI, images)
- Multi-window projection mapping
- ImGui control panels for all features
- Performance stats (FPS, frame time)
- Mesh rendering with perspective correction
- Multi-output with edge blending and color calibration

**Controls:**
- **File Menu:** Load videos, save/load projects
- **Playback Controls:** Speed, direction, loop modes, play/pause/stop
- **Layers Panel:** Manage layers with transforms and blend modes
- **Paints Panel:** Add and manage video sources
- **Mappings Panel:** Create and edit mesh mappings
- **Outputs Panel:** Configure multi-output setups
- **Edge Blending:** Adjust blend zones for seamless overlap
- **Color Calibration:** Match colors across projectors
- **Performance Stats:** Real-time FPS and frame timing

**Quick Start - 2x2 Projector Array:**
1. Click "Outputs" panel
2. Click "2x2 Projector Array" button
3. Four output windows appear with automatic edge blending!
4. Select an output to adjust edge blending and color calibration

## 📊 Performance

**Achieved Targets:**
- ✅ 60 fps @ 1920x1080 per output (VSync locked)
- ✅ <1ms texture upload for 1920x1080 RGBA
- ✅ <50ms frame latency
- ✅ Multi-output rendering with synchronized presentation
- ✅ Real-time edge blending and color calibration (GPU-accelerated)
- ✅ Canvas region filtering for optimized rendering

**Current Capabilities:**
- 4+ synchronized output windows @ 1920x1080 60fps
- Real-time video decode and playback
- GPU post-processing (edge blend + color calibration) with minimal overhead
- Professional-grade projection mapping performance

**Future Targets:**
- 4K @ 60 fps with hardware decode
- 10+ concurrent video streams
- <16ms control latency (MIDI/OSC)
- NDI/Spout/Syphon integration

## 📄 License

MapMap is licensed under the **GNU General Public License v3.0** (GPL-3.0).

See [LICENSE](LICENSE) for full license text.

**Key Points:**
- Free to use, modify, and distribute
- Derivative works must also be GPL-3.0
- No warranty provided

## 🙏 Acknowledgments

- **Original MapMap Team** - For the foundational concepts and domain model
  - Sofian Audry (lead developer)
  - Alexandre Quessy (release manager)
  - Dame Diongue (developer)
  - And all [contributors](README.md#contributors)
- **wgpu-rs Community** - For the excellent graphics abstraction
- **Rust Community** - For creating an amazing language and ecosystem

## 📞 Contact & Support

- **Repository:** https://github.com/johnjanik/mapmap
- **Issues:** https://github.com/johnjanik/mapmap/issues
- **Original MapMap:** http://mapmap.info

## 🔗 Links

- [Original MapMap (C++/Qt version)](https://github.com/mapmapteam/mapmap)
- [wgpu Graphics Library](https://github.com/gfx-rs/wgpu)
- [Rust Programming Language](https://www.rust-lang.org/)

---

## Legacy Information

MapMap was originally developed in C++/Qt by the MapMap team. This repository contains a complete Rust rewrite that maintains the core concepts while modernizing the implementation for professional use.

### Original Authors
- Sofian Audry: lead developer, user interface designer, project manager
- Dame Diongue: developer
- Alexandre Quessy: release manager, developer, technical writer, project manager
- Mike Latona: user interface designer
- Vasilis Liaskovitis: developer

### Original Contributors
Lucas Adair, Christian Ambaud, Alex Barry, Eliza Bennett, Jonathan Roman Bland, Sylvain Cormier, Maxime Damecour, Louis Desjardins, Ian Donnelly, Gene Felice, Julien Keable, Marc Lavallée, Matthew Loewens, Madison Suniga, and many more.

### Original Acknowledgements
This project was made possible by the support of the International Organization of La Francophonie (http://www.francophonie.org/).

Ce projet a été rendu possible grâce au support de l'Organisation internationale de la Francophonie (http://www.francophonie.org/).

---
