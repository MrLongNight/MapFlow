# MapFlow

**MapFlow** is a modular, node-based **VJ (Video Jockey) Software** built in **Rust** 🦀, designed for high-performance real-time visual synthesis and
projection mapping.

> 🚀 **CI/CD Status**: Verified (v3.0)

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

## 📦 Installation

See [INSTALLATION.md](docs/01-GETTING-STARTED/INSTALLATION.md).

## 📄 License

MIT / Apache 2.0
