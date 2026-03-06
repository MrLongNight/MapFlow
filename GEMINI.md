# GEMINI.md - Mapflow Projektkontext

Dieses Dokument dient als zentrale Wissensbasis für Maestro-Agenten im Mapflow-Projekt.

## Projektstruktur

Das Projekt ist als Cargo Workspace organisiert:

-   **`crates/`**: Quellcode der verschiedenen Module.
    -   `mapmap-core`: Kernlogik, mathematische Hilfsmittel, Datenmodelle.
    -   `mapmap-render`: Renderer-Implementierung (Bevy-basiert).
    -   `mapmap-ui`: Benutzeroberfläche (egui/Bevy).
    -   `mapmap-io`: Datei-I/O, OSC-Kommunikation.
    -   `mapmap-bevy`: Bevy-spezifische Integrationen.
    -   `mapmap-control`: Steuerung und Automatisierung.
    -   `mapmap-media`: Video- und Audio-Handling (FFmpeg).
    -   `mapmap-ffi`: Fremdsprachen-Interfaces.
    -   `mapmap-mcp`: Model Context Protocol Integration.
-   **`shaders/`**: WGSL Shader-Dateien für Effekte und Rendering.
    -   `effect_*.wgsl`: Verschiedene visuelle Effekte.
    -   `mesh_warp.wgsl`: Shader für Mesh-Deformation.
-   **`assets/`**: Statische Ressourcen (Icons, Bilder).
-   **`scripts/`**: Automatisierungsskripte für Build, Test und CI.
-   **`docs/`**: Projekt- und Entwicklerdokumentation.

## Architektur-Prinzipien

1.  **ECS-First**: Fast alle Logik sollte in Bevy-Systemen organisiert sein.
2.  **Modularität**: Crates sollten lose gekoppelt sein.
3.  **Hardware-Beschleunigung**: Nutzung von GPU-Shadern für visuelle Effekte ist Priorität.
4.  **Performance**: Kritische Pfade (Rendering, Video-Decoding) müssen hochoptimiert sein.

## Validierung (Validation Gates)

Für Mapflow gelten über `cargo check` hinausgehende Anforderungen:

1.  **Shader-Validierung**: Jede Änderung an `.wgsl`-Dateien muss mit `naga` (oder einem entsprechenden Skript) validiert werden.
2.  **Crate-Abhängigkeiten**: GUI-Logik darf niemals in `mapmap-core` landen.
3.  **Cross-Platform**: Achte auf Windows/Linux/macOS Kompatibilität, insbesondere bei Pfaden und FFI.

## Befehle für Maestro

- **Shader Check**: `naga path/to/shader.wgsl` (falls naga installiert ist)
- **Full Workspace Build**: `cargo build --workspace`
- **Lint Check**: `cargo clippy --workspace -- -D warnings`
