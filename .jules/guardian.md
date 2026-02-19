# Guardian's Journal 🧪

## 2026-02-08 - Audio Data Sanitization

**Erkenntnis:** Rohe Audio-Buffer enthalten oft NaNs oder Infinities von Treibern oder leeren Streams. Das Propagieren dieser Werte in FFT- oder RMS-Berechnungen vergiftet die gesamte Analyse-Pipeline, was zu NaNs in Uniform Buffern führt, die GPU-Treiber zum Absturz bringen oder schwarze Bildschirme verursachen können.

**Aktion:** Audio-Input-Buffer immer am Eingangspunkt sanitizen (nicht-finite Werte durch 0.0 ersetzen). `test_sanitization_of_bad_input` wurde zu `AudioAnalyzerV2` hinzugefügt, um dies strikt durchzusetzen.

## 2024-10-24 - Initial Setup

**Insight:** Established the Guardian role to improve test coverage and reliability.
**Action:** Created this journal to track critical testing insights.

## 2024-05-24 - [Assignment Module Coverage]

**Erkenntnis:** The `assignment` module (ControlSource, ControlTarget) was completely devoid of tests despite being critical for MIDI/OSC routing.
**Aktion:** Added `tests/assignment_tests.rs` with full CRUD and serialization coverage. Added to weekly check list.

## 2024-05-24 - [State Defaults]

**Erkenntnis:** `AppState` default values for deep fields (like `EffectParameterAnimator`) were not verified, risking hidden initialization bugs.
**Aktion:** Added `test_app_state_deep_defaults` to enforce correct initialization state.

## 2024-05-24 - [State Persistence]

**Erkenntnis:** `AppState` serialization tests were only checking a subset of fields, risking silent data loss for new features. Deep checking of
default states revealed nested managers must also be verified. `dirty` flag exclusion must be explicitly tested to avoid false positive
"unsaved changes" warnings.
**Aktion:** Refactored `test_app_state_serialization_roundtrip` to use `assert_eq!` on the full struct (via `PartialEq`). Added specific test
`test_dirty_flag_excluded` to guarantee transient flags are not persisted.

## 2024-05-25 - [MIDI Parsing]
**Erkenntnis:** `MidiMessage` parsing logic for PitchBend (14-bit reconstruction) and system messages (Start/Stop) was implemented but untested.
This created a risk for hardware controllers relying on high-resolution input or transport controls.
**Aktion:** Implemented `test_midi_message_parsing_extended` covering full 14-bit Pitch Bend reconstruction and all system realtime messages
to ensure reliable hardware integration.

## 2024-05-26 - [Trigger System Integration]

**Erkenntnis:** `TriggerSystem` integration logic was untested. While `TriggerConfig` logic was tested, the actual mapping of Audio FFT bands
to socket indices (0-8, 9-11) in the `update` loop was unverified, leaving a gap in ensuring audio reactivity works end-to-end.
**Aktion:** Restored `tests/trigger_system_tests.rs` with mocks for `ModuleManager` and `AudioTriggerData`, ensuring every frequency band
and volume trigger fires correctly.

## 2024-10-24 - TriggerSystem Coverage

**Insight:** `TriggerSystem` in `mapmap-core` was a critical logic component with zero unit tests. It relies heavily on `ModuleManager`
and `AudioTriggerData` integration.
**Action:** Implemented integration tests using `ModuleManager` to simulate module configuration and `AudioTriggerData` to simulate input.
This pattern effectively tests the interaction without needing full app state.

## 2024-10-25 - BPM Estimation Simulation

**Insight:** Verified that simulating audio buffers (chunked sine waves) effectively tests complex DSP logic like BPM detection without
needing real audio files or hardware.
**Action:** Use synthesized audio chunks for future audio analyzer tests to ensure deterministic behavior.

## 2024-10-26 - Part Socket Verification
**Insight:** Iterating over all `PartType` variants in a test to verify socket generation catches "orphan" parts that might be added to
the enum but lack input/output definitions, which leads to broken UI states.
**Action:** Apply this "enum iteration" pattern to other factory-like methods to ensure complete coverage of new variants.

## 2024-10-26 - Audio Buffer Resizing

**Insight:** Testing `update_config` in audio analyzers is critical because mismatched buffer sizes (e.g., between FFT and input buffers)
are a common source of runtime panics or silent failures when users change settings.
**Action:** Always include a "reconfiguration" test case for stateful processing components like audio analyzers or render pipelines.

## 2024-10-27 - [Audio Pipeline Robustness]

**Insight:** Testing threaded audio pipelines requires robust "polling atomics" (loops with `yield_now`) rather than fixed sleeps to avoid
flakiness and ensure valid data flow verification (e.g. `rms_volume > 0`). Explicit struct initialization in tests prevents Clippy
warnings and future-proofs against default value assumptions.
**Action:** Implemented `audio_pipeline_tests.rs` with data flow, stats, and dropped sample verification. Refactored `trigger_system_tests.rs`
to use explicit `AudioTriggerData` initialization.

## 2024-05-27 - [Critical: Audio Sanitization & Zero-Size Transforms]

**Erkenntnis:** `AudioAnalyzerV2` allowed NaN/Inf values to propagate, potentially destabilizing the entire audio pipeline. `ResizeMode`
calculations for zero-sized layers could result in division by zero (Inf).
**Aktion:** Implemented input sanitization in `AudioAnalyzerV2::process_samples` and zero-size checks in `ResizeMode::calculate_transform`.
Added regression tests `test_resilience_to_bad_input` and `test_resize_mode_zero_size`.

## 2024-05-28 - [Trigger System Correctness]

**Erkenntnis:** `TriggerSystem` (legacy/alternative implementation) completely ignored `inverted_outputs` configuration, meaning logical inversions (active on low signal) were never processed, unlike `ModuleEvaluator`.
**Aktion:** Fixed `TriggerSystem` to respect inversion flags and added `test_inverted_outputs_logic` to catch regressions.
