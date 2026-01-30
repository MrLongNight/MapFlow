# Guardian's Journal 🧪

## 2024-10-24 - Initial Setup
**Insight:** Established the Guardian role to improve test coverage and reliability.
**Action:** Created this journal to track critical testing insights.

<<<<<<< HEAD
## 2024-05-24 - [Assignment Module Coverage]
**Erkenntnis:** The `assignment` module (ControlSource, ControlTarget) was completely devoid of tests despite being critical for MIDI/OSC routing.
**Aktion:** Added `tests/assignment_tests.rs` with full CRUD and serialization coverage. Added to weekly check list.

## 2024-05-24 - [State Defaults]
**Erkenntnis:** `AppState` default values for deep fields (like `EffectParameterAnimator`) were not verified, risking hidden initialization bugs.
**Aktion:** Added `test_app_state_deep_defaults` to enforce correct initialization state.
## 2024-05-24 - [State Persistence]
**Erkenntnis:** `AppState` serialization tests were only checking a subset of fields, risking silent data loss for new features. Deep checking of default states revealed nested managers must also be verified. `dirty` flag exclusion must be explicitly tested to avoid false positive "unsaved changes" warnings.
**Aktion:** Refactored `test_app_state_serialization_roundtrip` to use `assert_eq!` on the full struct (via `PartialEq`). Added specific test `test_dirty_flag_excluded` to guarantee transient flags are not persisted.

## 2024-05-25 - [MIDI Parsing]
**Erkenntnis:** `MidiMessage` parsing logic for PitchBend (14-bit reconstruction) and system messages (Start/Stop) was implemented but untested. This created a risk for hardware controllers relying on high-resolution input or transport controls.
**Aktion:** Implemented `test_midi_message_parsing_extended` covering full 14-bit Pitch Bend reconstruction and all system realtime messages to ensure reliable hardware integration.

## 2024-05-26 - [Trigger System Integration]
**Erkenntnis:** `TriggerSystem` integration logic was untested. While `TriggerConfig` logic was tested, the actual mapping of Audio FFT bands to socket indices (0-8, 9-11) in the `update` loop was unverified, leaving a gap in ensuring audio reactivity works end-to-end.
**Aktion:** Restored `tests/trigger_system_tests.rs` with mocks for `ModuleManager` and `AudioTriggerData`, ensuring every frequency band and volume trigger fires correctly.
=======
## 2024-10-24 - TriggerSystem Coverage
**Insight:** `TriggerSystem` in `mapmap-core` was a critical logic component with zero unit tests. It relies heavily on `ModuleManager` and `AudioTriggerData` integration.
**Action:** Implemented integration tests using `ModuleManager` to simulate module configuration and `AudioTriggerData` to simulate input. This pattern effectively tests the interaction without needing full app state.
>>>>>>> main
