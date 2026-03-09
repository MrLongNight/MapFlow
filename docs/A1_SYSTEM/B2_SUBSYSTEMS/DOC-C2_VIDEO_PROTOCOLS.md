# DOC-C2: Video I/O & Protokolle

## 1. NDI (Network Device Interface)
*   **Status**: Integration via `ndi-rs`.
*   **Funktion**: Empfang und Senden von Video-Streams über das lokale Netzwerk.
*   **Latenz**: Optimiert für < 100ms.

## 2. Spout & Syphon
*   **Spout (Windows)**: Texture-Sharing via DirectX/OpenGL Handles.
*   **Syphon (macOS)**: Analoges System für Mac.
*   **Vorteil**: Nahezu null Latenz und keine CPU-Belastung durch Kopieren.

## 3. HAP Video Codec
*   **Format**: Nutzt GPU-Texturkompression (BC1/BC3).
*   **Performance**: Minimale CPU-Last, da die GPU das Decoding übernimmt.
