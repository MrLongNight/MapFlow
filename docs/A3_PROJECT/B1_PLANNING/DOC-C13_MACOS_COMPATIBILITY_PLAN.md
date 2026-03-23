# DOC-C13: macOS Compatibility Plan

## Status
- Status: Proposed
- Priority: High
- Roadmap task: `MF-061-MACOS-COMPATIBILITY`
- Baseline date: March 10, 2026
- Delivery strategy:
  - First milestone: internal macOS build
  - Second milestone: public macOS beta
  - Third milestone: production-ready macOS release

## 1. Summary

MapFlow can be made macOS-compatible, but the current repository is not yet ready to offer macOS as a supported release platform.

The core architecture is suitable for a macOS port:
- Rust workspace
- `wgpu` / `winit` / `egui`
- `cpal`
- `ffmpeg-next`

The real gap is platform hardening:
- no macOS CI job
- no macOS release artifact pipeline
- no `.app` bundle / signing / notarization flow
- `Syphon` is still a stub
- `VirtualCamera` is still a stub
- `VideoToolbox` exists in the decoder model but is not fully implemented as an actual macOS acceleration path

Recommendation:
- ship a macOS beta first
- do not block the first beta on Syphon or Virtual Camera
- add macOS-native interop only after the base app is stable

## 2. Scope

### MVP for macOS beta
- app launches on Apple Silicon
- app launches on Intel if Intel support remains in scope
- UI and rendering run on Metal through `wgpu`
- project open/save works
- multi-window output works
- FFmpeg media playback works at least with software decode
- audio input is either stable or correctly feature-gated
- CI can build the app on macOS

### Not required for the first beta
- Syphon parity
- CoreMediaIO virtual camera
- full installer hardening on day one
- feature parity for every platform-specific integration

### Required for production-ready macOS support
- signed artifact
- notarized artifact
- documented install path
- regression-tested behavior on real Macs
- stable media and audio path

## 3. Workstreams

### WS1: Build and dependency compatibility
Tasks:
- validate workspace build on macOS
- review shared `winit` feature flags
- review `mapflow-bevy` feature flags
- validate `rfd`, `cpal`, `midir`, `ffmpeg-next`, and `libmpv2` behavior on macOS
- feature-gate or disable unfinished platform paths

### WS2: Runtime stabilization
Tasks:
- verify startup and render loop on Metal
- verify window management, resize, redraw, and multi-monitor behavior
- verify project loading, previews, and output windows
- verify file dialogs and path handling
- add clear runtime errors for unsupported macOS features

### WS3: Media and audio
Tasks:
- validate FFmpeg discovery and runtime linking on macOS
- use software decode as the first stable baseline
- implement or explicitly defer `VideoToolbox`
- validate `cpal` device enumeration and stream startup
- add fallback behavior for audio init failures

### WS4: CI, packaging, and release
Tasks:
- add `macos-latest` validation job
- add macOS release artifact job
- create `.app` bundle layout
- add `Info.plist`
- define ZIP vs DMG strategy
- add codesign and notarization workflow

### WS5: Optional macOS-native interop
Tasks:
- add Syphon to the real app/runtime model
- implement actual Syphon sender/receiver support
- implement Virtual Camera only if product value justifies maintenance cost

## 4. Proposed Phases

### Phase 0: Discovery
Estimate: 2-3 days

Deliverables:
- Apple Silicon validation machine
- Intel validation machine if needed
- final macOS support policy
- feature matrix for macOS

### Phase 1: Build bootstrap
Estimate: 3-5 days

Deliverables:
- `cargo build -p mapflow` works on macOS
- compile blockers documented or fixed
- unfinished platform features gated

### Phase 2: Core beta stabilization
Estimate: 5-8 days

Deliverables:
- launchable app on real Macs
- stable core editing workflow
- stable file dialog and multi-window behavior
- bounded media and audio failure modes

### Phase 3: CI and packaging
Estimate: 3-5 days

Deliverables:
- macOS CI validation
- internal macOS artifact
- updated build and install docs

