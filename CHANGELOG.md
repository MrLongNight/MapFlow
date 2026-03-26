# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- 2026-03-26: feat: Introduce a PowerShell script for GitHub API interactions and a new directory for user manual screenshots. (#c900b1a9)
- 2026-03-26: docs: update user manual to match Vorce v2 rebranding (#1419)
- 2026-03-26: feat: update protocols and external IDs to Vorce naming (#1418)
- 2026-03-26: chore: merge local fixes from pr-1417 into main (#df9694c9)
- 2026-03-26: fix: reduce StrictMode to 1.0 and apply formatting fixes to scripts/jules (#bde0d4b2)
- 2026-03-26: test: test commit to reproduce error (#25a52234)
- 2026-03-26: feat: Introduce Jules scripts for managing GitHub issue and project board states. (#f5def575)
- 2026-03-25: feat: implement script-owned branding and runtime path rename to Vorce (#1417)
- 2026-03-25: fix: resolve toolchain inputs and fmt issue (#228d7aa1)
- 2026-03-25: feat: Introduce Jules system for GitHub issue orchestration and state management. (#50ae35e8)
- 2026-03-25: feat: implement script-owned branding and runtime path rename to Vorce (#c83e0da8)
- 2026-03-25: feat: Add a PowerShell script for master issue cycle automation, CI/CD validation and auto-merge GitHub Actions workflows, and documentation for Jules scripts. (#72458664)
- 2026-03-25: fix: 🛡️ Sentinel: [CRITICAL/HIGH] Fix path traversal and validation bypassing for OSC/MIDI inputs (#1414)
- 2026-03-25: perf: hoist search_query to_lowercase out of filter closure (#1415)
- 2026-03-25: feat: Feature: Rename MapFlow and mapmap crates to Vorce (#1408)
- 2026-03-24: chore: resolve local main divergence (#caf28b94)
- 2026-03-24: feat: 🎨 UX: Improve accessibility labels and tooltips in hold-to-confirm widgets (#1412)
- 2026-03-24: fix: Fix CI format and workflow issues after rename (#92acdee0)
- 2026-03-24: feat: Rename MapFlow and mapmap to Vorce (#7aeeb3ca)
- 2026-03-24: fix: resolve unused imports and variables in spout.rs and fix workspace members (#08bce072)
- 2026-03-24: fix: Apply standard unsupported warning styles and fix empty states (#1411)
- 2026-03-24: chore: Move misplaced files out of root according to rules (#1409)
- 2026-03-24: test: 🧪 Guardian: Add tests for ModuleManager repair functions (#1407)
- 2026-03-24: docs: 📚 Scribe: Fix broken documentation links and clean up archived audits (#1406)
- 2026-03-24: perf: ⚡ Bolt: Replace intermediate HashSets with Vecs for small lookups (#1410)
- 2026-03-25: refactor: Rename Cargo workspace and package identifiers from mapmap to vorce (#1204)
- 2026-03-24: docs: Fix deprecated ROADMAP.md references in documentation
- 2026-03-23: docs: Rebrand references from MapMap to MapFlow (#1379)
- 2026-03-21: test: establish release smoke baseline for visual capture (#1339)
- 2026-03-21: fix: Gate External I/O nodes without full runtime paths (#1321)
- 2026-03-19: refactor: Refactor Node Inspector Previews to a Family-Wide Standard (#1300)
- 2026-03-19: feat: Standardize RenderQueue Diagnostics Contract (#1282)
- 2026-03-19: feat: Implement uniform inspector previews for all node types (#1252)
- 2026-03-17: ui: Style AssignmentPanel to match Cyber Dark theme (#1226)
- 2026-03-18: fix(ci): Limit self-hosted runner resource usage (max 4 threads for build) and enable automatic cancellation of stale jobs to prevent system lockups.
- 2026-03-18: docs: Fix broken ROADMAP.md links (#1232)
- 2026-03-18: fix(ci): Final fix for self-hosted post-merge workflow (standardized checkout paths, fixed WSL bash conflict, and improved `vcpkg` detection logic).
- 2026-03-16: refactor: Decompose module_canvas inspector mod.rs (#1219)
- 2026-03-16: refactor: Decompose effect_chain_renderer.rs correctly (no regressions) (#-)
- 2026-03-16: refactor: Decompose ModuleEvaluator into submodules (#1188)
- 2026-03-16: refactor: Decompose module_canvas/utils.rs (#1211)
- 2026-03-16: refactor: Decompose Inspector UI into modules (#1213)
- 2026-03-16: refactor: Decompose controller overlay panel (#1209)
- 2026-03-16: refactor: Move inspector_panel into panels/inspector module (#-)
- 2026-03-16: refactor: Decompose `analyzer_v2.rs` into submodules (#-)
- 2026-03-16: feat: Use standard buttons in empty file states (#1198)
- 2026-03-16: fix: [CRITICAL] Fix DoS vulnerability during project export (#1195)
- 2026-03-16: chore: Repository Cleanup (#1194)
- 2026-03-16: chore: Update application branding to MapFlow (#1189)
- 2026-03-15: chore: Translate messages to German in post-merge workflow (#-)
- 2026-03-15: chore: Enable Node.js 24 for JavaScript actions in CI/CD (#-)
- 2026-03-15: fix: Fix echo statement in CI/CD validation workflow (#-)
- 2026-03-15: fix: Fix formatting in CodeQL Analysis category (#-)
- 2026-03-15: fix: Fix environment variable assignment in CI/CD workflow (#-)
- 2026-03-15: chore: Update CI/CD workflow for self-hosted post-merge (#-)
- 2026-03-15: chore: Update auto-merge workflow for PRs (#-)
- 2026-03-15: chore: Update security scan workflow for consistency and improvements (#-)
- 2026-03-15: refactor: Refactor CI/CD workflow for better efficiency (#-)
- 2026-03-15: refactor: Refactor auto-merge checks and workflows (#-)
- 2026-03-15: test: Add missing tests for logging module (#1153)
- 2026-03-15: fix: Resolve YAML syntax errors and cleanup validation workflow (#-)
- 2026-03-15: fix: Add concurrency groups and remove path filters to prevent runner queue saturation (#-)
- 2026-03-15: refactor: Decompose mapmap-ui lib.rs (#1154)
- 2026-03-15: feat: Implement media player orchestration, video playback, and texture management. (#-)
- 2026-03-15: fix: Ensure Node.js 24 opt-in is applied to all steps in post-merge workflow (#-)
- 2026-03-15: fix: Resolve case-sensitivity conflict by removing .Jules/ entry (#-)
- 2026-03-15: feat: MF-074-TIMELINE-SCENE-NESTING: Core/UI: Szenen-Gruppierung und Nesting-Logik fuer Module (#1140)
- 2026-03-15: refactor: Refactor CI workflow and update dependencies (#-)
- 2026-03-15: refactor: Refactor CI workflow for security scan (#-)
- 2026-03-14: feat: Universal Trigger Router (MIDI/OSC) for Trackline-Modus (#1139)
- 2026-03-14: chore: Delete ROADMAP.md (#-)
- 2026-03-14: fix: Add libxkbcommon-x11-0 system dependency for Linux builds (#-)
- 2026-03-14: chore: Change PR number input requirement to optional (#-)
- 2026-03-14: fix: Resolve case-sensitivity conflict between .jules and .Jules (#-)
- 2026-03-14: docs: Restore ROADMAP.md (Mandate: Tasks must not be removed) (#-)
- 2026-03-14: fix: Report commit status to the correct PR head SHA (#-)
- 2026-03-14: fix: Fix merge conflict markers in .geminiignore and cleanup ignore patterns (#-)
- 2026-03-14: docs: Remove old `ROADMAP.md` and add new `.jules/mary-styleux.md` documentation. (#-)
- 2026-03-14: fix: Ensure commit status is reported for push and pull_request events (#-)
- 2026-03-14: feat: Implement Trackline Mode for marker-to-marker playback (#1137)
- 2026-03-14: feat: MF-070-TIMELINE-ARCH-MULTI-TRACK: Architektur: Multi-Track Datenmodell (Module vs. Parameter Tracks) (#1135)
- 2026-03-14: feat: __MF-SubI_Echter MapFlow-Automationsmodus auf run_app-Basis (#1134)
- 2026-03-14: feat: __MF-SubI_Appnahe Referenzszenarien fuer Core-Features (#1125)
- 2026-03-14: feat: __MF-SubI_Deterministische Screenshot- und Artefakt-Pipeline (#1127)
- 2026-03-14: feat: Expand MCP and OSC implementation for full remote controllability (#1123)
- 2026-03-14: docs: Define new timeline revolution concept and add roadmap tasks (#1009)
- 2026-03-14: feat: Consistent Media/Mask picker buttons in empty states (#1122)
- 2026-03-14: feat: Consistent styling for empty states (#1121)
- 2026-03-14: fix: Fix DoS vulnerability in project export (#1120)
- 2026-03-14: perf: Optimize TexturePool view fast path (#1119)
- 2026-03-14: feat: Introduce `mapmap` crate with module evaluation and UI layout, establish visual capture tests, and implement CI/CD workflows. (#-)
- 2026-03-14: fix: Fix broken link in GitHub Workflows README (#1110)
- 2026-03-14: chore: Repository Cleanup Check - Clean State (#1109)
- 2026-03-14: feat: Implement `winit::application::ApplicationHandler` for the core application event loop and lifecycle management. (#-)
- 2026-03-14: feat: Add module graph evaluation, initial application framework, and module canvas UI components. (#-)
- 2026-03-14: feat: Modernisiere UI: responsive Panels, Preview‑Sidebar, Theme, Font‑Scale & Toolbar‑Metriken (#1099)
- 2026-03-14: feat: Enable Enter key to trigger cue jump (#1108)
- 2026-03-14: feat: Improve empty state visibility via text muting (#1107)
- 2026-03-14: perf: Eliminate unnecessary f32 clone in evaluation loop (#1106)
- 2026-03-14: test: Add comprehensive unit tests for mapmap-core layer structs and manager (#1105)
- 2026-03-14: chore: Verify clean repository state (#1104)
- 2026-03-14: docs: Update mapmap crate README to point to new semantic documentation structure (#1103)
- 2026-03-14: feat: Background generation of thumbnails in Media Browser (#1100)
- 2026-03-14: refactor: Refactor MCP Server by separating tool definitions and handlers (#1101)
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
