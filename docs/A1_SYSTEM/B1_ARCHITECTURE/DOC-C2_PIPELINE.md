# DOC-C2: Render Pipeline & Frame Orchestration

Dieses Dokument beschreibt den Weg eines Frames vom Decoder zum Display.

## 1. Threading-Modell
SubI nutzt eine asynchrone Pipeline, um Frame-Drops zu vermeiden:
*   **Decode-Thread**: MPV/FFmpeg dekomprimiert Video-Frames.
*   **Upload-Thread**: Lädt CPU-Daten via Staging-Buffer in GPU-Texturen.
*   **Main-Thread**: Orchestriert WGPU Render-Passes und UI.

## 2. Rendering Phasen
1.  **Compositor**: Mischt Layer basierend auf Blend-Modes (`shaders/blend_modes.wgsl`).
2.  **Effect Chain**: Wendet Shader-Effekte (Blur, Glitch, etc.) sequentiell an.
3.  **Warping/Mapping**: Transformiert das fertige Bild auf die Projektionsgeometrie (`shaders/mesh_warp.wgsl`).
4.  **Edge Blending**: Erzeugt weiche Übergänge bei Multi-Beamer-Setups.

## 3. GPU Techniken
*   **Texture Pool**: Wiederverwendung von GPU-Texturen zur Performance-Optimierung.
*   **Staging Buffers**: Expliziter Copy-Pfad für minimalen Main-Thread-Blocking.
*   **HAP Codec**: Native Unterstützung für GPU-komprimierte Texturen (BC1/BC3).
