# 🦀 MapFlow Project Context (GEMINI.md)

## 🚀 Projektübersicht
**MapFlow** ist eine modulare, knotenbasierte **VJ (Video Jockey) Software**, die in **Rust** entwickelt wurde. Sie ist für hochperformante Echtzeit-Visualsynthesis und Projection Mapping konzipiert.

- **Kern-Architektur**: Modularer Rust-Workspace mit spezialisierten Crates.
- **Grafik-Engine**: `wgpu` für Rendering, `bevy` für 3D/Partikel-Integration.
- **UI-Framework**: `egui` (via `eframe`).
- **Medien-Handling**: FFmpeg-next und libmpv2 für Video-Dekodierung.
- **Steuerung**: MIDI, OSC und Philips Hue Integration.
- **KI-Integration**: Eingebauter MCP-Server (`mapmap-mcp`) und Jules AI Unterstützung.

## 📦 Workspace Module
- `mapmap`: Die Hauptanwendung (Binary).
- `mapmap-core`: Zentrale Datenstrukturen, Layer-System und Logik.
- `mapmap-ui`: UI-Implementierung mit `egui`.
- `mapmap-render`: WGPU-basierte Rendering-Engine.
- `mapmap-bevy`: Integration der Bevy-Engine.
- `mapmap-mcp`: MCP-Server für KI-Agenten-Interaktion.
- `mapmap-media`: Medien-Dekodierung und Wiedergabe.
- `mapmap-control`: OSC/MIDI Input-Handling.
- `mapmap-io`: NDI und Spout Unterstützung.

## 🛠️ Build & Entwicklung

### Wichtige Befehle
- **Bauen**: `cargo build`
- **Ausführen**: `cargo run -p mapmap`
- **Testen**: `cargo test --workspace` (oder `cargo make test`)
- **Linting**: `cargo clippy --workspace` (oder `cargo make lint`)
- **Lokale CI**: `cargo make ci-local` (erfordert `cargo-make`)
- **Dokumentation**: `cargo doc --no-deps --workspace`

### Scripts
- `scripts/run_mapflow.bat`: Windows Batch-Script zum Starten.
- `scripts/jules-setup.sh`: Setup für die Jules KI-Integration.
- `scripts/copy_ffmpeg_dlls.bat`: Hilfsscript für FFmpeg-Abhängigkeiten unter Windows.

## 🤖 KI-Agenten & Automatisierung
Das Projekt ist stark auf die Zusammenarbeit mit KI-Agenten (insbesondere **Jules**) optimiert.
- **Jules Integration**: Automatisierte Issue-Erstellung und PR-Handling (siehe `.jules/SETUP_GUIDE.md`).
- **Labels**: Nutzt spezifische Labels wie `jules-task` und `jules-pr` für die Workflow-Steuerung.
- **Workflows**: Umfangreiche GitHub Actions in `.github/workflows/` für CI/CD, Auto-Merge und Changelog-Updates.

## 📝 Entwicklungskonventionen
- **Safety First**: Strikte Einhaltung von Rust-Sicherheitsgarantien.
- **Modularität**: Neue Features sollten in das passende Crate integriert oder als neues Crate angelegt werden.
- **Performance**: Echtzeit-Fähigkeit (min. 60 FPS) steht bei Rendering-Änderungen im Vordergrund.
- **Logging**: Nutzt `tracing` für strukturiertes Logging.
- **Shader**: Shader befinden sich in `shaders/` und nutzen WGSL.

## 📂 Wichtige Verzeichnisse
- `crates/`: Der Quellcode der modularen Workspace-Komponenten.
- `shaders/`: WGSL-Shader für Effekte und Rendering.
- `assets/`: Icons und statische Ressourcen.
- `docs/`: Umfangreiche Dokumentation (Architektur, API, User Guide).
- `.agent/`: Workflows und Pläne für KI-Agenten.
- `.jules/`: Spezifische Konfigurationen für den Jules-Agenten.