### Phase 4: Release hardening
Estimate: 4-8 days

Deliverables:
- signing
- notarization
- release checklist
- support notes for macOS users

### Phase 5: Advanced parity
Estimate: 2-6 weeks

Deliverables:
- real Syphon support
- optional virtual camera support
- better media acceleration path if required

## 5. Concrete Backlog

### Immediate backlog
- [ ] Add macOS CI job to `.github/workflows/CICD-DevFlow_Job01_Validation.yml`
- [ ] Add macOS release artifact job to `.github/workflows/CICD-MainFlow_Job03_Release.yml`
- [ ] Validate `Cargo.toml` shared `winit` flags on macOS
- [ ] Validate `crates/mapflow-bevy/Cargo.toml` feature set on macOS
- [ ] Verify `cargo build --release -p mapflow` on Apple Silicon
- [ ] Verify `cargo build --release -p mapflow` on Intel macOS

### Runtime backlog
- [ ] Test startup and render loop on Metal
- [ ] Test file dialogs and media import
- [ ] Test multi-window projector flow
- [ ] Add user-facing fallback when unsupported macOS features are accessed

### Media backlog
- [ ] Validate FFmpeg runtime strategy on macOS
- [ ] Decide whether software decode ships as the first baseline
- [ ] Implement or defer `VideoToolbox`
- [ ] Validate audio input path on macOS

### Release backlog
- [ ] Create `.app` bundle metadata
- [ ] Add `Info.plist`
- [ ] Define signing certificate requirements
- [ ] Implement notarization flow
- [ ] Produce first internal macOS artifact

### Optional parity backlog
- [ ] Add Syphon to app-level source/output model
- [ ] Implement real `mapflow-io` Syphon support
- [ ] Expose Syphon in UI only after runtime support is real
- [ ] Re-evaluate Virtual Camera after beta feedback

## 6. Risks

### High risk
- FFmpeg distribution and linking on macOS
- signing and notarization complexity
- audio permissions and device behavior across different Macs

### Medium risk
- Apple Silicon plus Intel support doubles validation effort
- `VideoToolbox` may require extra FFmpeg and pixel-format work
- third-party crates may compile but still have runtime edge cases

### Lower risk
- core rendering via `wgpu`/Metal is likely less risky than packaging and interop

## 7. Success Criteria

MapFlow counts as macOS beta-ready when:
- CI builds the app on macOS
- app launches on Apple Silicon
- the main editing UI is usable
- project open/save works
- at least one media playback path works reliably
- audio is stable or intentionally disabled with clear UX
- internal testers can run a packaged artifact

MapFlow counts as production-ready for macOS when:
- a signed and notarized artifact exists
- install steps are documented
- core workflows pass regression testing on real Macs
- unsupported features are clearly documented

## 8. Recommended Execution Order

Recommended order for implementation:
1. `MF-062-MACOS-BUILD-BOOTSTRAP`
2. `MF-066-MACOS-CI-VALIDATION`
3. `MF-063-MACOS-RUNTIME-STABILIZATION`
4. `MF-064-MACOS-MEDIA-FFMPEG-PATH`
5. `MF-065-MACOS-AUDIO-VALIDATION`
6. `MF-067-MACOS-PACKAGING-NOTARIZATION`
7. `MF-068-MACOS-NATIVE-INTEROP`

Rationale:
- `MF-062` removes the biggest compile and configuration blockers first.
- `MF-066` should follow immediately so every later macOS fix gets automated feedback.
- `MF-063` validates whether the app is actually usable after it compiles.
- `MF-064` and `MF-065` stabilize media and audio once the base app runs.
- `MF-067` should start only after the beta path is technically stable.
- `MF-068` is intentionally last because it is optional for the first macOS beta.

## 9. Subtask Briefs for Agent Delegation

### MF-062-MACOS-BUILD-BOOTSTRAP
- Goal: make the workspace build cleanly on macOS with a clearly defined feature set.
- Primary files:
  - `Cargo.toml`
  - `crates/mapflow/Cargo.toml`
  - `crates/mapflow-bevy/Cargo.toml`
  - `crates/mapflow-ui/Cargo.toml`
