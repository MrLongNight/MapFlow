# VjMapper

[![CI](https://github.com/MrLongNight/VjMapper/actions/workflows/ci.yml/badge.svg)](https://github.com/MrLongNight/VjMapper/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

> **Modern, High-Performance Projection Mapping Suite**

VjMapper is a professional-grade, open-source projection mapping system being completely rewritten in Rust. Originally a C++/Qt application, VjMapper is being transformed into a modern, high-performance tool capable of competing with commercial solutions like Resolume Arena.

## 🎯 Vision

Projection mapping (also known as video mapping and spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. VjMapper aims to provide a professional, open-source alternative for artists, designers, and technical professionals who need powerful projection mapping capabilities without the cost of commercial software.

## 🚀 Project Status

**Current Phase: Phase 1 (Core Engine) - 🚧 IN PROGRESS**

VjMapper is a complete rewrite of the original C++/Qt application in Rust. The project is in its early stages, focusing on building a solid foundation for a high-performance, memory-safe, and modern projection mapping tool.

### From C++/Qt to Rust:
- **Memory Safety:** Eliminates entire classes of crashes common in live performance software.
- **Modern Graphics:** Utilizes `wgpu` for access to Vulkan, Metal, and DX12, moving beyond legacy OpenGL.
- **High Performance:** Built for 60fps+ at 4K with multiple outputs, leveraging Rust's zero-cost abstractions and fearless concurrency.
- **Cross-Platform:** Designed to run on Linux, macOS, and Windows.

## 🗺️ Roadmap

The development of VjMapper is planned in several phases, starting with the core engine and gradually adding more advanced features.

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

For a detailed breakdown, see the [Roadmap](docs/roadmap.md).

## 🛠️ Quick Start

### Prerequisites

**Rust Toolchain:**
```bash
# Install Rust 1.75 or later
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Audio Backend:**
VjMapper requires a functional audio backend for its audio-reactive features. Please follow the [Audio Setup Guide](docs/04-USER-GUIDE/AUDIO-SETUP.md) for platform-specific instructions.
