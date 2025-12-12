# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2025-12-12: 🎨 Palette: Add Tooltips to Dashboard Controls (#41)
- 2025-12-11: feat(media): Implement robust media playback state machine (#40)

### Fixed

- **CI:** Add `toolchain: stable` to the build workflow to fix CI failures. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
- **UI:** Fix incorrect import path for media player enums in `dashboard.rs`. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))

### Added

- **Media:** Implement a robust and fault-tolerant media playback state machine with a command-based control system, validated state transitions, and comprehensive unit tests. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
- **UI:** Add a speed slider, loop mode selector, and timeline scrubber to the dashboard for media playback control. ([#39](https://github.com/MrLongNight/VjMapper/pull/39))
