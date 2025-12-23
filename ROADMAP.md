# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 1.3  
> **Stand:** 2025-12-23  
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

### General Updates
- 🟡 **Rebranding: MapFlow -> MapFlow**
  - ✅ Rename Project (2025-12-22)
  - 🟡 Update UI Strings & Docs (In Progress)
  - 🟡 Rename WiX Installer Config (In Progress)

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
  - ✅ GPU-beschleunigte Color-Grading

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

- ✅ **Audio-Backend-Integration** (COMPLETED 2025-12-19)
  - ✅ CPAL-Backend verdrahtet (Feature: `audio` in `mapmap-core/Cargo.toml`)
  - ✅ Windows: WASAPI-Backend über CPAL integriert
  - ✅ Linux: ALSA/PulseAudio-Backend über CPAL integriert
  - ⬜ macOS: CoreAudio-Backend (optional, ungetestet)
  - ✅ Audio-Input-Device-Auswahl in UI (Dashboard)
  - ⬜ Audio-Stream in Media-Pipeline verdrahten (Phase 2)
  - ⬜ Latenz-Kompensation implementieren (Phase 3)

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

- ✅ **UI Panel Migration Status (egui)** – COMPLETED (2025-12-23)
  - ✅ Transform Controls (`transform_panel.rs`) – Migriert
  - ✅ Paint Manager (`paint_panel.rs`) – Migriert
  - ✅ Audio Visualization (`audio_panel.rs`) – Migriert
  - ✅ Main Menu & Toolbar (`menu_bar.rs`) – Migriert
  - ✅ Layer Manager (`render_layer_panel`) – Migriert (COMPLETED 2025-12-22)
  - ✅ Mapping Manager (`render_mapping_panel`) – Migriert (COMPLETED PR #97, 2025-12-23)
  - ✅ Output Configuration (`output_panel.rs`) – Migriert (COMPLETED 2025-12-23)
  - ✅ Edge Blend & Color Calibration (`edge_blend_panel.rs`) – Migriert (COMPLETED Verified 2025-12-23)
  - ✅ Oscillator Control (`oscillator_panel.rs`) – Migriert (COMPLETED Verified 2025-12-23)
  - ✅ Shader Graph Editor (`node_editor.rs`) – Migriert (COMPLETED 2025-12-23)
  - ✅ Cue List (`cue_panel.rs`) – Migriert (COMPLETED 2025-12-23)
  - ✅ OSC Panel (`osc_panel.rs`) – Migriert (COMPLETED 2025-12-23)
  - ✅ ImGui Removal (Code Cleanup) – COMPLETED (2025-12-23)

- 🟡 **UI Redesign (Resolume Style)**
  - 🔄 Cyber Dark Theme (Jules Session: 15619292958684189574)
  - 🔄 Docking Layout & Unified Inspector (Jules Session: 12159547036669143793)
  - ✅ Icon System Infrastructure (Ready for Assets)

- 🟡 **Internationalisierung (i18n) – NEU**
  - ✅ Sprachauswahl UI (Deutsch / Englisch)
  - ✅ `fluent` oder `rust-i18n` Crate integrieren
  - ✅ Übersetzungsdateien (`locales/de.ftl`, `locales/en.ftl`)
  - ✅ Dynamischer Sprachwechsel zur Laufzeit
  - ✅ Persistierung der Spracheinstellung in User-Config (COMPLETED 2025-12-21)
  - ⬜ Alle UI-Strings extrahieren und übersetzen

### Phase 7: Advanced Show Control (Module-Based Timeline) – PLANNED

- ⬜ **Architecture Refactor (Timeline V3)**
  - ⬜ **Module Concept**: `TimelineModule` struct (Triggers, Resources, Assigned Layers)
  - ⬜ **Modes**: Automatic vs. Manual/Hybrid Playback
  - ⬜ **Track System**: Module-based tracks with collision detection (Track-based only)
  - ⬜ **Data Model**: Migration from simple Keyframes to rich Modules

- ⬜ **UI Components**
  - ⬜ **Module Canvas**: Drag & Drop palette for module creation
  - ⬜ **Timeline Editor**: Drag & Drop arrangement, Snapping, Multi-Track
  - ⬜ **Module Editor**: Multi-tab interface for parallel module editing (Double-click action)
  - ⬜ **Transition Modules**: Visual transition editing with timeline-based duration

- ⬜ **Features**
  - ⬜ **Undo/Redo**: Full Command-Pattern integration for Timeline actions
  - ⬜ **Templates**: Save/Load Module configurations
  - ⬜ **Library**: Reusable Module presets

### MCP-Server Integration (Model Context Protocol) – NEU

- ✅ **MCP-Server Implementierung (COMPLETED 2025-12-18)**
  - ✅ MCP-Server-Crate erstellt (`mapmap-mcp/`)
  - ✅ JSON-RPC 2.0 Transport (stdio/SSE)
  - ✅ Tool-Definitionen für MapFlow-Funktionen implementiert
  - ✅ Resource-Definitionen implementiert
  - ✅ Prompt-Definitionen für AI-Assistenz implementiert
  - ✅ Integration mit Gemini CLI / Claude Desktop
  - ✅ Dokumentation: MCP-API-Referenz (TODO)

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

- 🟡 **Fehlende Tests**
  - ✅ Audio-System-Tests mit Audio-Feature aktiviert (COMPLETED 2025-12-21, 16 Tests)
  - ✅ OSC-Integration-Tests (COMPLETED 2025-12-21, 19 Tests)
  - ✅ Project-Save/Load-Tests (COMPLETED PR #68, 2025-12-22)
  - ✅ Multi-Output-Rendering-Tests (COMPLETED 2025-12-22)
  - ⬜ Effect-Chain-Tests
  - ⬜ End-to-End-Tests

### Packaging / Developer Experience (DX)

- ✅ **CI/CD** (`.github/workflows/`)
  - ✅ CI-Workflow vorhanden und optimiert (2025-12-18)
  - ✅ Workflow-Lints behoben (deprecation warnings entfernt)
  - ✅ FFmpeg-Installation in Linux-Builds korrigiert (libavutil-Fehler behoben)
  - ✅ Toolchain-Updates (stable verwendet, dtolnay/rust-toolchain@stable)
  - ✅ Windows-Build-Fixes (vcpkg-Pfade, git-ownership)
  - ✅ Audio-Feature in CI aktiviert
  - ⬜ FFmpeg in CI-Builds aktivieren fehlt
  - ✅ Windows-CI-Builds (COMPLETED 2025-12-21, non-blocking)
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
