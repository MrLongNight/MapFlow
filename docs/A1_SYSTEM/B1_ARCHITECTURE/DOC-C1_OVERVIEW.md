# DOC-B1: MapFlow System Architecture

Dieses Dokument dient als zentrale technische Referenz für die interne Funktionsweise von MapFlow. Es beschreibt das System-Design, die Crate-Hierarchie und den Datenfluss.

## 1. System-Design & Crates

MapFlow basiert auf einer modernen, modularen Architektur in **Rust**, die **Bevy** als ECS-Engine und **WGPU** für das Rendering nutzt. Das Projekt ist als Cargo Workspace organisiert.

### Crate-Ökosystem

```mermaid
graph TD
    %% Crates
    Main[mapflow] --> UI[mapflow-ui]
    Main --> Bevy[mapflow-bevy]
    Main --> Media[mapflow-media]
    Main --> Control[mapflow-control]
    Main --> Render[mapflow-render]

    UI --> Core[mapflow-core]
    Render --> Core
    Bevy --> Core
    Control --> Core
    Media --> IO[mapflow-io]
    IO --> Core

    %% Externe Ressourcen
    Render --> Shaders[WGSL Shaders]
    Media --> FFmpeg[FFmpeg / MPV]
    Control --> Hue[Philips Hue API]
```

| Crate | Logische Rolle | Wichtigste Typen / Zuständigkeiten |
| :--- | :--- | :--- |
| `mapflow` | **Main App** | Einstiegspunkt, Event-Loop, App-State Orchestrierung. |
| `mapflow-core` | **Logik-Kern** | Datenmodelle (Layer, Mapping, Paint), Graph-Evaluierung, Math. |
| `mapflow-render` | **Renderer** | WGPU-Abstraktion, Shader-Verwaltung, Compositing, Texture-Pooling. |
| `mapflow-ui` | **User Interface** | Egui-Implementierung, Panels, Node-Editor, Timeline. |
| `mapflow-media` | **Media Engine** | Frame-Pipeline, Video-Decoding (FFmpeg), Bild-Loading. |
| `mapflow-control` | **Peripherie** | MIDI, OSC, Philips Hue, Shortcuts. |
| `mapflow-io` | **I/O & Netz** | NDI, Spout, Datei-System, Persistenz. |
| `mapflow-bevy` | **3D/Particles** | Bevy ECS Integration für komplexe 3D-Inhalte. |
| `mapflow-mcp` | **AI Interface** | Model Context Protocol Server für Agenten-Integration. |

---

## 2. Globaler Frame-Loop

MapFlow trennt strikt zwischen Logik-Update (fest 60Hz) und Render-Update (VSync).

### Phase A: Logic Update (`logic.rs`)
1. **Input Sampling**: Gather MIDI/OSC/Keyboard Events.
2. **Audio Analysis**: FFT-Berechnung (9 Bänder) via `AudioAnalyzer`.
3. **Graph Evaluation**: `ModuleEvaluator` berechnet Knoten-Zustände, Trigger und Signalfluss.
4. **Smoothing**: Anwendung von Attack/Release-Filtern auf reaktive Parameter.
5. **Command Generation**: Erzeugung von `SourceCommands` und `RenderOps`.

### Phase B: Render Update (`render.rs`)
1. **Texture Preparation**: Upload frischer Frames in GPU-Texturen.
2. **Effect Processing**: Abarbeitung der WGSL-Shader-Ketten pro Layer.
3. **Compositing**: Finale Mischung aller Layer auf die Ziel-Outputs (Warping/Masking).
4. **UI Overlay**: Egui-Rendering als letzter Pass über Output 0.

---

## 3. Datenfluss: Die PAP-Kette

Der fachliche Datenfluss folgt dem Prinzip:
`TRIGGER → SOURCE → MODULIZER → LAYER → OUTPUT`

*   **Trigger**: Signale (Audio, MIDI, Random), die Parameter steuern.
*   **Source**: Video, Bild, Shader-Generator oder Live-Input.
*   **Modulizer**: Effekte, Blend-Modes und Masken.
*   **Layer**: Gruppierung und räumliche Anordnung.
*   **Output**: Physikalische Ausgänge (Projektoren) inkl. Edge-Blending.

---

## 4. Render-Pipeline & Threading

Aktuell nutzt MapFlow ein asynchrones Modell für Medien-Frames:
- **Decode-Thread**: Erzeugt Frames aus Video-Quellen.
- **Upload-Thread**: Lädt Daten via Staging-Buffer in GPU-Texturen (WGPU).
- **Render-Thread**: Nutzt die Texturen für die Komposition.

Synchronisation erfolgt über bounded `crossbeam_channels`, um Backpressure zu kontrollieren.

---

## 5. UI-Architektur

Das UI basiert auf `egui` (Retained Mode Style).
- **Unified Inspector**: Kontextsensitive Steuerung, die sich automatisch an das selektierte Element (Module, Layer, Output) anpasst.
- **Module Canvas**: Custom-Knoteneditor für die visuelle Programmierung (keine externen Node-Libs).

---
*Referenzen: ../DOC-A1_MODULE_TREE.md (Physische Struktur)*
