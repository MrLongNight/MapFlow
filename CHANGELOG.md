# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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
