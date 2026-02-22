<div align="center">
  <img src="resources/app_icons/MapFlow_Logo_HQ-Full-M.png" alt="MapFlow Logo" width="500"/>

  # MapFlow
  ### High-Performance Real-Time Visual Synthesis & Projection Mapping

  [![Build & Quality](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-DevFlow_Job01_Validation.yml/badge.svg?branch=main)](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-DevFlow_Job01_Validation.yml)
  [![Security Analysis](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CI-02_security-scan.yml)
  [![Stable Release](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-MainFlow_Job03_Release.yml/badge.svg)](https://github.com/MrLongNight/MapFlow/actions/workflows/CICD-MainFlow_Job03_Release.yml)
  ![OS: Windows | Linux](https://img.shields.io/badge/OS-Windows%20%7C%20Linux-blue.svg)
  [![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
  [![Rust: 1.75+](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
  [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

  **MapFlow** is a next-generation, modular **VJ (Video Jockey) Software** engineered for high-performance visual synthesis, real-time effects, and professional projection mapping. Built with the speed and safety of **Rust**, it empowers artists to create immersive visual experiences with unprecedented flexibility.
</div>

---

## 🚀 Vision

In an era of complex visual performances, MapFlow bridges the gap between ease of use and professional power. By utilizing a **node-based architecture**, every parameter becomes a playground for automation, audio-reactivity, and external control.

## ✨ Core Features

### 🧩 Modular Node System
Design complex visual flows by connecting video sources, generative shaders, and real-time filters. Every node property is a control target for our unified modulation system.

### ⚡ Professional Rendering Engine
Powered by **WGPU** and the **Bevy Engine**, MapFlow delivers low-latency, hardware-accelerated rendering.
* **Multi-Layer Composition**: Advanced blend modes and hierarchical grouping.
* **3D & Particle Integration**: Native Bevy support for stunning volumetric effects and 3D scenes.
* **LUT Color Grading**: Industry-standard `.cube` support for cinematic looks.

### 🔊 Deep Audio Reactivity
Our **AudioAnalyzer V2** tracks 9 frequency bands, RMS volume, and peak detection in real-time, allowing visuals to dance perfectly to the beat.

### 📐 Precision Projection Mapping
* **Bezier Warping**: Flexible mesh deformation for complex surfaces.
* **Edge Blending**: Seamless multi-projector setups with per-output gamma correction.
* **Advanced Masking**: Integrated shape and file-based masking tools.

### 🎛️ Unified Control
Seamlessly integrate with your performance setup via **OSC**, **MIDI**, and **Ableton Link**. Our built-in **Jules AI assistant** is always ready to help you extend the software's capabilities.

---

## 🛠️ Technology Stack

| Component | Technology |
| :--- | :--- |
| **Core** | [Rust 🦀](https://www.rust-lang.org/) (High-performance, Thread-safe) |
| **Graphics** | [WGPU](https://wgpu.rs/) (Modern WebGPU-based hardware acceleration) |
| **3D Engine** | [Bevy](https://bevyengine.org/) (Data-driven ECS engine) |
| **Interface** | [egui](https://github.com/emilk/egui) (Blazing fast immediate mode UI) |
| **Video/Audio** | FFmpeg (via `ffmpeg-next`), CPAL (Cross-platform audio) |
| **Protocol** | [Model Context Protocol](https://modelcontextprotocol.io/) (AI integration) |

---

## 🚦 Quick Start

### 1. Requirements
* **Rust**: [Install latest stable version](https://rustup.rs/)
* **FFmpeg**: System-wide installation required for video decoding.
* **NDI (Optional)**: For network video I/O.

### 2. Run from Source
```bash
# Clone the repository
git clone https://github.com/MrLongNight/MapFlow.git
cd MapFlow

# Run the application (Release mode recommended for performance)
cargo run --release
```

### 3. Usage
* Check the [**Quick Start Guide**](docs/user/getting-started/QUICK-START.md) to create your first composition.
* Explore the [**User Manual**](docs/user/manual/) for detailed control explanations.

---

## 📚 Documentation

Explore our comprehensive guides in the [`docs/`](docs/README.md) directory:

* 📖 [**User Guide**](docs/user/manual/): Interface layout, keyboard shortcuts, and performance tips.
* 👨‍💻 [**Developer Portal**](docs/dev/): Architecture overview, coding standards, and build instructions.
* 🗺️ [**Project Roadmap**](ROADMAP.md): Current status and upcoming Phase 1.0 release goals.

---

## 🤝 Contributing

We welcome contributions from visual artists and developers alike! Please read our [**Contributing Guidelines**](CONTRIBUTING.md) and check our [**GitHub Issues**](https://github.com/MrLongNight/MapFlow/issues) for open tasks.

## 📄 License

MapFlow is licensed under **GPL-3.0**. See the [LICENSE](LICENSE) file for more information.

---
<div align="center">
  Created with ❤️ by the MapFlow Contributors.
</div>
