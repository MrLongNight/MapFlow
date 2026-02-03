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

## 2024-10-26 - Part Socket Verification
**Insight:** Iterating over all `PartType` variants in a test to verify socket generation catches "orphan" parts that might be added to the enum but lack input/output definitions, which leads to broken UI states.
**Action:** Apply this "enum iteration" pattern to other factory-like methods to ensure complete coverage of new variants.

## 2024-10-26 - Audio Buffer Resizing
**Insight:** Testing `update_config` in audio analyzers is critical because mismatched buffer sizes (e.g., between FFT and input buffers) are a common source of runtime panics or silent failures when users change settings.
**Action:** Always include a "reconfiguration" test case for stateful processing components like audio analyzers or render pipelines.

## 2024-10-27 - Disabled Pipeline Logic
**Insight:** `mapmap-media/src/pipeline.rs` contains critical threading logic for video decoding but is currently disabled (commented out in `lib.rs`) and contains compilation errors in tests.
**Action:** Flagged `pipeline.rs` for future repair. For now, focus testing on enabled components.

## 2024-10-27 - NDI Feature Stub Verification
**Insight:** Feature-gated modules like NDI often lack tests for their disabled state (stubs). This can lead to runtime errors or confusion when features are missing.
**Action:** Added explicit tests for NDI stubs to verify they return correct errors when the feature is disabled, ensuring graceful degradation.
