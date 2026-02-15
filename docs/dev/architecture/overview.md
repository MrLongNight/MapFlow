# MapFlow Architektur Übersicht

## System-Design

MapFlow basiert auf einer modernen, modularen Architektur, die **Rust** für Performance und Sicherheit, **Bevy** als ECS-Engine und **WGPU** für das Rendering nutzt.

### Kern-Komponenten (Crates)

Das Projekt ist als Cargo Workspace organisiert, wobei jede Funktionalität in eigene Crates gekapselt ist:

| Crate | Beschreibung |
|-------|--------------|
| `mapmap` | **Main Application**. Der Einstiegspunkt. Initialisiert Plugins und startet die App. |
| `mapmap-core` | **Logik-Kern**. Enthält Datenstrukturen (Layers, Mappings, Paints), State-Management und die Geschäftslogik. Unabhängig von UI und Rendering. |
| `mapmap-render` | **Grafik-Engine**. Zuständig für die WGPU-Pipeline, Shader-Verwaltung, Compositing, Warping und Effekte. |
| `mapmap-ui` | **Benutzeroberfläche**. Basiert auf `egui`. Enthält alle Panels, den Node-Editor und die Timeline. |
| `mapmap-media` | **Medien-Playback**. Video-Decoding (FFmpeg/MPV), Audio-Streaming und Bild-Loading. |
| `mapmap-control` | **Externe Steuerung**. OSC, MIDI, WebSocket Server zur Fernsteuerung der App. |
| `mapmap-io` | **Input/Output**. NDI, Spout, Dateisystem-Zugriffe. |
| `mapmap-mcp` | **AI Interface**. Model Context Protocol Server für die Integration von AI-Agenten. |

---

## Datenfluss & Pipeline

### 1. Layer & Mapping System (`mapmap-core`)
Das visuelle Ergebnis entsteht durch die Kombination von:
*   **Paints**: Die Quellen (Videos, Bilder, Shader, Live-Inputs).
*   **Layers**: Container für Paints mit Transformation (Pos, Rot, Scale), Opacity und Blend-Modes.
*   **Mappings**: Die Projektionsflächen (Meshes, Quads), auf die Layer projiziert werden.
*   **Trigger**: Signale (Audio, MIDI, Random), die Parameter steuern via Node-Graph.

### 2. Render Pipeline (`mapmap-render`)
Die Rendering-Engine arbeitet in mehreren Pässen:
1.  **Media Upload**: Dekodierte Video-Frames werden in GPU-Texturen geladen.
2.  **Layer Composition**: Layer werden basierend auf Blend-Modes und Masken in einen Offscreen-Buffer gerendert.
3.  **Effect Chain**: Post-Processing-Effekte (Blur, Color-Grade, Glitch) werden angewendet.
4.  **Mapper / Warping**: Der fertige Composition-Buffer wird auf die 3D-Meshes (Mappings) texturiert.
5.  **Output Processing**: Edge-Blending, Color-Calibration und Gamma-Korrektur werden final angewendet.
6.  **Display**: Das Ergebnis wird im Projektor-Fenster (und Preview) angezeigt.

### 3. Modul-System & Node-Graph
Die Logik ist Node-basiert (`module_canvas`).
*   **Flow**: Trigger-Signale fließen durch Nodes -> Modulatoren -> Ziel-Parameter.
*   **Evaluierung**: Das System evaluiert den Graphen jeden Frame, um Parameter-Updates zu berechnen.

---

## Audio-Integration

MapFlow nutzt `cpal` für Low-Level Audio I/O.
*   **Analyse**: FFT (Fast Fourier Transform) zur Frequenzanalyse.
*   **Reactivity**: Audio-Signale (Bass, Mids, Highs) können direkt Parameter in der Render-Pipeline steuern.
