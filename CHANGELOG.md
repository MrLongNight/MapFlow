# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2025-12-20: fix: Correct Windows release workflow by adding missing avfilter and avdevice components to the FFmpeg installation. (#54)
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

- **CI:** Add `toolchain: stable` to the build workflow to fix CI failures. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
- **UI:** Fix incorrect import path for media player enums in `dashboard.rs`. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))

### Added

- **Media:** Implement a robust and fault-tolerant media playback state machine with a command-based control system, validated state transitions, and comprehensive unit tests. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
- **UI:** Add a speed slider, loop mode selector, and timeline scrubber to the dashboard for media playback control. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
