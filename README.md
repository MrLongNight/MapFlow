# MapFlow

**MapFlow** is a modular, node-based **VJ (Video Jockey) Software** built in **Rust** 🦀, designed for high-performance real-time visual synthesis and projection mapping.

> 🚀 **CI/CD Status**: Verified (v3.0)

## ✨ Key Features
*   **Modular Node System**: Connect video, image, and effect nodes.
*   **Real-time Rendering**: Powered by `wgpu` and `bevy`.
*   **Projection Mapping**: Advanced warping and masking.
*   **Jules AI Integration**: Built-in AI coding assistant.

## 🛠️ Tech Stack
*   **Core**: Rust
*   **Engine**: Bevy (via `mapmap-bevy`)
*   **UI**: eframe / egui
*   **Graphics**: wgpu
*   **Audio**: cpal, rodio

## 📂 Project Structure
*   **mapmap**: Main application binary.
*   **mapmap-core**: Core data structures and logic.
*   **mapmap-ui**: User interface (egui).
*   **mapmap-render**: WGPU-based rendering engine.
*   **mapmap-bevy**: Bevy engine integration for 3D/Generative Art.
*   **mapmap-media**: Media decoding (ffmpeg/mpv) and playback.
*   **mapmap-control**: Input control (OSC, MIDI).
*   **mapmap-io**: I/O support (NDI, Spout).
*   **mapmap-mcp**: Model Context Protocol (MCP) server integration.

## 📦 Installation
See [SETUP_GUIDE.md](docs/08-TECHNICAL/SETUP_GUIDE.md).

## 📄 License
MIT / Apache 2.0
