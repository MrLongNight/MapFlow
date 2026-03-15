# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2026-03-14: docs: Remove broken pull request template link from GitHub workflows README (#-)
- 2026-03-13: docs: Update mapmap crate README to point to new semantic documentation structure.
- 2026-03-12: fix(ci): Fix pre-commit checks after UI config additions (#1029)
- 2026-03-12: fix(ci): Fix the Windows MSI release step by removing the stray `-- --release` from the `cargo wix` invocation so WiX packages the existing release build correctly (follow-up to Windows release workflow regression, run 22981970147).
- 2026-03-12: fix(ci): Correct the Windows MSI release step by replacing the unsupported `cargo wix --no-check-includes` flag with `--no-build` in the release workflow (follow-up to Windows release workflow regression, run 22958743385).
- 2026-03-10: fix(ci): Fix Windows release `vcpkg` baseline checkout by replacing cached shallow `vcpkg` clones with full-history bootstraps, and restore `SetMeterStyle` action handling for release builds.
- 2026-03-09: fix(ui): Standardize panel layout and resolve UI consistency gaps (MF-011).
- 2026-03-08: feat(ui): Implement Toast Notification system for engine errors and status updates (MF-023).
- 2026-03-08: fix(core): Refactor Trigger System logic for synchronized multi-module evaluation (MF-038).
- 2026-03-08: fix(ci): Resolve workspace test failures by updating audio frequency band counts (7 -> 9) (MF-034).
- 2026-03-08: fix(hue): Replace OpenSSL-disabled stub with real DTLS implementation using webrtc-dtls (ring) (MF-009).
- 2026-03-08: ui: Fix empty states visual clarity and remove hold-to-confirm pattern for standard resets (#970)
- 2026-03-08: docs: Fix broken documentation links to new semantic structure (#971)
- 2026-03-08: refactor: Modularize app logic into orchestrators and restore audio analysis functionality (226202b)
- 2026-03-06: fix(security): Fix Information Disclosure via Hardcoded Developer Paths (#929)
- 2026-03-03: docs: Repariere veraltete Links und Ordner-Referenzen in der Dokumentation (0[1-9]-* -> neue Struktur).
- 2026-03-02: fix(stability): Resolve main branch build failures, failing tests, and clippy warnings.

## [0.2.0] - 2026-02-27
### Added
- **UI:** Comprehensive migration from ImGui to **egui** for the entire interface.
- **Node-Editor:** Custom visual programming interface for complex logic and triggers.
- **Timeline:** Keyframe-based animation system for all layer parameters.
- **Project-Management:** Persistent project files (RON/JSON) and autosave support.
- **Audio:** Multi-band FFT analysis with beat detection and onset detection.
- **Rendering:** Advanced WGPU pipeline with support for effect chains and masks.

### Fixed
- **Core:** Resolved numerous race conditions in the evaluation loop.
- **Media:** Optimized GPU texture uploads for high-resolution video playback.
- **UI:** Improved responsiveness and layout stability on various screen sizes.

## [0.1.0] - 2025-12-15
### Added
- Initial release of MapFlow (VJMapper).
- Basic layer management and transformation.
- FFmpeg-based video decoding.
- Simple MIDI and OSC support.
- Basic shader effects.
