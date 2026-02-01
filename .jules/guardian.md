# Guardian's Journal 🧪

## 2024-10-24 - Initial Setup
**Insight:** Established the Guardian role to improve test coverage and reliability.
**Action:** Created this journal to track critical testing insights.

## 2024-10-24 - TriggerSystem Coverage
**Insight:** `TriggerSystem` in `mapmap-core` was a critical logic component with zero unit tests. It relies heavily on `ModuleManager` and `AudioTriggerData` integration.
**Action:** Implemented integration tests using `ModuleManager` to simulate module configuration and `AudioTriggerData` to simulate input. This pattern effectively tests the interaction without needing full app state.

## 2024-10-25 - BPM Estimation Simulation
**Insight:** Verified that simulating audio buffers (chunked sine waves) effectively tests complex DSP logic like BPM detection without needing real audio files or hardware.
**Action:** Use synthesized audio chunks for future audio analyzer tests to ensure deterministic behavior.

## 2024-10-25 - TriggerSystem Unit Tests
**Insight:** `TriggerSystem` logic (thresholds, band mapping) is complex enough to warrant dedicated unit tests within the module, not just integration tests.
**Action:** Added comprehensive unit test suite to `crates/mapmap-core/src/trigger_system.rs` covering all trigger types (Band, RMS, Peak, Beat).
