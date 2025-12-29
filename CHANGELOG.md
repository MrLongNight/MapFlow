# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2025-12-29: feat(audio): Add real-time BPM detection from beat intervals
- 2025-12-29: feat(module-canvas): Add live audio trigger visualization (VU meter, threshold, glow)
- 2025-12-29: feat(audio): Implement AudioAnalyzerV2 with working FFT analysis (replaces defective AudioAnalyzer)
- 2025-12-29: fix(config): Add missing selected_audio_device field to UserConfig test
- 2025-12-28: 🎨 Palette: Add tooltips to Layer Panel controls (#125)
- 2025-12-28: feat(ui): implement stereo audio meter with Retro and Digital styles (#128)
- 2025-12-28: ⚡ Bolt: Optimize ModuleSocketType to be Copy and remove redundant clones (#127)
- 2025-12-28: 🛡️ Sentinel: [HIGH] Fix overly permissive CORS configuration (#126)
- 2025-12-28: Performance Optimierungen - perf(core): avoid allocation in visible_layers and fix formatting (#122)
- 2025-12-26: Remove trailing whitespace in controller_overlay_panel.rs (#118)
- 2025-12-26: Fix PR check issues (#117)
- 2025-12-26: resources/controllers/ecler_nuo4/elements.json hinzugefügt, um CI-Build-Fehler aus PR #117 zu beheben
- 2025-12-26: Trailing whitespace in module_canvas.rs entfernt, CI-Fix für PR #117
- 2025-12-26: test: enhance mapmap-core test coverage for layers (#114)
- 2025-12-25: feat: Audio Meter Styles (Retro & Digital) (#112)
- 2025-12-25: Implement Module Canvas System Foundation (#111)
- 2025-12-24: Complete Icon System Integration (#110)
- fix(ci): Mark unstable GPU tests in `multi_output_tests.rs` as ignored to prevent CI failures.

## [0.2.0] - 2025-12-22: MapFlow Rebranding
- 2025-12-23: Fix: Resize-Prozess bei Fenstergrößenanpassung robust gegen fehlende Größenangaben gemacht (siehe PR #104)
- **REBRANDING:** Das Projekt wurde von **MapFlow** in **MapFlow** umbenannt.
## [0.2.0] - 2025-12-23: MapFlow & UI Modernization

### Rebranding
- **REBRANDING:** Das Projekt wurde von **VjMapper** in **MapFlow** umbenannt.
  - Windows Executable: `mapflow.exe`
  - Linux Executable: `mapflow`
  - Repository URL: `https://github.com/MrLongNight/MapFlow`
  - Neue CI Icons und Application Icons integriert.
  - Alle Dokumentationen aktualisiert.

### UI Migration (Phase 6 COMPLETE)
- 2025-12-23: **COMPLETE ImGui Removal** – Alle Panels auf egui migriert
- 2025-12-23: Cyber Dark Theme implementiert (Jules Session)
- 2025-12-23: UI Modernization mit Themes, Scaling, und Docking Layout
- 2025-12-23: Node Editor (Shader Graph) vollständig aktiviert
- 2025-12-23: Timeline V2 Panel vollständig aktiviert
- 2025-12-23: Mapping Manager Panel migriert (PR #97)
- 2025-12-23: Output Panel vollständig migriert
- 2025-12-23: Edge Blend & Oscillator Panels verifiziert
- 2025-12-23: OSC Panel und Cue Panel migriert
- 2025-12-22: Layer Manager Panel migriert

### Multi-PC Architecture (Phase 8 Documentation)
- 2025-12-23: Multi-PC-Architektur umfassend dokumentiert
  - Option A: NDI Video-Streaming
  - Option B: Distributed Rendering
  - Option C: Legacy Slave Client (H.264/RTSP)
  - Option D: Raspberry Pi Player

### Tests & CI
- 2025-12-22: Effect Chain Integration Tests hinzugefügt (PR #100)
- 2025-12-22: Cue System UI Panel implementiert (PR #99)
- 2025-12-22: Multi-Output-Rendering-Tests abgeschlossen

### Audio & Media Pipeline (COMPLETED 2025-12-23)
- **Audio-Media-Pipeline Integration**: Audio-Stream vollständig in Media-Pipeline integriert
  - Konfigurierbare Sample-Rate (default: 44100 Hz)
  - Ring-Buffer für Audio-Analyse-Historie
  - Audio-Position-Tracking für Frame-genaue Synchronisation
  - Pipeline-Statistiken (Samples processed, frames analyzed, buffer fill level)
- **Latenz-Kompensation**: Implementiert mit konfigurierbarem Delay (0-500ms)
  - Automatische Latenz-Schätzung basierend auf Buffer-Status
  - Zeitstempel-basierte Analyse-Auswahl für Audio-Video-Sync
  - Smoothed-Analysis für geglättete Audio-Reaktivität
- **GIF-Animation**: Vollständig implementiert mit korrektem Timing
  - Frame-genaue Delay-Unterstützung aus GIF-Metadaten
  - Loop-Unterstützung
- **Image-Sequence-Playback**: Directory-basierte Bild-Sequenzen
  - Automatische Erkennung von Bild-Formaten (PNG, JPG, TIFF, BMP, WebP)
  - Sortierte Wiedergabe nach Dateiname
  - Konfigurierbares FPS
- **GPU-Upload-Optimierung**: Staging-Buffer-Pool implementiert
  - Automatische Entscheidung zwischen Direct-Upload (<64KB) und Staged-Upload (>64KB)
  - Row-Padding für wgpu Alignment Requirements
  - Reduzierte CPU-GPU-Synchronisierungen für Video-Streaming

## [0.1.0] - Unreleased
- 2025-12-22: [CONSOLIDATED] All Jules UI Migrations (#78)
- 2025-12-22: Migrate Audio Visualization Panel to egui (#72)
- 2025-12-22: Add Project Save/Load Tests (#68)
- 2025-12-22: Migrate Paint Manager Panel from ImGui to egui (#73)
- 2025-12-22: Migrate Transform Controls Panel to egui (#70)
- 2025-12-22: Fix: CI-Testfehler und Clippy-Warnungen (#77)
- 2025-12-21: feat: Complete media pipeline for GIFs and image sequences (#67)
- 2025-12-21: fix(ci): Correct formatting in mapmap-media/src/lib.rs (#65)
- 2025-12-21: feat(media): Complete media pipeline for GIFs and image sequences (#65)
- 2025-12-21: Implement Cue System UI Panel (#66)
- 2025-12-21: test(osc): Expand OSC address routing integration tests (#62)
- 2025-12-21: test(audio): Expand audio system unit tests (#61)
- 2025-12-21: ci: Add Windows build job to CI-01 workflow (#60)
- 2025-12-21: feat(i18n): Add user config persistence for language settings (#59)
- 2025-12-20: docs(roadmap): Mark audio backend integration as completed (#56)
- 2025-12-19: feat(mcp): Add media playback tools and fix send_osc handler (#55)
- 2025-12-16: Enforce Audio Build and Integrate CPAL Backend (#51)
- 2025-12-14: Refactor Media Playback State Machine and Control System (#52)
- 2025-12-14: Refactor: Complete rewrite of Media Playback State Machine and Control System Refactoring.
    - `mapmap-media`: New `PlaybackState`, `PlaybackCommand`, `PlaybackStatus`. Removed legacy modes. Robust State Machine implementation in `player.rs`.
    - `mapmap-control`: Removed `OscLearn`, `MidiLearn`. Simplified `OscMapping` and `MidiMapping` (HashMap based). Robust initialization for missing backends.
    - `mapmap-ui`: Updated `Dashboard` and `AppUI` to match new Media API (Loop/PlayOnce modes).
- 2025-12-14: fix: resolve winit/wgpu dependency conflicts in mapmap-ui (#50)
- 2025-12-12: Fix: `mapmap-control` doc test for OSC server updated to use `poll_packet` instead of non-existent `poll_event`.
- 2025-12-12: Fix: `test_backend_creation` now handles headless CI environments by skipping gracefully when GPU backend unavailable.
- 2025-12-12: Fix: Corrected `VideoEncoder` keyframe logic (first frame is now keyframe) and updated `test_video_encoder_keyframe` to match.
- 2025-12-12: Fix: MIDI unit tests (input/output) now accept initialization failures in CI environments where MIDI devices are unavailable.
- 2025-12-12: Fix: Alle aktuellen dead_code-Stellen mit #[allow(dead_code)] und Erklärung markiert, so dass der Build wieder erfolgreich läuft. (Siehe auch DEAD_CODE_GUIDE.md)
- 2025-12-12: fix: CI `alsa-sys` and `ffmpeg-sys-next` build failures by installing `libasound2-dev` and FFmpeg dev libs in `quality` job.
- 2025-12-12: fix: Updated examples `simple_render.rs` and `hello_world_projection.rs` for `winit` 0.29 and `wgpu` 0.19.
- 2025-12-12: CI: Umstellung auf Rust Nightly für Edition 2024 Support (#50).
- 2025-12-12: fix: Import-Fehler in mapmap/src/main.rs behoben (mapmap-render Refactoring).
- 2025-12-12: Behoben: Version-Konflikte bei winit (von 0.27.5 auf 0.29) und Kompatibilitätsissues mit wgpu 0.19 in mapmap-ui.
- 2025-12-12: Update Roadmap: Phase 6 UI Migration & Phase 7 Packaging Status (#47)
- 2025-12-12: fix: resolve CI config, winres dependency and dashboard loop logic (#46)
- 2025-12-12: fix: stabilize build, CI and control tests (#45)
- 2025-12-12: fix: CI Workflow fixes (Package Name, VS Verification, Release Artifacts)
- 2025-12-12: fix: Build stabilization (wgpu lifetimes, lockfile corruption)
- 2025-12-12: test: Complete unit tests for Control Web API
- 2025-12-12: fix: Feature flag guards for Control module
- 2025-12-12: fix: Resolve WGPU compilation errors in mapmap-render (removed compilation_options)
- 2025-12-12: fix: Update winit dependency in mapmap-ui to 0.27.5 with features
- 2025-12-12: fix: Refactor dashboard assignment logic
- 2025-12-12: feat: Release Workflow & Installers (MSI/Deb) (#44)
- 2025-12-12: docs: Add Multi-PC Feasibility Study (#43)
- 2025-12-12: 🎨 Palette: Add Tooltips to Dashboard Controls (#41)
- 2025-12-11: feat(media): Implement robust media playback state machine (#40)

### Fixed

- **CI:** Add `toolchain: stable` to the build workflow to fix CI failures. ([#39](https://github.com/MrLongNight/MapFlow/pull/39))
- **UI:** Fix incorrect import path for media player enums in `dashboard.rs`. ([#39](https://github.com/MrLongNight/MapFlow/pull/39))

### Added

- **Media:** Implement a robust and fault-tolerant media playback state machine with a command-based control system, validated state transitions, and comprehensive unit tests. ([#39](https://github.com/MrLongNight/MapFlow/pull/39))
- **UI:** Add a speed slider, loop mode selector, and timeline scrubber to the dashboard for media playback control. ([#39](https://github.com/MrLongNight/MapFlow/pull/39))
