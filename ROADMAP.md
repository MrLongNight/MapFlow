# VjMapper – Vollständige Roadmap und Feature-Status

> **Version:** 1.2  
> **Stand:** 2025-12-16  
> **Zielgruppe:** @jules und Entwickler-Team  
> **Projekt-Version:** 0.1.0

---

## 📋 Inhaltsverzeichnis

1. [Feature-Status-Übersicht](#feature-status-übersicht)
2. [Architektur und Crate-Übersicht](#architektur-und-crate-übersicht)
3. [Arbeitspakete für @jules](#arbeitspakete-für-jules)
4. [Task-Gruppen (Adaptiert für Rust)](#task-gruppen-adaptiert-für-rust)
5. [Implementierungsdetails nach Crate](#implementierungsdetails-nach-crate)
6. [Technologie-Stack und Entscheidungen](#technologie-stack-und-entscheidungen)
7. [Build- und Test-Strategie](#build--und-test-strategie)

---

## Feature-Status-Übersicht

### Core / Layer / Mapping System

- ✅ **Layer-System** (`mapmap-core/src/layer.rs`)
  - ✅ Transform-System (Position, Rotation, Scale)
  - ✅ Opacity-Steuerung (0.0-1.0)
  - ✅ Blend-Modi (Normal, Add, Multiply, Screen, Overlay, etc.)
  - ✅ ResizeMode (Fill, Fit, Stretch, Original)
  - ✅ LayerManager für Komposition
  - ✅ Hierarchisches Layer-System

- ✅ **Mapping-System** (`mapmap-core/src/mapping.rs`)
  - ✅ Mapping-Hierarchie (Paint → Mapping → Mesh)
  - ✅ MappingManager für Verwaltung
  - ✅ Mapping-IDs und Referenzen

- ✅ **Mesh-System** (`mapmap-core/src/mesh.rs`)
  - ✅ MeshVertex mit UV-Koordinaten
  - ✅ BezierPatch für Warping
  - ✅ Keystone-Korrektur
  - ✅ MeshType (Quad, Grid, Custom)

- ✅ **Paint-System** (`mapmap-core/src/paint.rs`)
  - ✅ Paint als Basis-Datenstruktur
  - ✅ Media-Source-Integration

### Rendering (Compositor / Edge-Blend / Color-Calib / Mesh / Oszillator / Effekt-Chain)

- ✅ **Compositor** (`mapmap-render/src/compositor.rs`)
  - ✅ Multi-Layer-Komposition
  - ✅ Blend-Modi-Unterstützung
  - ✅ GPU-beschleunigte Rendering-Pipeline
  - ✅ Texture-Caching und Upload-Optimierung

- ✅ **Edge-Blend-Renderer** (`mapmap-render/src/edge_blend_renderer.rs`)
  - ✅ GPU-Shader-basiertes Edge-Blending
  - ✅ Gamma-Korrektur
  - ✅ Blend-Zonen-Konfiguration
  - ✅ Multi-Projektor-Unterstützung
  - ✅ WGSL-Shader: `shaders/edge_blend.wgsl`

- ✅ **Color-Calibration-Renderer** (`mapmap-render/src/color_calibration_renderer.rs`)
  - ✅ Per-Output-Farbkalibrierung
  - ✅ RGB-Gain/Offset-Steuerung
  - ✅ Gamma-Kurven
  - ✅ WGSL-Shader: `shaders/color_calibration.wgsl`

- ✅ **Mesh-Renderer** (`mapmap-render/src/mesh_renderer.rs`)
  - ✅ Bezier-basiertes Mesh-Warping
  - ✅ GPU-Vertex-Transformation
  - ✅ Texture-Mapping auf Meshes
  - ✅ WGSL-Shader: `shaders/mesh_warp.wgsl`

- ✅ **Oscillator-Renderer** (`mapmap-render/src/oscillator_renderer.rs`)
  - ✅ GPU-basierte Oszillator-Simulation
  - ✅ Distortion-Effekte
  - ✅ WGSL-Shader: `shaders/oscillator_simulation.wgsl`, `shaders/oscillator_distortion.wgsl`

- ✅ **Blend-Modi-Shader** (`shaders/blend_modes.wgsl`)
  - ✅ 10+ Blend-Modi implementiert (Normal, Add, Multiply, Screen, Overlay, SoftLight, HardLight, ColorDodge, ColorBurn, Difference)

- ✅ **LUT-Color-Grading** (`shaders/lut_color_grade.wgsl`, `mapmap-core/src/lut.rs`)
  - ✅ 3D-LUT-Unterstützung
  - ✅ LUT-Format-Parser (.cube)
  - ✅ LUT-Manager mit Presets
  - ✅ GPU-beschleunigtes Color-Grading

- ✅ **Effekt-Chain-Hooks**
  - ✅ Pluggable Effect System integriert
  - ✅ Post-FX-Pipeline verdrahtet
  - ✅ Effect-Parameter-Binding an UI vorhanden
  - ✅ Real-time Effect Hot-Reload implementiert

### Audio (Plattformspezifische Backends, Analyzer/Mapping)

- ✅ **Audio-Analyse** (`mapmap-core/src/audio.rs`)
  - ✅ FFT-Analyse mit RustFFT
  - ✅ 7 Frequenzbänder (SubBass, Bass, LowMid, Mid, HighMid, Presence, Brilliance)
  - ✅ RMS-Volume-Analyse
  - ✅ Peak-Detektion
  - ✅ Beat-Detection-Grundlagen
  - ✅ AudioAnalyzer mit konfigurierbarem FFT-Window

- ✅ **Audio-Reactive-System** (`mapmap-core/src/audio_reactive.rs`)
  - ✅ AudioReactiveController für Parameter-Mapping
  - ✅ AudioReactiveAnimationSystem
  - ✅ AudioMappingType (Volume, FrequencyBand, Beat, Onset, Spectral)
  - ✅ Audio-zu-Parameter-Mappings mit Smooth/Attack/Decay

- ⬜ **Audio-Backend-Integration** (VERPFLICHTEND)
  - ⬜ CPAL-Backend muss verdrahtet werden (Feature: `audio` in `mapmap-core/Cargo.toml` vorhanden)
  - ⬜ Windows: WASAPI-Backend testen und integrieren
  - ⬜ Linux: ALSA/PulseAudio/JACK-Backend testen und integrieren
  - ⬜ macOS: CoreAudio-Backend (optional, falls Mehraufwand vertretbar)
  - ⬜ Audio-Input-Device-Auswahl in UI fehlt
  - ⬜ Audio-Stream in Media-Pipeline verdrahten fehlt
  - ⬜ Latenz-Kompensation implementieren fehlt

- ✅ **Audio-Build-Enforcement**
  - ✅ Default-Feature `audio` in Workspace aktivieren (aktuell optional)
  - ✅ CI/CD: Audio-Feature in Tests aktivieren
  - ✅ Dokumentation: Audio als Pflicht-Dependency markieren

### Media (FFmpeg-Decode / Playback-Control / GPU-Upload)

- ✅ **FFmpeg-Decoder** (`mapmap-media/src/decoder.rs`)
  - ✅ FFmpeg-Integration über `ffmpeg-next` (optional feature)
  - ✅ Video-Decode mit Hardware-Acceleration-Support
  - ✅ Multi-threaded Decode-Pipeline
  - ✅ Frame-Queue-Management

- ✅ **Image-Decoder** (`mapmap-media/src/image_decoder.rs`)
  - ✅ PNG, JPG, BMP, TGA Support
  - ✅ Image-Crate-basierte Dekodierung
  - ⬜ GIF-Animation noch nicht vollständig implementiert
  - ⬜ Image-Sequence-Playback fehlt (walkdir-Dependency vorhanden)

- ✅ **Player** (`mapmap-media/src/player.rs`)
  - ✅ Robust State-Machine (Idle, Loading, Playing, Paused, Stopped, Error)
  - ✅ PlaybackCommand System
  - ✅ PlaybackStatus Channel
  - ✅ Simplified Loop-Modi (Loop, PlayOnce) - Legacy modes removed
  - ✅ Frame-Seeking & Timestamp-Management

- ✅ **Pipeline** (`mapmap-media/src/pipeline.rs`)
  - ✅ Media-Pipeline-Abstraktion
  - ✅ Async-Channel-basierte Frame-Delivery
  - ✅ Thread-Pool-Integration

- ⬜ **GPU-Upload-Optimierung**
  - ✅ Texture-Upload-Benchmark vorhanden (`mapmap-render/benches/texture_upload.rs`)
  - ⬜ Zero-Copy-Upload fehlt (aktuell: CPU→GPU-Copy)
  - ⬜ PBO (Pixel Buffer Objects) für asynchronen Upload fehlt
  - ⬜ Hardware-Decode-zu-GPU-Direct-Upload fehlt

- ⬜ **Codec-Support**
  - ✅ H.264, H.265, VP8, VP9 über FFmpeg
  - ⬜ ProRes noch nicht getestet/optimiert
  - ⬜ HAP-Codec fehlt (GPU-native Compression)
  - ⬜ DXV-Codec fehlt

### Effects / PostFX

- ✅ **LUT-Color-Grading** (siehe oben)
- ✅ **Blend-Modi** (siehe oben)
- ✅ **Oscillator-Effekte** (siehe oben)
- ✅ **Animation-System** (`mapmap-core/src/animation.rs`)
  - ✅ Keyframe-Animation
  - ✅ AnimationClip und AnimationPlayer
  - ✅ Interpolation-Modi (Linear, Cubic, Step)
  - ✅ TimePoint-basiertes Timing

- ✅ **Shader-Graph-System** (`mapmap-core/src/shader_graph.rs`)
  - ✅ Node-basiertes Shader-System
  - ✅ ParameterValue-System (Float, Vec2, Vec3, Vec4, Color, etc.)
  - ✅ Node-Connections und Graph-Traversal
  - ✅ WGSL-Codegen (`mapmap-core/src/codegen.rs`)

- ⬜ **Effect-Chain-Integration**
  - ⬜ Shader-Graph in Render-Pipeline integrieren fehlt
  - ⬜ Custom-Shader-Hot-Reload fehlt
  - ⬜ Effect-Preset-System fehlt
  - ⬜ Effect-Parameter-Automation via Timeline fehlt

### Control (OSC als Hauptpfad / MIDI low priority)

- ✅ **OSC-System** (`mapmap-control/src/osc/`)
  - ✅ OSC-Server (`osc/server.rs`) mit UDP-Socket
  - ✅ OSC-Client (`osc/client.rs`) für Outgoing-Messages
  - ✅ OSC-Address-Parser (`osc/address.rs`)
  - ✅ OSC-zu-Control-Value-Mapping (`osc/types.rs`)
  - ✅ Feature-Flag: `osc` (optional, muss aktiviert werden)
  
- ✅ **OSC-Integration (HAUPTPFAD – IMPLEMENTIERT)**
  - ✅ OSC-Command-Schema definiert und dokumentiert
  - ✅ OSC-Events an `ControlTarget`s geroutet
  - ✅ OSC-Feedback (State-Updates zurück an Controller) implementiert
  - ✅ Simplified OSC-Mapping (HashMap) - Legacy Learn Mode removed
  - ✅ UI: OSC-Server-Status und Port-Konfiguration implementiert (mit `imgui`)
  - ✅ Default-OSC-Port: 8000 (konfigurierbar)

- ✅ **MIDI-System (LOW PRIORITY)** (`mapmap-control/src/midi/`)
  - ✅ MIDI-Input (`midi/input.rs`)
  - ✅ MIDI-Output (`midi/output.rs`)
  - ✅ MIDI-Mapping (`midi/mapping.rs`) - Simplified HashMap implementation
  - ❌ MIDI-Learn removed (Legacy cleanup)
  - ✅ MIDI-Clock (`midi/clock.rs`)
  - ✅ MIDI-Profiles (`midi/profiles.rs`)
  - ✅ Feature-Flag: `midi` (optional)
  - ⬜ MIDI-zu-Parameter-Routing verdrahten fehlt (low priority)

- ✅ **WebSocket-System** (`mapmap-control/src/web/`) – NICHT NUTZEN
  - ✅ WebSocket-Server vorhanden (`web/websocket.rs`)
  - ✅ Web-API-Routes (`web/routes.rs`, `web/handlers.rs`)
  - ⬜ **Entscheidung: WebSocket NICHT als Control-Pfad nutzen, OSC priorisieren**

- ✅ **DMX-System** (`mapmap-control/src/dmx/`) – FUTURE
  - ✅ Art-Net (`dmx/artnet.rs`)
  - ✅ sACN (`dmx/sacn.rs`)
  - ✅ DMX-Channel-Mapping (`dmx/channels.rs`)
  - ✅ DMX-Fixtures (`dmx/fixtures.rs`)
  - ⬜ Nicht sofort erforderlich, für Phase 4+

- ✅ **Cue-System** (`mapmap-control/src/cue/`)
  - ✅ Cue-Struktur (`cue/cue.rs`)
  - ✅ CueList (`cue/cue_list.rs`)
  - ✅ Crossfade (`cue/crossfade.rs`)
  - ✅ Triggers (`cue/triggers.rs`)
  - ⬜ Cue-System in UI integrieren fehlt

- ✅ **Shortcuts** (`mapmap-control/src/shortcuts/`)
  - ✅ Keyboard-Shortcuts (`shortcuts/shortcuts.rs`)
  - ✅ Bindings (`shortcuts/bindings.rs`)
  - ✅ Macros (`shortcuts/macros.rs`)
  - ⬜ Shortcut-UI fehlt

### UI (ImGui / egui)

- ✅ **UI-Framework-Status**
  - ✅ ImGui-Integration (`mapmap-ui` via `imgui`, `imgui-wgpu`, `imgui-winit-support`)
  - ✅ egui-Integration vorbereitet (`egui`, `egui-wgpu`, `egui-winit`, `egui_dock`, `egui_extras`)
  - 🟡 **Phase 6: Migration von ImGui zu egui im Gange (Hybrid-Betrieb)**

- ✅ **UI-Module (Migriert zu egui)** (`mapmap-ui/src/`)
  - ✅ Dashboard (`dashboard.rs`) – Hauptansicht
  - ✅ Media-Browser (`media_browser.rs`) – Datei-Auswahl
  - ✅ Mesh-Editor (`mesh_editor.rs`) – Mesh-Warping-UI
  - ✅ Node-Editor (`node_editor.rs`) – Shader-Graph-Editor
  - ✅ Timeline V2 (`timeline_v2.rs`) – Keyframe Animation
  - ✅ Undo-Redo (`undo_redo.rs`) – Command-Pattern
  - ✅ Asset-Manager (`asset_manager.rs`)
  - ✅ Theme (`theme.rs`)

- ⬜ **Pending Migration (Legacy ImGui Panels)**
  - ⬜ Layer Manager (`render_layer_panel`)
  - ⬜ Paint Manager (`render_paint_panel`)
  - ⬜ Mapping Manager (`render_mapping_panel`)
  - ⬜ Transform Controls (`render_transform_panel`)
  - ⬜ Output Configuration (`render_output_panel`)
  - ⬜ Edge Blend & Color Calibration (`render_edge_blend_panel`)
  - ⬜ Audio Visualization (`render_audio_panel`)
  - ⬜ Oscillator Control (`render_oscillator_panel`)
  - ⬜ Main Menu & Toolbar (`render_menu_bar`)
  - ⬜ Shader Graph Editor (`shader_graph_editor.rs` - Legacy Version)

- ⬜ **Internationalisierung (i18n) – NEU**
  - ⬜ Sprachauswahl UI (Deutsch / Englisch)
  - ⬜ `fluent` oder `rust-i18n` Crate integrieren
  - ⬜ Übersetzungsdateien (`locales/de.ftl`, `locales/en.ftl`)
  - ⬜ Dynamischer Sprachwechsel zur Laufzeit
  - ⬜ Persistierung der Spracheinstellung in User-Config
  - ⬜ Alle UI-Strings extrahieren und übersetzen

### MCP-Server Integration (Model Context Protocol) – NEU

- ⬜ **MCP-Server Implementierung**
  - ⬜ MCP-Server-Crate erstellen (`mapmap-mcp/`)
  - ⬜ JSON-RPC 2.0 Transport (stdio/SSE)
  - ⬜ Tool-Definitionen für VJMapper-Funktionen:
    - ⬜ `layer_create`, `layer_delete`, `layer_set_opacity`
    - ⬜ `media_load`, `media_play`, `media_pause`, `media_stop`
    - ⬜ `mapping_create`, `mapping_update`
    - ⬜ `output_configure`, `output_enable`
    - ⬜ `cue_trigger`, `cue_next`, `cue_previous`
    - ⬜ `project_save`, `project_load`
  - ⬜ Resource-Definitionen:
    - ⬜ `project://current` – Aktuelles Projekt
    - ⬜ `layer://list` – Layer-Liste
    - ⬜ `media://library` – Media-Bibliothek
    - ⬜ `output://status` – Output-Status
  - ⬜ Prompt-Definitionen für AI-Assistenz:
    - ⬜ `create_mapping` – Mapping-Erstellung assistieren
    - ⬜ `setup_projection` – Projektor-Setup assistieren
    - ⬜ `troubleshoot` – Problemdiagnose
  - ⬜ Integration mit Gemini CLI / Claude Desktop
  - ⬜ Dokumentation: MCP-API-Referenz

### Persistenz / IO (Projektformat, Save/Load)

- ✅ **IO-Subsystem** (`mapmap-io/src/`)
  - ✅ Source (`source.rs`) – Input-Source-Abstraktion
  - ✅ Sink (`sink.rs`) – Output-Sink-Abstraktion
  - ✅ Converter (`converter.rs`) – Format-Konvertierung
  - ✅ Format (`format.rs`) – Format-Definitionen
  - ✅ NDI (`ndi/mod.rs`) – Placeholder (Phase 5)
  - ✅ DeckLink (`decklink/mod.rs`) – Placeholder (Phase 5)
  - ✅ Spout (`spout/mod.rs`) – Placeholder (Phase 5)
  - ✅ Syphon (`syphon/mod.rs`) – Placeholder (Phase 5)
  - ✅ Streaming (`stream/`) – RTMP, SRT, Encoder

- ⬜ **Projektformat**
  - ⬜ JSON/RON-basiertes Projektformat definieren
  - ⬜ Serialisierung aller Projekt-Entitäten (Layers, Mappings, Meshes, Outputs, Cues, etc.)
  - ⬜ Deserialisierung mit Validierung
  - ⬜ Versioning und Migration
  - ⬜ Auto-Save-Mechanismus
  - ⬜ Recent-Files-Liste

- ⬜ **Asset-Management**
  - ⬜ Asset-Pfad-Verwaltung (relativ/absolut)
  - ⬜ Asset-Caching
  - ⬜ Thumbnail-Generierung für Media

### Tests

- ✅ **Bestehende Tests**
  - ✅ Unit-Tests in Core (`mapmap-core/src/*.rs` mit `#[cfg(test)]`)
  - ✅ PropTest für Property-Based-Testing (`mapmap-core/Cargo.toml`)
  - ✅ Benchmarks: `texture_upload.rs`, `video_decode.rs`
  - ✅ Examples: `hello_world_projection.rs`, `simple_render.rs`

- ⬜ **Fehlende Tests**
  - ⬜ Audio-System-Tests mit Audio-Feature aktiviert
  - ⬜ OSC-Integration-Tests
  - ⬜ Multi-Output-Rendering-Tests
  - ⬜ Effect-Chain-Tests
  - ⬜ Project-Save/Load-Tests
  - ⬜ End-to-End-Tests

### Packaging / Developer Experience (DX)

- ✅ **CI/CD** (`.github/workflows/`)
  - ✅ CI-Workflow vorhanden
  - ⬜ Audio-Feature in CI aktivieren fehlt
  - ⬜ FFmpeg in CI-Builds aktivieren fehlt
  - ⬜ Windows-CI-Builds fehlen
  - ⬜ macOS-CI-Builds fehlen (optional)

- 🟡 **Packaging**
  - 🟡 Windows-Installer (WiX) – Konfiguration (`crates/mapmap/wix/main.wxs`) vorhanden
  - ✅ App Icon Embedding (`winres` in `build.rs` konfiguriert)
  - ⬜ Linux Packaging (.deb)
  - ⬜ Linux-AppImage/Flatpak/Snap
  - ⬜ Dependency-Bundling (FFmpeg-Libs)

- ✅ **Developer-Tools**
  - ✅ `scripts/check-ffmpeg-env.sh` – FFmpeg-Check
  - ✅ `scripts/install-ffmpeg-dev.sh` – FFmpeg-Install-Script
  - ✅ `rust-toolchain.toml` – Rust-Version 1.75

---

## Architektur und Crate-Übersicht

### Workspace-Struktur

```
crates/
├── mapmap-core/       # Domain-Modell (Layer, Mapping, Mesh, Audio, Shader-Graph, Animation)
├── mapmap-render/     # Rendering-Engine (wgpu-Backend, Compositor, Renderer)
├── mapmap-media/      # Media-Decoding (FFmpeg, Image, Player, Pipeline)
├── mapmap-ui/         # UI-Layer (ImGui/egui, Editor-Panels)
├── mapmap-control/    # Control-Systeme (OSC, MIDI, DMX, Web-API, Cues, Shortcuts)
├── mapmap-io/         # Professional I/O (NDI, DeckLink, Spout, Syphon, Streaming)
├── mapmap-ffi/        # FFI-Bindings (Placeholder für Phase 5)
└── mapmap/            # Hauptanwendung (Binary, Window-Management, Integration)
```

### Abhängigkeiten

- **Graphics:** wgpu (Vulkan/Metal/DX12), winit (Windowing)
- **Media:** ffmpeg-next (optional feature), image
- **Audio:** cpal (optional feature), rustfft, hound
- **UI:** imgui (Phase 0-5), egui (Phase 6+), egui_dock, egui_extras
- **Control:** rosc (OSC, optional), midir (MIDI, optional), axum (HTTP-API, optional)
- **Serialisierung:** serde, serde_json, toml, ron
- **Async:** tokio, futures, crossbeam-channel
- **Logging:** tracing, tracing-subscriber
- **Testing:** proptest, criterion

---

## Arbeitspakete für @jules

### 🟢 **Priorität 1: Audio-Build-Enforcement (COMPLETED)**

**Zweck:** Audio ist Kern-Feature des Systems und muss immer verfügbar sein.

**Status:** ✅ Completed (2025-12-16)

**Realisiert:**
- ✅ Feature-Aktivierung in `Cargo.toml`
- ✅ Backend-Verdrahtung (CPAL)
- ✅ Audio-Stream-Integration in `main.rs`
- ✅ UI-Integration (Dashboard, Device Selector, FFT)
- ✅ CI/CD-Testing mit Audio-Support

**Schritte:**

1. **Feature-Aktivierung:**
   - `crates/mapmap-core/Cargo.toml`: `default = ["audio"]` setzen (aktuell: `default = []`)
   - `crates/mapmap/Cargo.toml`: `default = ["audio"]` setzen

2. **Backend-Verdrahtung:**
   - `mapmap-core/src/audio.rs`: `AudioSource` mit CPAL-Stream verbinden
   - Neues Modul `mapmap-core/src/audio/backend.rs` erstellen:
     ```rust
     // Audio-Backend-Abstraktion
     pub trait AudioBackend {
         fn start(&mut self) -> Result<(), AudioError>;
         fn stop(&mut self);
         fn get_samples(&mut self) -> Vec<f32>;
     }
     
     // CPAL-Implementation
     #[cfg(feature = "audio")]
     pub struct CpalBackend { /* ... */ }
     
     impl AudioBackend for CpalBackend {
         // Windows: WASAPI
         // Linux: ALSA/PulseAudio/JACK
         // macOS: CoreAudio
     }
     ```

3. **Audio-Stream-Integration:**
   - In `mapmap/src/main.rs`: `AudioBackend` initialisieren und mit `AudioAnalyzer` verbinden
   - Audio-Samples aus CPAL-Stream in FFT-Pipeline einspeisen
   - Latenz-Kompensation: Buffer-Size-Konfiguration (empfohlen: 512-2048 Samples)

4. **UI-Integration:**
   - `mapmap-ui/src/dashboard.rs`: Audio-Input-Device-Selector hinzufügen
   - Audio-Level-Meter und FFT-Visualisierung
   - Device-Enumeration via `cpal::available_hosts()` und `cpal::default_host().input_devices()`

5. **Plattform-Tests:**
   - **Windows:** WASAPI-Backend testen (empfohlene Device: "Stereo Mix" oder "What U Hear")
   - **Linux:** ALSA/PulseAudio testen (empfohlene Config: PulseAudio-Monitor-Device)
   - **macOS (optional):** CoreAudio testen

6. **CI/CD-Anpassung:**
   - `.github/workflows/ci.yml`: `--features audio` zu `cargo build` und `cargo test` hinzufügen
   - Audio-Tests ohne Hardware: Mock-Backend für CI

7. **Dokumentation:**
   - `README.md`: Audio als Pflicht-Dependency markieren
   - `docs/01-GETTING-STARTED/`: Audio-Setup-Anleitung für Windows/Linux
   - Env-Check-Script: `check-audio-backend.sh` erstellen (analog zu `scripts/check-ffmpeg-env.sh`)

**Akzeptanzkriterien:**
- Build schlägt fehl, wenn Audio-Backend nicht verfügbar
- Audio-Input funktioniert auf Windows und Linux
- UI zeigt Audio-Level in Echtzeit
- FFT-Analyse läuft mit <10ms Latenz

---

### 🟢 **Priorität 2: OSC-Command-Schema und Integration (COMPLETED)**

**Zweck:** OSC als primärer External-Control-Pfad (statt WebSocket). MIDI ist low priority.

**Status:** ✅ Completed (2025-12-15)

**Realisiert:**
- **Command-Schema:** OSC-Address-Space definiert und dokumentiert
- **OSC-Events:** Routing zu `ControlTarget`s implementiert
- **OSC-Feedback:** State-Updates zurück an Controller implementiert
- **Simplified OSC-Mapping:** HashMap-basiertes Mapping (Legacy Learn Mode entfernt)
- **UI:** OSC-Server-Status und Port-Konfiguration mit `imgui` implementiert
- **Default-Port:** 8000 (konfigurierbar)

**Schritte (Archiv):**

1. **Command-Schema definieren:**
   - Dokumentation in `mapmap-control/src/osc/mod.rs` erweitern (aktuell: nur Beispiele)
   - Full Address Space definieren:
     ```
     # Layer Control
     /mapmap/layer/{id}/opacity        [f32: 0.0-1.0]
     /mapmap/layer/{id}/position       [f32, f32: x, y]
     /mapmap/layer/{id}/rotation       [f32: degrees]
     /mapmap/layer/{id}/scale          [f32, f32: x, y]
     /mapmap/layer/{id}/visible        [bool]
     /mapmap/layer/{id}/blend_mode     [string: "add"|"multiply"|...]
     
     # Paint Control
     /mapmap/paint/{id}/opacity        [f32: 0.0-1.0]
     /mapmap/paint/{id}/brightness     [f32: 0.0-1.0]
     
     # Effect Control
     /mapmap/effect/{id}/param/{name}  [varies]
     
     # Playback Control
     /mapmap/playback/play             []
     /mapmap/playback/pause            []
     /mapmap/playback/stop             []
     /mapmap/playback/speed            [f32: -4.0 to 4.0]
     /mapmap/playback/position         [f32: 0.0-1.0]
     
     # Output Control
     /mapmap/output/{id}/brightness    [f32: 0.0-1.0]
     /mapmap/output/{id}/edge_blend    [f32: 0.0-1.0]
     
     # Cue Control
     /mapmap/cue/trigger/{id}          []
     /mapmap/cue/next                  []
     /mapmap/cue/previous              []
     ```

2. **OSC-Event-Routing:**
   - `mapmap-control/src/manager.rs`: `ControlManager` erweitern
   - OSC-Events zu `ControlTarget` routen (bereits implementiert in `target.rs`)
   - Event-Queue für Thread-sichere Communication mit Main-Thread

3. **State-Updates zurück an Controller (OSC-Feedback):**
   - `OscClient::send_update()` nutzen (bereits implementiert)
   - State-Changes im `LayerManager` abfangen und als OSC-Messages zurückschicken
   - Konfigurierbares Feedback-Routing (um Feedback-Loops zu vermeiden)

4. **OSC-Learn-Mode:**
   - UI: "OSC Learn"-Button in Control-Panel
   - Learn-Mode aktivieren → nächste eingehende OSC-Message auf ausgewählten Parameter mappen
   - Mapping speichern in `ControlManager`

5. **UI-Integration:**
   - `mapmap-ui/src/dashboard.rs`: OSC-Server-Status-Panel hinzufügen
   - OSC-Port-Konfiguration (Default: 8000)
   - OSC-Message-Log (Debugging)
   - OSC-Mapping-Liste

6. **Feature-Aktivierung:**
   - `crates/mapmap/Cargo.toml`: `default = ["osc"]` setzen (oder `full` feature verwenden)
   - OSC-Feature standardmäßig aktiviert, MIDI optional

7. **Tests:**
   - Integration-Test: OSC-Server starten, Messages schicken, State-Changes verifizieren
   - OSC-Client-Test: Feedback-Messages empfangen

8. **Dokumentation:**
   - `docs/`: OSC-Command-Reference erstellen
   - TouchOSC-Template als Beispiel
   - QLab-Integration-Beispiel

**Akzeptanzkriterien:**
- OSC-Server läuft standardmäßig auf Port 8000
- Layer-Opacity via OSC steuerbar
- OSC-Feedback funktioniert (bidirektional)
- OSC-Learn-Mode funktioniert
- UI zeigt OSC-Status und Message-Log

---

### 🟢 **Priorität 3: Media-Playback-State-Machine (COMPLETED)**

**Zweck:** Robuste Playback-Control mit Zustandsverwaltung.

**Status:** ✅ Completed (2025-12-14)

**Realisiert:**
- **State-Machine:** Komplett neu implementiert (`mapmap-media/src/player.rs`) mit `PlaybackState` (Idle, Loading, Playing, Paused, Stopped, Error)
- **Commands:** `PlaybackCommand` System (Play, Pause, Stop, Seek, SetSpeed, SetLoopMode)
- **Status:** `PlaybackStatus` Channel für asynchrone Rückmeldungen
- **UI:** Integration in `Dashboard` und `AppUI` aktualisiert
- **Cleanup:** Legacy Modes (PingPong, PlayOnceAndEject) entfernt
- **Tests:** Umfassende Unit-Tests für Transitions und Commands

---

### 🟢 **Priorität 4: Effect-Chain-Hooks und Integration (COMPLETED)**

**Zweck:** Shader-Graph in Render-Pipeline integrieren, Effect-Chain nutzbar machen.

**Status:** 🟢 Vollständig abgeschlossen (2025-12-16)

**Realisiert:**
- ✅ **Effect-Chain-Renderer:** `mapmap-render/src/effect_chain_renderer.rs` erstellt
- ✅ **Multi-Pass-Rendering:** Ping-Pong-Buffers für mehrstufige Effects implementiert
- ✅ **Effect-Parameter-System:** `EffectParams` Uniform-Buffer an GPU
- ✅ **10 Effekt-Typen:** ColorAdjust, Blur, ChromaticAberration, EdgeDetect, Glow, Kaleidoscope, Invert, Pixelate, Vignette, FilmGrain
- ✅ **WGSL Shader:** 9 Shader-Dateien in `crates/mapmap-render/shaders/`
- ✅ **Unit-Tests:** 5 Tests für EffectChain Logik
- ✅ **Hot-Reload-System:** `mapmap-render/src/hot_reload.rs` mit File-Watcher via `notify` crate
  - Debouncing für schnelle Änderungen
  - Fallback zu letztem funktionierenden Shader bei Compile-Fehlern
  - `ShaderHotReload` und `HotReloadIntegration` Klassen
- ✅ **Preset-System:** `mapmap-render/src/preset.rs` vollständig implementiert
  - JSON-basierte Speicherung
  - `PresetLibrary` für Verwaltung
  - Kategorien und Favoriten-System
  - Factory-Presets: Cinema Look, Retro VHS, Dreamy, Neon Edges
- ✅ **UI-Integration:** `mapmap-ui/src/effect_chain_panel.rs` egui-basiert
  - Effect-Chain-Liste mit Reorder-Buttons
  - Parameter-Sliders für jeden Effekt
  - Preset-Browser mit Suche und Favoriten
  - Drag-vorbereitung für zukünftiges Drag&Drop

**Schritte (Archiv):**

1. **Shader-Graph-zu-WGSL-Pipeline:**
   - `mapmap-core/src/codegen.rs`: WGSL-Codegen testen und debuggen
   - Test: Shader-Graph → WGSL-String → wgpu::ShaderModule

2. **Effect-Chain-Renderer:**
   - `mapmap-render/src/effect_chain_renderer.rs` erstellen ✅
   - Multi-Pass-Rendering: Input-Texture → Effect 1 → Effect 2 → ... → Output-Texture ✅
   - Ping-Pong-Buffers für mehrstufige Effects ✅

3. **Effect-Parameter-Binding:**
   - Shader-Graph-Parameter als Uniform-Buffer an GPU schicken ✅
   - Parameter-Updates via `wgpu::Queue::write_buffer()` ✅

4. **Hot-Reload:**
   - File-Watcher für `.wgsl`-Files (via `notify` crate) ✅
   - Shader-Reload ohne Neustart der Anwendung ✅
   - Error-Handling bei Shader-Compile-Fehlern (Fallback zu Previous-Shader) ✅

5. **UI-Integration:**
   - `mapmap-ui/src/effect_chain_panel.rs` erstellt ✅
   - Effect-Chain-Liste (Reorder-Buttons) ✅
   - Parameter-Sliders für jeden Effect ✅

6. **Preset-System:**
   - `mapmap-render/src/preset.rs` erstellt ✅
   - Effect-Presets als JSON speichern ✅
   - Preset-Browser in UI ✅

**Akzeptanzkriterien:**
- ✅ Shader-Graph wird zu WGSL kompiliert
- ✅ Effect-Chain läuft in Render-Pipeline
- ✅ Parameter-Änderungen in UI wirken sich in Echtzeit aus
- ✅ Shader-Hot-Reload funktioniert
- ✅ Presets können gespeichert und geladen werden

---

### 🟡 **Priorität 5: Projektformat und Persistenz (IN PROGRESS)**

**Zweck:** Save/Load von Projekten, um Setups zu speichern und wiederherzustellen.

**Schritte:**

1. **Format-Definition:**
   - ✅ RON (Rusty Object Notation) oder JSON als Format wählen (RON und JSON implementiert)
   - ✅ Projekt-Struktur (`AppState` in `mapmap-core/src/state.rs`)
     ```rust
     #[derive(Serialize, Deserialize)]
     pub struct AppState {
         pub name: String,
         pub paint_manager: PaintManager,
         // ...
     }
     ```

2. **Serialisierung:**
   - ✅ Alle Core-Structs mit `#[derive(Serialize, Deserialize)]` annotieren (in `mapmap-core` erledigt)
   - ⬜ Custom-Serializer für komplexe Typen (z. B. wgpu-Textures: nur Pfad speichern, nicht Binärdaten)

3. **Deserialisierung mit Validierung:**
   - Schema-Validierung (Version-Check)
   - Asset-Pfad-Validierung (existieren die Dateien?)
   - Migration von älteren Versionen (z. B. v0.1.0 → v0.2.0)

4. **Auto-Save:**
   - Periodisches Auto-Save (alle 5 Minuten)
   - Auto-Save-File: `.mapmap_autosave`

5. **Recent-Files:**
   - Recent-Files-Liste in User-Config speichern
   - UI: Recent-Files-Menu in Dashboard

6. **UI-Integration:**
   - ✅ File-Menu: New, Open, Save, Save As, Recent Files (Save/Load implementiert)
   - ✅ Native-File-Dialog via `rfd` (implementiert)

7. **Tests:**
   - Save/Load-Roundtrip-Test: Projekt speichern → laden → verifizieren
   - Migration-Test: Altes Format → Neues Format

**Akzeptanzkriterien:**
- Projekte können gespeichert und geladen werden
- Alle Projekt-Entitäten werden korrekt persistiert
- Auto-Save funktioniert
- Recent-Files-Liste funktioniert

---

### 🟢 **Priorität 6: Multi-Window-Rendering (Phase 2 Completion)**

**Zweck:** Multi-Projektor-Setup mit synchronisierter Frame-Präsentation.

**Schritte:**

1. **Window-per-Output-Architektur:**
   - `mapmap/src/window_manager.rs` erweitern
   - Ein `winit::Window` pro Output-Device erstellen
   - Monitor-Detection via `winit::monitor::MonitorHandle`

2. **Per-Output-Render-Target:**
   - Jedes Window hat eigene `wgpu::Surface` und `wgpu::SurfaceTexture`
   - Output-Konfiguration (Resolution, Position) aus `mapmap-core/src/output.rs`

3. **Frame-Synchronisation:**
   - VSync-basiertes Timing (Standard)
   - Optional: Manual-Sync via `wgpu::Queue::submit()` mit Barriers
   - Frame-Drop-Detection: Warnung bei >16ms Frame-Time (60fps Target)

4. **Canvas-Region-Filtering:**
   - Jedes Output-Window rendert nur den zugeordneten Canvas-Bereich
   - Viewport-Transform: Canvas-Space → Output-Space

5. **Output-Management-UI:**
   - `mapmap-ui/src/dashboard.rs`: Output-Liste mit Preview
   - Output-Konfiguration: Position, Size, Edge-Blend, Color-Calib
   - 2x2-Projektor-Array-Preset (bereits in Code erwähnt)

6. **Tests:**
   - Multi-Monitor-Test (2 virtuelle Displays)
   - Frame-Sync-Test (Frame-Time-Messung)

**Akzeptanzkriterien:**
- Multi-Window-Rendering funktioniert auf 2+ Displays
- Frame-Sync hält VSync-Target (60fps)
- Output-Konfiguration ist in UI editierbar

---

### 🟢 **Priorität 7: CI/CD mit Audio und FFmpeg**

**Zweck:** Builds automatisieren und auf allen Plattformen testen.

**Schritte:**

1. **GitHub-Actions-Anpassung:**
   - `.github/workflows/ci.yml` erweitern:
     ```yaml
     - name: Install dependencies (Linux)
       run: |
         sudo apt-get update
         sudo apt-get install -y \
           libfontconfig1-dev libfreetype6-dev \
           libxcb1-dev libx11-dev libasound2-dev \
           libavcodec-dev libavformat-dev libavutil-dev
     
     - name: Build with audio and ffmpeg
       run: cargo build --workspace --features audio,ffmpeg
     
     - name: Test with audio and ffmpeg
       run: cargo test --workspace --features audio,ffmpeg
     ```

2. **Windows-CI:**
   - Separate Job für Windows-Build
   - FFmpeg-Binaries via vcpkg oder pre-built package installieren
   - WASAPI-Backend (keine Hardware erforderlich, Mock-Device nutzen)

3. **macOS-CI (optional):**
   - Nur wenn Mehraufwand vertretbar
   - FFmpeg via Homebrew installieren

4. **Env-Check-Scripts:**
   - `scripts/check-ffmpeg-env.sh`: Erweitern um Audio-Backend-Check
   - `check-audio-backend.sh`: Neu erstellen

5. **CI-Badge-Update:**
   - `README.md`: CI-Badge aktualisieren

**Akzeptanzkriterien:**
- Linux-CI-Build mit Audio und FFmpeg läuft grün
- Windows-CI-Build mit Audio und FFmpeg läuft grün
- macOS-CI-Build optional, aber dokumentiert

---

### 🟢 **Priorität 8: Dokumentation und Developer Experience**

**Zweck:** Entwickler-Onboarding verbessern, Code-Dokumentation vervollständigen.

**Schritte:**

1. **README-Update:**
   - Audio als Pflicht-Feature markieren
   - FFmpeg-Installation-Anleitung erweitern
   - Quick-Start-Guide aktualisieren

2. **API-Dokumentation:**
   - Rustdoc-Kommentare für alle Public-APIs vervollständigen
   - Examples in Rustdoc hinzufügen

3. **Architektur-Dokumentation:**
   - `docs/03-ARCHITECTURE/`: Crate-Dependencies visualisieren
   - Datenfluss-Diagramme (Media-Pipeline, Render-Pipeline, Control-Pipeline)

4. **User-Guide:**
   - `docs/04-USER-GUIDE/`: OSC-Command-Reference
   - Audio-Setup-Anleitung
   - Multi-Projektor-Setup-Anleitung

5. **Video-Tutorials:**
   - Screencast: Audio-reaktive Effekte einrichten
   - Screencast: OSC-Control mit TouchOSC
   - Screencast: Multi-Projektor-Setup

**Akzeptanzkriterien:**
- Alle Public-APIs haben Rustdoc-Kommentare
- `docs/` ist vollständig und aktuell
- `README.md` ist klar und hilfreich

---

### 🟡 **Priorität 9: Internationalisierung (i18n) – NEU**

**Zweck:** UI-Sprache zwischen Deutsch und Englisch umschaltbar machen.

**Schritte:**

1. **i18n-Framework auswählen:**
   - Option A: `fluent-rs` (Mozilla Fluent) – Empfohlen für Flexibilität
   - Option B: `rust-i18n` – Einfacher zu integrieren
   - Entscheidung: `fluent-rs` wegen besserer Pluralisierung und Rich-Text

2. **Übersetzungsdateien erstellen:**
   ```
   locales/
   ├── en/
   │   └── main.ftl          # Englische Übersetzungen
   └── de/
       └── main.ftl          # Deutsche Übersetzungen
   ```

3. **i18n-Modul implementieren:**
   - `mapmap-ui/src/i18n.rs`: Sprachmanager
   - `LocaleManager` mit `set_locale()`, `get_locale()`, `t()` (translate)
   - Fallback auf Englisch wenn Übersetzung fehlt

4. **UI-Integration:**
   - Sprachauswahl-Dropdown in Settings-Panel
   - Dynamischer Sprachwechsel ohne Neustart
   - Alle UI-Strings durch `t!("key")` Makro ersetzen

5. **Persistenz:**
   - Spracheinstellung in User-Config speichern (`~/.vjmapper/config.toml`)
   - Beim Start aus Config laden

6. **Übersetzungsarbeit:**
   - Alle UI-Strings extrahieren (ca. 200-300 Strings)
   - Deutsche Übersetzungen erstellen
   - Review durch Native-Speaker

**Akzeptanzkriterien:**
- Sprachauswahl (DE/EN) in UI verfügbar
- Dynamischer Sprachwechsel funktioniert
- Alle UI-Strings übersetzt
- Spracheinstellung wird persistiert

---

### 🟡 **Priorität 10: MCP-Server Integration – NEU**

**Zweck:** VJMapper als MCP-Server bereitstellen für AI-gestützte Steuerung und Automatisierung.

**Schritte:**

1. **MCP-Crate erstellen:**
   ```bash
   cargo new --lib crates/mapmap-mcp
   ```
   - Abhängigkeiten: `serde`, `serde_json`, `tokio`, `jsonrpc-core`

2. **Transport-Layer:**
   - stdio-Transport für CLI-Integration (Gemini CLI, Claude Desktop)
   - Optional: SSE-Transport für Web-Clients
   - JSON-RPC 2.0 Protokoll implementieren

3. **Tool-Definitionen:**
   ```rust
   // Beispiel Tool-Definition
   pub struct LayerSetOpacity {
       pub layer_id: String,
       pub opacity: f32,  // 0.0-1.0
   }
   ```
   - Layer-Management: `layer_create`, `layer_delete`, `layer_set_opacity`, `layer_set_visibility`
   - Media-Control: `media_load`, `media_play`, `media_pause`, `media_stop`, `media_seek`
   - Mapping: `mapping_create`, `mapping_update`, `mapping_delete`
   - Output: `output_configure`, `output_enable`, `output_disable`
   - Cue: `cue_trigger`, `cue_next`, `cue_previous`
   - Project: `project_save`, `project_load`, `project_new`

4. **Resource-Definitionen:**
   - `project://current` – JSON-Repräsentation des aktuellen Projekts
   - `layer://list` – Liste aller Layer mit Status
   - `media://library` – Verfügbare Media-Assets
   - `output://status` – Status aller Outputs
   - `cue://list` – Cue-Liste

5. **Prompt-Definitionen:**
   - `create_mapping` – Assistiert beim Erstellen eines neuen Mappings
   - `setup_projection` – Hilft beim Multi-Projektor-Setup
   - `troubleshoot` – Diagnose bei Problemen
   - `optimize_performance` – Performance-Optimierungsvorschläge

6. **Integration mit Main-App:**
   - `mapmap/src/main.rs`: MCP-Server als separater Thread starten
   - Command-Queue für Thread-sichere Kommunikation
   - State-Updates an MCP-Clients pushen

7. **Manifest-Datei:**
   ```json
   {
     "name": "vjmapper",
     "version": "0.1.0",
     "description": "VJMapper Projection Mapping Control",
     "tools": [...],
     "resources": [...],
     "prompts": [...]
   }
   ```

8. **Dokumentation:**
   - `docs/MCP-API.md`: Vollständige API-Referenz
   - Beispiele für Gemini CLI und Claude Desktop

**Akzeptanzkriterien:**
- MCP-Server startet mit VJMapper
- Tools funktionieren (Layer, Media, Cue)
- Resources liefern korrekten State
- Integration mit Gemini CLI funktioniert
- Dokumentation vollständig

---

---

## 🎯 Task-Gruppen (Adaptiert für Rust)

> Die folgenden Task-Gruppen sind aus dem ursprünglichen TypeScript/Electron-Konzept adaptiert für das Rust/wgpu-Projekt.

---

### 📦 TG-01: State Management & Persistence

**Ziel:** Zentrales State-Management mit automatischem Speichern.

#### Task 01.1: State Store (Rust)
- 👤 [Jules] Entscheidung: `Arc<RwLock<AppState>>` vs. Message-Passing
- 🤖 [Gemini] **Datei:** `mapmap-core/src/state.rs`
- 🤖 [Gemini] **Logik:** 
  ```rust
  pub struct AppState {
      pub layers: Vec<Layer>,
      pub mappings: Vec<Mapping>,
      pub outputs: Vec<Output>,
      pub audio_config: AudioConfig,
      pub dirty: bool,  // Änderungen vorhanden?
  }
  
  impl AppState {
      pub fn add_layer(&mut self, layer: Layer) { /* ... */ }
      pub fn update_layer(&mut self, id: LayerId, update: LayerUpdate) { /* ... */ }
  }
  ```

#### Task 01.2: Autosave Service
- 🤖 [Gemini] **Datei:** `mapmap-core/src/autosave.rs`
- 🤖 [Gemini] **Logik:** 
  - Bei jeder State-Änderung → `dirty = true`
  - Debounce Timer (1s) → Speichern in JSON-Datei
  - Atomares Schreiben (temp-file → rename)

#### Task 01.3: Persistence Service
- 🤖 [Gemini] **Datei:** `mapmap-io/src/project.rs`
- 🤖 [Gemini] **Format:** RON oder JSON (Serde)
- 🤖 [Gemini] **Logik:**
  ```rust
  pub fn save_project(state: &AppState, path: &Path) -> Result<(), ProjectError>;
  pub fn load_project(path: &Path) -> Result<AppState, ProjectError>;
  ```

**Akzeptanzkriterien:**
- [ ] State-Änderungen werden automatisch gespeichert
- [ ] Atomares Schreiben verhindert Datenverlust
- [ ] Projekt kann geladen und gespeichert werden

---

### 🖼 TG-02: Flexible UI Framework (Docking)

**Ziel:** Das moderne Interface-Gerüst mit Docking-Support.

#### Task 02.1: Layout Engine
- 🤖 [Gemini] **Lib:** `egui_dock` (bereits als Dependency vorhanden)
- 🤖 [Gemini] **Datei:** `mapmap-ui/src/layout.rs`
- 🤖 [Gemini] **Code:** 
  ```rust
  pub struct AppLayout {
      dock_state: DockState<PanelType>,
  }
  
  pub enum PanelType {
      LayerList,
      Viewport,
      Inspector,
      MediaLibrary,
      Timeline,
  }
  ```
- 🤖 [Gemini] **Default Layout:** Links: Layer-Liste, Mitte: Viewport, Rechts: Inspector

#### Task 02.2: Panel Registry
- 🤖 [Gemini] **Module:**
  - `mapmap-ui/src/panels/layer_panel.rs`
  - `mapmap-ui/src/panels/inspector_panel.rs`
  - `mapmap-ui/src/panels/library_panel.rs`
  - `mapmap-ui/src/panels/viewport_panel.rs`
- 🤖 [Gemini] **Feature:** Layout-Präferenzen in User-Config speichern

**Akzeptanzkriterien:**
- [ ] Panels können per Drag&Drop angeordnet werden
- [ ] Layout wird beim Schließen gespeichert
- [ ] Layout wird beim Start wiederhergestellt

---

### 🎭 TG-04: Advanced Features (VPT 8 Level)

**Ziel:** Masken, Grid-Warping, Multi-Layer Compositing.

#### Task 04.1: Multi-Layer Compositing
- 🤖 [Gemini] **Datei:** `mapmap-render/src/compositor.rs` (erweitern)
- 🤖 [Gemini] **Logik:**
  ```rust
  // Layer A → Framebuffer A
  // Layer B → Framebuffer B
  // Blend(A, B, blend_mode) → Output
  
  pub fn composite_layers(
      layers: &[LayerRenderData],
      blend_modes: &[BlendMode],
  ) -> wgpu::TextureView;
  ```

#### Task 04.2: Grid Warping (Mesh)
- 🤖 [Gemini] **Datei:** `mapmap-core/src/math/bilinear.rs` (neu)
- 🤖 [Gemini] **Logik:** Gitter-Interpolation für flexible Mesh-Deformation
  ```rust
  pub struct GridMesh {
      pub columns: usize,
      pub rows: usize,
      pub control_points: Vec<Vec2>,
  }
  
  impl GridMesh {
      pub fn interpolate(&self, uv: Vec2) -> Vec2;
  }
  ```
- 🤖 [Gemini] **UI:** Umschalter "Corner Pin" vs. "Mesh Warp" im Inspector

#### Task 04.3: Masking System
- 🤖 [Gemini] **Datei:** `mapmap-core/src/math/bezier.rs` (erweitern)
- 🤖 [Gemini] **Datei:** `mapmap-ui/src/tools/pen_tool.rs` (neu)
- 🤖 [Gemini] **UI:** Pen-Tool zum Zeichnen von Bezier-Pfaden
- 🤖 [Gemini] **Renderer:** Stencil Buffer zum Ausschneiden
  ```rust
  // In WGSL Shader
  @group(0) @binding(2) var mask_texture: texture_2d<f32>;
  
  // Discard pixels where mask < 0.5
  if (textureSample(mask_texture, sampler, uv).r < 0.5) {
      discard;
  }
  ```

**Akzeptanzkriterien:**
- [ ] Multi-Layer mit verschiedenen Blend-Modi
- [ ] Grid-Mesh mit 4x4+ Kontrollpunkten
- [ ] Bezier-Masken zum Ausschneiden

---

### 🎛 TG-05: Library & Presets

**Ziel:** Drag & Drop von Einstellungen und Media-Assets.

#### Task 05.1: Preset System
- 🤖 [Gemini] **Datei:** `mapmap-core/src/preset.rs`
- 🤖 [Gemini] **Speicherort:** `~/.vjmapper/presets/`
- 🤖 [Gemini] **Format:**
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct LayerPreset {
      pub name: String,
      pub blend_mode: BlendMode,
      pub opacity: f32,
      pub effects: Vec<EffectConfig>,
  }
  ```

#### Task 05.2: Media Library
- 🤖 [Gemini] **Datei:** `mapmap-ui/src/panels/library_panel.rs`
- 🤖 [Gemini] **Features:**
  - Thumbnail-Generierung für Videos/Bilder
  - Drag & Drop auf Layer
  - Ordner-Struktur anzeigen

**Akzeptanzkriterien:**
- [ ] Presets können gespeichert und geladen werden
- [ ] Drag & Drop funktioniert
- [ ] Thumbnails werden angezeigt

---

### 🚀 TG-06: Hybrid Multi-PC Architecture

**Ziel:** Master steuert Slave(s) für verteiltes Projection Mapping.

#### Task 06.1: TCP Networking
- 🤖 [Gemini] **Datei:** `mapmap-control/src/net/sync_server.rs` (Master)
- 🤖 [Gemini] **Datei:** `mapmap-control/src/net/sync_client.rs` (Slave)
- 🤖 [Gemini] **Protokoll:** JSON über TCP
  ```rust
  #[derive(Serialize, Deserialize)]
  pub enum SyncMessage {
      UpdateState { payload: AppState },
      TriggerCue { cue_id: CueId },
      SyncClock { timestamp: u64 },
  }
  ```

#### Task 06.2: App Modes
- 🤖 [Gemini] **Datei:** `mapmap/src/config.rs`
- 🤖 [Gemini] **Logik:** Parse CLI-Argumente
  ```bash
  vjmapper --mode=master --port=9000
  vjmapper --mode=slave --master=192.168.1.100:9000
  ```
- 🤖 [Gemini] **Slave Mode:** Nur Viewport (Fullscreen), keine UI-Panels

#### Task 06.3: Clock Sync
- 🤖 [Gemini] **Datei:** `mapmap-control/src/net/clock_sync.rs`
- 🤖 [Gemini] **Logik:** NTP-ähnlicher Offset-Check
  ```rust
  pub struct ClockSync {
      offset_ms: i64,
      round_trip_time: u64,
  }
  
  impl ClockSync {
      pub fn synchronized_time(&self) -> Duration;
  }
  ```

**Akzeptanzkriterien:**
- [ ] Master kann State an Slaves senden
- [ ] Slaves rendern synchron (< 1 Frame Versatz)
- [ ] Automatische Wiederverbindung bei Disconnect

---

### 📦 TG-07: Distribution & Polishing

**Ziel:** Installer für Endanwender (Windows & Linux).

#### Task 07.1: Universal Installer
- 🤖 [Gemini] **Windows:** WiX Installer (bereits konfiguriert in `crates/mapmap/wix/`)
- 🤖 [Gemini] **Linux:** `.deb` Package (bereits in CI)
- 🤖 [Gemini] **Config:** Single-Binary mit gebündelten Assets

#### Task 07.2: First Run Wizard
- 🤖 [Gemini] **Datei:** `mapmap-ui/src/wizard.rs`
- 🤖 [Gemini] **UI:** Beim ersten Start fragen:
  - "Master oder Slave?"
  - Audio-Input-Device auswählen
  - Sprache wählen (DE/EN)
- 🤖 [Gemini] **Speichern:** `~/.vjmapper/config.toml`

#### Task 07.3: Auto-Update (Optional)
- 👤 [Jules] Entscheidung: Self-Update oder nur Benachrichtigung?
- 🤖 [Gemini] **Logik:** GitHub Releases API abfragen
- 🤖 [Gemini] **UI:** Update-Banner anzeigen

**Akzeptanzkriterien:**
- [ ] Windows MSI-Installer funktioniert
- [ ] Linux .deb-Package funktioniert
- [ ] First Run Wizard konfiguriert App korrekt

---

## Implementierungsdetails nach Crate

### mapmap-core

**Status:** ✅ 90% implementiert, ⬜ 10% Integration fehlt

**Implementierte Module:**
- `layer.rs`: Layer-System komplett
- `mapping.rs`: Mapping-Hierarchie komplett
- `mesh.rs`: Mesh-Warping komplett
- `paint.rs`: Paint-System komplett
- `audio.rs`: Audio-Analyse komplett
- `audio_reactive.rs`: Audio-Reactive-Mappings komplett
- `animation.rs`: Keyframe-Animation komplett
- `shader_graph.rs`: Shader-Graph komplett
- `lut.rs`: LUT-System komplett
- `oscillator.rs`: Oscillator-System komplett
- `codegen.rs`: WGSL-Codegen komplett
- `monitor.rs`, `output.rs`: Output-Management komplett

**Fehlende Integration:**
- Audio-Backend-Verdrahtung (CPAL) fehlt
- Shader-Graph-zu-Render-Pipeline-Integration fehlt

**Dateipfade für @jules:**
- `crates/mapmap-core/src/audio.rs` – Audio-Analyse
- `crates/mapmap-core/src/audio_reactive.rs` – Audio-Reactive-Controller
- `crates/mapmap-core/src/shader_graph.rs` – Shader-Graph
- `crates/mapmap-core/Cargo.toml` – Feature `audio` aktivieren

---

### mapmap-render

**Status:** ✅ 95% implementiert, ⬜ 5% Integration fehlt

**Implementierte Module:**
- `compositor.rs`: Multi-Layer-Komposition komplett
- `edge_blend_renderer.rs`: Edge-Blending komplett
- `color_calibration_renderer.rs`: Farbkalibrierung komplett
- `mesh_renderer.rs`: Mesh-Warping komplett
- `oscillator_renderer.rs`: Oscillator-Effekte komplett
- `shader.rs`: Shader-Loader komplett
- `texture.rs`: Texture-Management komplett
- `backend.rs`: wgpu-Backend-Abstraktion komplett

**Fehlende Integration:**
- Effect-Chain-Renderer fehlt (`effect_chain_renderer.rs` erstellen)
- Shader-Graph-Integration in Render-Pipeline fehlt

**Dateipfade für @jules:**
- `crates/mapmap-render/src/compositor.rs` – Compositor
- `crates/mapmap-render/src/edge_blend_renderer.rs` – Edge-Blending
- `crates/mapmap-render/src/color_calibration_renderer.rs` – Color-Calib
- Neu erstellen: `crates/mapmap-render/src/effect_chain_renderer.rs` – Effect-Chain

---

### mapmap-media

**Status:** ✅ 85% implementiert, ⬜ 15% Features fehlen

**Implementierte Module:**
- `decoder.rs`: FFmpeg-Decoder komplett
- `image_decoder.rs`: Image-Decoder komplett (PNG, JPG, BMP, TGA)
- `player.rs`: Playback-State-Machine komplett
- `pipeline.rs`: Media-Pipeline komplett

**Fehlende Features:**
- GIF-Animation noch nicht vollständig
- Image-Sequence-Playback fehlt (walkdir-Dependency vorhanden, aber nicht genutzt)
- ProRes-Codec noch nicht getestet
- HAP/DXV-Codecs fehlen

**Dateipfade für @jules:**
- `crates/mapmap-media/src/player.rs` – Playback-State-Machine
- `crates/mapmap-media/src/decoder.rs` – FFmpeg-Decoder
- `crates/mapmap-media/src/image_decoder.rs` – Image-Decoder (GIF-Support hinzufügen)

---

### mapmap-ui

**Status:** ✅ 80% implementiert, ⬜ 20% Panels fehlen

**Implementierte Module:**
- `dashboard.rs`: Hauptansicht komplett
- `media_browser.rs`: Media-Browser komplett
- `mesh_editor.rs`: Mesh-Editor komplett
- `node_editor.rs`: Node-Editor komplett
- `shader_graph_editor.rs`: Shader-Graph-Editor komplett
- `timeline.rs`, `timeline_v2.rs`: Timeline komplett
- `undo_redo.rs`: Undo-Redo komplett
- `asset_manager.rs`: Asset-Manager komplett
- `theme.rs`: Theme komplett

**Fehlende Panels:**
- Audio-Input-Device-Selector fehlt
- OSC-Server-Config-Panel fehlt
- Effect-Chain-Editor fehlt
- Output-Config-Panel fehlt
- Project-Management-UI fehlt
- Cue-List-UI fehlt

**Dateipfade für @jules:**
- `crates/mapmap-ui/src/dashboard.rs` – Hauptansicht (Audio-Selector, OSC-Panel hinzufügen)
- `crates/mapmap-ui/src/shader_graph_editor.rs` – Shader-Graph-Editor (Effect-Chain-UI hinzufügen)
- Neu erstellen: `crates/mapmap-ui/src/audio_config.rs` – Audio-Config-Panel
- Neu erstellen: `crates/mapmap-ui/src/osc_config.rs` – OSC-Config-Panel
- Neu erstellen: `crates/mapmap-ui/src/output_config.rs` – Output-Config-Panel

---

### mapmap-control

**Status:** ✅ 90% implementiert, ⬜ 10% Integration fehlt

**Implementierte Module:**
- `osc/`: OSC-System komplett (Server, Client, Address-Parser, Types)
- `midi/`: MIDI-System komplett (Input, Output, Mapping, Learn, Clock, Profiles)
- `dmx/`: DMX-System komplett (Art-Net, sACN, Channels, Fixtures)
- `cue/`: Cue-System komplett (Cue, CueList, Crossfade, Triggers)
- `shortcuts/`: Shortcuts komplett (Bindings, Macros)
- `web/`: Web-API komplett (WebSocket, Routes, Handlers) – NICHT NUTZEN
- `manager.rs`: ControlManager komplett
- `target.rs`: ControlTarget komplett

**Fehlende Integration:**
- OSC-Events zu Layer/Paint/Effect-Parameter routen fehlt
- OSC-Feedback (State-Updates) fehlt
- OSC-Learn-Mode in UI fehlt
- MIDI-zu-Parameter-Routing fehlt (low priority)

**Dateipfade für @jules:**
- `crates/mapmap-control/src/osc/mod.rs` – OSC-Command-Schema definieren
- `crates/mapmap-control/src/osc/server.rs` – OSC-Server
- `crates/mapmap-control/src/manager.rs` – ControlManager (OSC-Routing hinzufügen)
- `crates/mapmap-control/Cargo.toml` – Feature `osc` aktivieren

---

### mapmap-io

**Status:** ⬜ 20% implementiert (Placeholder für Phase 5)

**Implementierte Module:**
- `source.rs`, `sink.rs`, `converter.rs`, `format.rs`: Abstractions komplett
- `ndi/`, `decklink/`, `spout/`, `syphon/`: Placeholders
- `stream/`: RTMP, SRT, Encoder komplett

**Fehlende Integration:**
- NDI, DeckLink, Spout, Syphon nicht implementiert (Phase 5)
- Virtual-Camera fehlt

**Dateipfade für @jules:**
- `crates/mapmap-io/src/` – Placeholder für Phase 5, aktuell nicht prioritär

---

### mapmap-ffi

**Status:** ⬜ 10% implementiert (Placeholder für Phase 5)

**Implementierte Module:**
- `lib.rs`: FFI-Error-Types

**Fehlende Integration:**
- NDI-FFI, DeckLink-FFI, Spout-FFI, Syphon-FFI nicht implementiert (Phase 5)

**Dateipfade für @jules:**
- `crates/mapmap-ffi/src/lib.rs` – Placeholder für Phase 5, aktuell nicht prioritär

---

### mapmap (Main Application)

**Status:** ✅ 70% implementiert, ⬜ 30% Integration fehlt

**Implementierte Module:**
- `main.rs`: Main-Loop komplett
- `window_manager.rs`: Window-Management komplett (Single-Window)

**Fehlende Integration:**
- Audio-Backend-Initialisierung fehlt
- OSC-Server-Initialisierung fehlt
- Multi-Window-Rendering fehlt
- Project-Save/Load fehlt

**Dateipfade für @jules:**
- `crates/mapmap/src/main.rs` – Main-Loop (Audio, OSC, Project-Load hinzufügen)
- `crates/mapmap/src/window_manager.rs` – Multi-Window-Support hinzufügen
- `crates/mapmap/Cargo.toml` – Features aktivieren

---

## Technologie-Stack und Entscheidungen

### Plattform-Support

**Verpflichtend:**
- ✅ **Windows 10/11** (WASAPI-Audio, Vulkan/DX12-Graphics)
- ✅ **Linux (Ubuntu 20.04+)** (ALSA/PulseAudio-Audio, Vulkan-Graphics)

**Optional (falls Mehraufwand vertretbar):**
- ⬜ **macOS 11+** (CoreAudio, Metal-Graphics)

### Audio-Backend-Entscheidung

**Technologie:** CPAL (Cross-Platform Audio Library)

**Plattformen:**
- **Windows:** WASAPI (Windows Audio Session API) – Low-Latency, Professional
- **Linux:** ALSA (Low-Level) oder PulseAudio (High-Level) oder JACK (Professional)
- **macOS:** CoreAudio (optional)

**Entscheidung:** Audio ist **verpflichtend**. Kein Build ohne Audio-Feature. Jeder Build muss Audio-Backend aktiviert haben.

### Control-Pfad-Entscheidung

**Primär:** OSC (Open Sound Control)
- ✅ UDP-basiert, Low-Latency
- ✅ Standard in VJ/Live-Performance-Software (TouchOSC, QLab, etc.)
- ✅ Flexibles Address-Schema
- ✅ Bidirektional (Control + Feedback)

**Sekundär (Low Priority):** MIDI
- ⬜ Optional, nur wenn Zeit vorhanden
- ⬜127-Wert-Auflösung (limitiert für präzise Steuerung)
- ⬜ Hardware-Abhängig

**Nicht nutzen:** WebSocket
- ❌ Komplexere Architektur
- ❌ Höhere Latenz als OSC
- ❌ OSC ist Standard in VJ-Industrie

### FFmpeg-Build

**Entscheidung:** FFmpeg ist **verpflichtend** für Media-Playback.

**Plattformen:**
- **Linux:** System-FFmpeg via `apt-get install libavcodec-dev libavformat-dev libavutil-dev`
- **Windows:** Pre-built FFmpeg-Binaries oder vcpkg
- **macOS:** Homebrew FFmpeg (optional)

**CI/CD:** FFmpeg in CI-Builds aktiviert (`--features ffmpeg`)

---

## Build- und Test-Strategie

### Build-Commands

```bash
# Full Build (Audio + FFmpeg)
cargo build --workspace --features audio,ffmpeg --release

# Check nur (schneller)
cargo check --workspace --features audio,ffmpeg

# Tests mit Audio + FFmpeg
cargo test --workspace --features audio,ffmpeg

# Benchmarks
cargo bench --workspace --features audio,ffmpeg
```

### Feature-Flags

**mapmap-core:**
- `default = ["audio"]` (verpflichtend)
- `audio` – CPAL-Audio-Backend

**mapmap-media:**
- `default = ["ffmpeg"]` (verpflichtend)
- `ffmpeg` – FFmpeg-Decoder

**mapmap-control:**
- `default = ["osc"]` (primär)
- `osc` – OSC-System
- `midi` – MIDI-System (optional, low priority)
- `http-api` – Web-API (optional, nicht nutzen)
- `full` – Alle Features

**mapmap (main):**
- `default = ["audio", "ffmpeg", "osc"]` (alle verpflichtend)

### CI/CD-Strategie

**GitHub Actions:**
- **Linux:** Ubuntu 20.04, System-Dependencies via `apt-get`, Build mit `--features audio,ffmpeg,osc`
- **Windows:** Windows Server 2022, FFmpeg via vcpkg, Build mit `--features audio,ffmpeg,osc`
- **macOS (optional):** macOS 12, FFmpeg via Homebrew, Build mit `--features audio,ffmpeg,osc`

**Tests:**
- Unit-Tests: Alle Crates
- Integration-Tests: OSC-Server, Audio-Analyzer, Media-Player
- Benchmarks: Texture-Upload, Video-Decode

**Env-Checks:**
- `scripts/check-ffmpeg-env.sh` – FFmpeg-Verfügbarkeit prüfen
- `check-audio-backend.sh` – Audio-Backend prüfen (neu erstellen)

---

## Zusammenfassung für @jules

**Kernentscheidungen:**
1. ✅ **Audio ist VERPFLICHTEND** – Jeder Build muss Audio-Feature aktiviert haben.
2. ✅ **OSC ist PRIMÄR** – OSC als Haupt-Control-Pfad, MIDI ist low priority.
3. ✅ **FFmpeg ist VERPFLICHTEND** – Media-Playback ohne FFmpeg nicht sinnvoll.
4. ✅ **Windows + Linux** – Hauptplattformen, macOS optional.
5. ❌ **Keine WebSocket-Control** – OSC ist Standard in VJ-Industrie.
6. ✅ **MCP-Server für AI-Integration** – NEU: Gemini CLI / Claude Desktop Unterstützung
7. ✅ **Mehrsprachigkeit (DE/EN)** – NEU: UI in Deutsch und Englisch

**Kritische Arbeitspakete (in Reihenfolge):**
1. 🟢 Audio-Build-Enforcement ✅ COMPLETED (2025-12-16)
2. 🟢 OSC-Command-Schema und Integration ✅ COMPLETED (2025-12-15)
3. 🟢 Media-Playback-State-Machine ✅ COMPLETED (2025-12-14)
4. 🟢 Effect-Chain-Hooks ✅ COMPLETED (2025-12-16)
5. 🟡 Projektformat und Persistenz (Save/Load implementiert)
6. 🟢 Multi-Window-Rendering (Phase 2 Completion)
7. 🟢 CI/CD mit Audio und FFmpeg (Builds automatisieren)
8. 🟢 Dokumentation und DX (Onboarding verbessern)
9. 🟡 **Internationalisierung (i18n)** – NEU: Deutsch/Englisch UI
10. 🟡 **MCP-Server Integration** – NEU: AI-gestützte Steuerung

**Nächste Schritte:**
1. Audio-Backend-Verdrahtung starten (`mapmap-core/src/audio/backend.rs` erstellen)
2. OSC-Command-Schema dokumentieren (`mapmap-control/src/osc/mod.rs` erweitern)
3. UI-Panels für Audio und OSC erstellen (`mapmap-ui/src/audio_config.rs`, `mapmap-ui/src/osc_config.rs`)
4. CI/CD anpassen (Audio + FFmpeg aktivieren)
5. Tests schreiben und laufen lassen
6. **NEU:** i18n-Framework (`fluent-rs`) integrieren
7. **NEU:** MCP-Server-Crate (`mapmap-mcp/`) erstellen

---

**Letzte Aktualisierung:** 2025-12-15  
**Erstellt von:** VjMapper Development Team  
**Für:** @jules und Contributors
