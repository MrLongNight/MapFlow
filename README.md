<div align="center">
  <img src="resources/app_icons/MapFlow_Logo_HQ-Full-M.png" alt="MapFlow Logo" width="500"/>
</div>

# MapFlow

**MapFlow** is a modular, node-based **VJ (Video Jockey) Software** built in **Rust** 🦀, designed for high-performance real-time visual synthesis and
projection mapping.

> 🚀 **CI/CD Status**: Verified (v3.0)
[![CICD-DevFlow: Job01 Validation](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-DevFlow_Job01_Validation.yml/badge.svg?branch=main)](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-DevFlow_Job01_Validation.yml)

## ✨ Key Features

* **Modular Node System**: Connect video, image, and effect nodes.
* **Real-time Rendering**: Powered by `wgpu` and `bevy`.
* **Projection Mapping**: Advanced warping and masking.
* **Jules AI Integration**: Built-in AI coding assistant.

## 🛠️ Tech Stack

* **Core**: Rust
* **Engine**: Bevy (via `mapmap-bevy`)
* **UI**: eframe / egui
* **Graphics**: wgpu
* **Audio**: cpal, rodio
* **AI**: Model Context Protocol (`mapmap-mcp`)

## 📦 Workspace Modules

* `mapmap`: Main application binary
* `mapmap-core`: Core data structures and logic
* `mapmap-ui`: UI implementation (egui)
* `mapmap-render`: WGPU rendering engine
* `mapmap-bevy`: Bevy engine integration (3D/Particles)
* `mapmap-mcp`: MCP Server for AI integration
* `mapmap-media`: Media decoding and playback
* `mapmap-control`: OSC/MIDI input handling
* `mapmap-io`: NDI/Spout IO
* `mapmap-ffi`: C/C++ Foreign Function Interface

## 📚 Documentation

Detailed documentation is available in the [`docs/`](docs/README.md) directory:

* [**User Guide**](docs/user/README.md): Features and controls.
* [**Developer Guide**](docs/dev/README.md): Setup and guidelines.
* [**Architecture**](docs/dev/architecture/ARCHITECTURE.md): System design.
* [**Roadmap**](ROADMAP.md): Project status and plans.

## ⚙️ CI/CD

MapFlow uses a comprehensive GitHub Actions workflow for validation and release management.
See [CI/CD Workflow](docs/project/cicd/README_CICD.md) for details.

## 📦 Installation

See [INSTALLATION.md](docs/user/getting-started/INSTALLATION.md).

## 📄 License

MIT / Apache 2.0
