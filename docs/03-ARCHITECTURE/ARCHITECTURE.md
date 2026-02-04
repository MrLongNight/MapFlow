# MapFlow Architecture Documentation

## Overview

MapFlow is a modern projection mapping system written in Rust, designed to compete with professional tools like Resolume Arena. This document describes the architecture implemented as of **Phase 6 (UI Migration Complete)**.

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Application                           │
│                       (mapmap binary)                        │
└───────────────┬─────────────────────────────────────────────┘
                │
    ┌───────────┼───────────┬───────────────┬──────────────┐
    │           │           │               │              │
    ▼           ▼           ▼               ▼              ▼
┌────────┐ ┌────────┐ ┌─────────┐ ┌──────────────┐ ┌──────────┐
│ Core   │ │ Render │ │  Media  │ │      UI      │ │ Control  │
│        │ │        │ │         │ │              │ │          │
└────────┘ └────┬───┘ └────┬────┘ └──────┬───────┘ └──────────┘
               │          │              │
               ▼          ▼              ▼
          ┌────────────────────────────────┐
          │        wgpu / FFmpeg           │
          │     (External Dependencies)    │
          └────────────────────────────────┘
```

### Crate Structure

**mapmap-core:**
- Domain model (Project, Layer, Mapping, Mesh)
- Module System (Nodes, Triggers)
- Audio Analysis Logic
- Pure Rust, no external dependencies

**mapmap-render:**
- Graphics abstraction layer
- `wgpu` backend implementation
- Texture pool management & Staging Buffer Pool
- Shader Graph execution
- Advanced rendering (Edge Blend, Color Calibration)

**mapmap-media:**
- Video decoding via `ffmpeg-next` and `libmpv2`
- HAP Codec support
- Media Playback State Machine
- Image Sequence & GIF support

**mapmap-ui:**
- `egui` integration (ImGui removed)
- Node-based "Module Canvas"
- Timeline V2
- Context-sensitive Inspector
- "Cyber Dark" Theme

**mapmap-control:**
- OSC server (Implemented)
- MIDI input/output (Implemented)
- Cue System
- Philips Hue Integration (Lighting)

**mapmap-io:**
- NDI Input/Output (In Progress)
- Spout Integration (Windows)

**mapmap-mcp:**
- Model Context Protocol server for AI integration

## Threading Model

### Current Architecture (Async/Parallel)

MapFlow utilizes a multi-threaded architecture with async channels for communication.

```
┌─────────────────┐    ┌──────────────────┐    ┌────────────────┐
│ Decode Threads  │    │ Upload Task      │    │ Render Loop    │
│ (Worker Pool)   │    │ (Async)          │    │ (Main Thread)  │
│ ┌────────────┐  │    │ ┌──────────────┐ │    │ ┌────────────┐ │
│ │  FFmpeg    │  │    │ │   Staging    │ │    │ │   wgpu     │ │
│ │  Decode    │──┼───▶│ │   Buffer     │─┼───▶│ │  Render    │ │
│ │            │  │    │ │   Upload     │ │    │ │            │ │
│ └────────────┘  │    │ └──────────────┘ │    │ └────────────┘ │
│                 │    │                  │    │       │        │
└─────────────────┘    └──────────────────┘    └───────┼────────┘
        │                      │                       │
        └──────────────────────┴───────────────────────┘
            Lock-Free Channels (crossbeam/tokio)
```

## Data Flow

### Video Playback Pipeline

```
1. Video File
   │
   ▼
2. FFmpeg/MPV Decoder
   │ (Decode to RGBA/YUV)
   ▼
3. DecodedFrame
   │ (Shared Memory)
   ▼
4. Texture Upload
   │ (Smart Staging Buffer Pool)
   ▼
5. GPU Texture
   │
   ▼
6. Compositor / Shader Graph
   │ (Effects, Blending)
   ▼
7. Output Rendering
   │ (Warping, Edge Blend)
   ▼
8. Display
```

## Graphics Pipeline

### wgpu Rendering Architecture

MapFlow uses `wgpu` for cross-platform GPU access (Vulkan, Metal, DX12).

**Key Components:**
- **Texture Pool:** Reuses GPU allocations to avoid expensive `create_texture` calls.
- **Staging Buffer Pool:** Optimizes CPU-to-GPU data transfer for video frames.
- **Shader Graph:** dynamically compiles WGSL shaders based on user node configurations.

## Error Handling

MapFlow uses a hierarchical error handling strategy:

- **CoreError:** Logic and data validation errors.
- **RenderError:** GPU device loss, shader compilation failures.
- **MediaError:** Decode failures, file not found.
- **ControlError:** OSC/MIDI parsing errors.

**Recovery:**
- **Device Lost:** The application attempts to recreate the wgpu surface and reload resources.
- **Media Errors:** The player enters an Error state but does not crash the application.

## Future Phases (Roadmap 2.0)

See `ROADMAP_2.0.md` for the active roadmap.

### Phase 7: Advanced Show Control
- Node-based Timeline (V3)
- Hybrid/Manual playback modes

### Phase 8: Multi-PC Architecture
- Distributed rendering via NDI
- Legacy Client support

### Phase 9: Lighting Integration
- DMX/Art-Net Output
