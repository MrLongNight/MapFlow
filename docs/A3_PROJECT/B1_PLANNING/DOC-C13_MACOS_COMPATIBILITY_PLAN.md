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
- review `mapmap-bevy` feature flags
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
- `cargo build -p mapmap` works on macOS
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
- [ ] Validate `crates/mapmap-bevy/Cargo.toml` feature set on macOS
- [ ] Verify `cargo build --release -p mapmap` on Apple Silicon
- [ ] Verify `cargo build --release -p mapmap` on Intel macOS

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
- [ ] Implement real `mapmap-io` Syphon support
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

## 8. Roadmap Link

- Roadmap task: `MF-061-MACOS-COMPATIBILITY`
- See root `ROADMAP.md` for status tracking

### Subtasks
- `MF-062-MACOS-BUILD-BOOTSTRAP`
- `MF-063-MACOS-RUNTIME-STABILIZATION`
- `MF-064-MACOS-MEDIA-FFMPEG-PATH`
- `MF-065-MACOS-AUDIO-VALIDATION`
- `MF-066-MACOS-CI-VALIDATION`
- `MF-067-MACOS-PACKAGING-NOTARIZATION`
- `MF-068-MACOS-NATIVE-INTEROP`