- Required outputs:
  - documented macOS build command
  - compile blockers removed or explicitly feature-gated
  - macOS build notes added where needed
- Definition of done:
  - `cargo build -p mapflow` is expected to work on macOS
  - no unfinished macOS blockers remain hidden behind default settings

### MF-066-MACOS-CI-VALIDATION
- Goal: add automated macOS validation before deeper platform work continues.
- Primary files:
  - `.github/workflows/CICD-DevFlow_Job01_Validation.yml`
  - optional supporting CI docs
- Required outputs:
  - macOS validation job
  - documented feature set used by the CI job
  - clear failure logs for macOS-specific problems
- Definition of done:
  - repository has a macOS validation path
  - later tasks can rely on CI feedback instead of local-only testing

### MF-063-MACOS-RUNTIME-STABILIZATION
- Goal: make the compiled app launch and behave predictably on macOS.
- Primary files:
  - `crates/mapflow/src/main.rs`
  - `crates/mapflow/src/window_manager.rs`
  - `crates/mapflow/src/orchestration/outputs.rs`
  - platform-relevant UI/runtime files as needed
- Required outputs:
  - stable startup
  - stable window creation and redraw
  - known unsupported runtime paths guarded with clear errors
- Definition of done:
  - app launches on at least one real Mac
  - core UI and output windows are usable without critical crashes

### MF-064-MACOS-MEDIA-FFMPEG-PATH
- Goal: establish a stable macOS media path even if it starts with software decode only.
- Primary files:
  - `crates/mapflow-media/Cargo.toml`
  - `crates/mapflow-media/src/decoder.rs`
  - build and install docs if dependency handling changes
- Required outputs:
  - defined FFmpeg strategy for macOS
  - reliable fallback to software decode
  - explicit decision on `VideoToolbox` now vs later
- Definition of done:
  - media playback on macOS is stable enough for beta testing

### MF-065-MACOS-AUDIO-VALIDATION
- Goal: validate whether audio should ship enabled, gated, or deferred on macOS.
- Primary files:
  - `crates/mapflow-core/src/audio/`
  - `crates/mapflow-ui/Cargo.toml`
  - relevant docs around build and runtime support
- Required outputs:
  - clear macOS audio support decision
  - runtime fallback if audio init fails
  - updated docs consistent with actual behavior
- Definition of done:
  - no ambiguous "maybe supported" state remains for macOS audio

### MF-067-MACOS-PACKAGING-NOTARIZATION
- Goal: turn a working beta build into a deliverable release artifact.
- Primary files:
  - release workflows
  - macOS bundle metadata
  - install/build docs
- Required outputs:
  - `.app` bundle strategy
  - signing and notarization checklist or implementation
  - internal or public macOS artifact path
- Definition of done:
  - macOS artifact can be distributed without ad hoc manual steps

### MF-068-MACOS-NATIVE-INTEROP
- Goal: add optional macOS-native interop only after the base app is already stable.
- Primary files:
  - `crates/mapflow-io/src/syphon/mod.rs`
  - `crates/mapflow-io/src/virtual_camera/mod.rs`
  - app-level source/output model and UI wiring
- Required outputs:
  - explicit scope decision for Syphon and Virtual Camera
  - no UI exposure for features that remain stubs
- Definition of done:
  - macOS-native interop is either real or clearly deferred

## 10. Roadmap Link

- Roadmap task: `MF-061-MACOS-COMPATIBILITY`
- See GitHub Project Issues for status tracking

### Subtasks
- `MF-062-MACOS-BUILD-BOOTSTRAP`
- `MF-063-MACOS-RUNTIME-STABILIZATION`
- `MF-064-MACOS-MEDIA-FFMPEG-PATH`
- `MF-065-MACOS-AUDIO-VALIDATION`
- `MF-066-MACOS-CI-VALIDATION`
- `MF-067-MACOS-PACKAGING-NOTARIZATION`
- `MF-068-MACOS-NATIVE-INTEROP`
