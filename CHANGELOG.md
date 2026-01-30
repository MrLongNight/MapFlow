# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2026-01-26: fix(ci): Ensure FFmpeg development libraries are installed in CI-01 pre-checks job to fix ffmpeg-sys-next build errors
- 2026-01-26: fix(ci): Improve library verification in CI-01 with robust pkg-config check loop and detailed diagnostics
- 2026-01-26: fix(ci): Fix missing FFmpeg DLLs in WiX installer by explicitly verifying and copying each required DLL to target/release in CI-09
- 2026-01-26: fix(scripts): Add missing libswresample-dev to install-ffmpeg-dev.sh
- 2026-01-26: fix(ci): Harden CI-01 workflow by ensuring X11 and system dependencies are installed across all jobs (pre-checks, quality), fixing build failures in non-release steps
- 2026-01-26: fix(ci): Ensure FFmpeg DLLs (avcodec-61, etc.) are explicitly copied to target/release for WiX installer in CI-09, verifying expected versions from vcpkg
- 2026-01-20: docs: Update Roadmap status for NDI and Hue integration (Tracker)
- 2026-01-19: fix(ci): Ensure VCPKG_ROOT is set and vcpkg integrated in release workflow
- 2026-01-18: fix(ci): Fix CI-09 workflow build error by explicitly installing vcpkg and ffmpeg (#287)
- 2026-01-18: chore: Clean up documentation structure and move audit reports (Archivist) (#286)
- 2026-01-18: feat(hue): Philips Hue Integration Overhaul & Merge Resolution (#b8dd83b)
- 2026-01-18: feat(core): Implement node-based module system with Media/Audio/NDI/Hue support (#484c78e)
- 2026-01-16: fix(ci): Fix Windows release workflow by adding dynamic FFmpeg integration (#270)
- 2026-01-16: fix(render): Fix headless crash in wgpu backend on CI (#269)
- 2026-01-16: perf(core): Optimize ModuleEvaluator allocations (Bolt) (#268)
- 2026-01-16: feat(ui): Improve Media Clip Region interaction with fluid drag and snapping (#267)
- 2026-01-16: style(ui): Refine Inspector hierarchy and visual style (#266)
- 2026-01-16: fix(security): Validate control inputs to prevent injection (#265)
- 2026-01-16: docs: Update documentation (Scribe) (#262)
- 2026-01-16: test(core): Add comprehensive ModuleEvaluator tests (Guardian) (#263)
- 2026-01-16: chore: Clean up repository artifacts (Archivist) (#261)
- 2026-01-15: feat(core): Implement stateful trigger infrastructure and random intervals (#252)
- 2026-01-15: assets: Add comprehensive icon library for UI (#252)
- 2026-01-15: chore: Add maintenance and setup scripts (#252)
- 2026-01-15: fix(render): Fix Windows surface format crash (#260)
- 2026-01-15: feat(ui): Media clip region editor (#258)
- 2026-01-15: feat(ui): Node graph visual refinements (#257)
- 2026-01-15: perf(core): Optimize FPS calculation with VecDeque (#256)
- 2026-01-15: fix(security): Input validation for UpdateLayer (#255)
- 2026-01-15: docs: Update libmpv documentation (#254)
- 2026-01-15: test(core): Add more core tests (#253)
- 2026-01-15: fix(render): Fix layer pipeline verification (#251)
- 2026-01-15: feat(audio): Complete AudioFFT Trigger Node Implementation (#249)
- 2026-01-15: feat(ui): Interactive clip region (#248)
- 2026-01-14: test(core): Guardian ModuleEvaluator coverage for triggers and chains
- 2026-01-13: test(core): Add robust tests for Layer-Transform and State-Serialization (#228)
- 2026-01-10: feat(render): Add SourceProperties to RenderOp for color/transform/flip (#b8453dc)
- 2026-01-10: feat(media): Add flip, reverse playback and seek controls to Media Node (#9f8163d)
- 2026-01-09: fix(security): Fix auth timing side-channel (#222)
- 2026-01-09: perf(render): Cache egui textures to reduce bind group creation overhead (#221)
- 2026-01-09: docs: Tracker update docs (#220)
- 2026-01-09: chore: Archivist cleanup (#219)
- 2026-01-09: test(core): Guardian test improvements (#218)
- 2026-01-09: feat(media): libmpv2 integration (#216)
- 2026-01-09: fix(security): Fix auth timing leak (#214)
- 2026-01-09: docs: reorganize documentation structure into subfolders (#210)
- 2026-01-09: test(core): Add robust tests for Transform and Mesh logic (#212)
- 2026-01-09: perf(render): Optimize texture sync loop in render path (#215)
- 2026-01-09: docs: add README.md for core workspace crates (#213)
- 2026-01-09: docs: Prioritize libraries by Core function relevance
- 2026-01-09: docs: Expand Phase 7 with detailed implementation plan
- 2026-01-08: feat(ui): add Source Scaling UI controls for MediaFile nodes
- 2026-01-08: feat(ui): add Output Settings UI controls for Projector nodes
- 2026-01-08: docs: Clarify Cue-System integration and Phase 7 playback modes
- 2026-01-07: feat(ui): add separate toggle buttons for Controls and Preview panels
- 2026-01-07: fix(control): default web server bind address to 127.0.0.1 for security (#207)
- 2026-01-07: perf(ci): optimize GitHub Actions to reduce minutes usage
- 2026-01-07: test(core): Guardian Module Tests - socket, mesh, CRUD (#205)
- 2026-01-07: docs: Add crate READMEs (#196)
- 2026-01-09: docs: 📚 Scribe: Add mapmap-mcp README and update ROADMAP/CHANGELOG
- 2026-01-02: feat(render): Advanced Output Rendering & NDI Integration (#155)
- 2026-01-02: feat: Advanced Link System & Audio Trigger Improvements (#157)
- 2026-01-02: fix: Remove build artifact files from repository (#156)
- 2026-01-02: feat: Effect Chain Rendering with Configurable Parameters (#153)
- 2026-01-02: fix: Complete mesh implementations and resolve layer ID collisions (#154)
- 2026-01-01: feat(presets): Add NDI Source, NDI Output, Spout Source, Spout Output presets
- 2026-01-01: feat(presets): Increase node spacing (200→300px) and add missing output connections
- 2026-01-01: feat(ui): Add category to node titles (e.g., "🎬 Source: Media File")
- 2026-01-01: feat(ui): Add right-click context menu for nodes with Properties and Delete options
- 2026-01-01: feat(jules): Add 4 specialized agent roles (Guardian, Scribe, Tracker, Archivist)
- 2025-12-31: 🛡️ Sentinel: [HIGH] Add HTTP Security Headers (Defense in Depth) (#143)
- 2025-12-31: merge: PR #139 - Spout Integration (Core + UI + IO)
- 2025-12-31: merge: PR #140 - Assignment System Foundation (AssignmentManager)
- 2025-12-31: fix: Resolve all compiler warnings (unused variables, missing docs, ndi feature)
- 2025-12-31: feat: Add `sd_480p30_rgba()` format preset to VideoFormat
- 2025-12-31: feat: Add `ndi` feature flag to main mapmap crate
- 2025-12-31: Implement Spout Source and Output Nodes (#139)
- 2025-12-31: feat(ui): Unified Add Node menu with search (#131)
- 2025-12-31: Finalize Cue System UI Integration (#142)
- 2025-12-31: Feat: Implement UI Panel for Shortcut Editor (#141)
- 2025-12-31: ⚡ Bolt: Remove unnecessary allocations in render/UI hot paths (#144)
- 2025-12-31: merge: Resolve NDI/Spout conflicts with Unified Node Menu (#131, #137, #139)
- 2025-12-31: feat(ui): Unified "Add Node" menu with search and NDI/Spout integration
- 2025-12-30: feat(ui): Add proper Fader SVG icon for MIDI controller button (replaces broken emoji)
- 2025-12-30: feat(ui): Remove Layers section and Inspector Panel from sidebar (use Module Canvas)
- 2025-12-30: feat(config): Complete settings persistence (window size/position, panel visibility)
- 2025-12-30: fix(config): Load all user settings at startup (audio device, target FPS, panel states)
- 2025-12-30: fix(autosave): Use proper .mflow extension and store in user data directory
- 2025-12-29: feat(ui): Unified "Add Node" menu with quick-search replacing 8 toolbar dropdowns
- 2025-12-29: feat(ui): Added BPM display and playback controls to main toolbar (removed from sidebar)
- 2025-12-29: fix(control): Stabilize MIDI clock BPM with sliding window average
- 2025-12-29: fix(audio): Improve BPM stability with median filtering and outlier removal
- 2025-12-29: fix(ui): Fix app settings window toggle logic and ID stability
- 2025-12-31: refactor: optimize logging structure and levels (#138)
- 2025-12-31: Knoteneigenschaften als Popup-Dialog implementieren (#136)
- 2025-12-31: ⚡ Bolt: Implement Vertex Buffer Caching (#133)
- 2025-12-31: 🛡️ Sentinel: [CRITICAL] Fix missing API authentication enforcement (#134)
- 2025-12-31: ⚡ Bolt: optimize mesh vector allocations (#135)
- 2025-12-29: 🛡️ Sentinel: Fix timing attack in API key validation (#132)
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
- 2025-12-12: Fix: `VideoEncoder` keyframe logic (first frame is now keyframe) and updated `test_video_encoder_keyframe` to match.
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
