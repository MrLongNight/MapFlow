
## 2026-02-08 - MIDI Learn Timeout State

**Erkenntnis:** State machines like `MidiLearnState` rely heavily on timeouts for user experience, but logic checks like `check_timeout` are often assumed correct without verifying the state transition actually happens.
**Aktion:** Explicitly test state machine transitions involving time using `std::thread::sleep` with short durations in unit tests to guarantee UI states (like "Timed Out") are reachable.

## 2026-02-08 - Audio Data Sanitization

**Erkenntnis:** Rohe Audio-Buffer enthalten oft NaNs oder Infinities von Treibern oder leeren Streams. Das Propagieren dieser Werte in FFT- oder RMS-Berechnungen vergiftet die gesamte Analyse-Pipeline, was zu NaNs in Uniform Buffern führt, die GPU-Treiber zum Absturz bringen oder schwarze Bildschirme verursachen können.
**Aktion:** Audio-Input-Buffer immer am Eingangspunkt sanitizen (nicht-finite Werte durch 0.0 ersetzen). `test_sanitization_of_bad_input` wurde zu `AudioAnalyzerV2` hinzugefügt, um dies strikt durchzusetzen.

## 2025-05-24 - Initial Insights

**Erkenntnis:** `TriggerConfig::apply` in `mapmap-core` creates a new `rand::rng()` instance on every call for `RandomInRange` mode. This is likely a performance bottleneck in hot paths (e.g., audio triggers).
**Aktion:** Consider refactoring `TriggerConfig::apply` to accept a mutable reference to an RNG or use `thread_rng()` more efficiently. For now, testing acknowledges this behavior.

**Erkenntnis:** `VideoFrame` in `mapmap-io` uses `FrameData::Gpu(Arc<wgpu::Texture>)`, making it difficult to unit test without a GPU context.
**Aktion:** Use `#[ignore]` for GPU-dependent tests or separate logic from resource holding where possible. Ensure CPU fallback paths are robustly tested.

**Erkenntnis:** `MidiMappingKey` implements `From<&MidiMessage>` returning `Option<MidiMappingKey>`. This is unconventional (vs `TryFrom`) but enables ergonomic `let key: Option<_> = msg.into()` in event loops.
**Aktion:** Document this pattern in `MidiMappingKey` docs to avoid confusion during future refactoring.

## 2025-02-18 - [Critical Test Gaps]

**Erkenntnis:** Critical socket generation logic in `module.rs` was relying on untested `match` arms, particularly for `Bevy` source types and `Hue` integration. `Layer` transformation logic also lacked explicit verification of delegate calls to `Transform`.

**Aktion:** Implemented comprehensive socket verification tests (`test_bevy_source_sockets`, `test_hue_sockets`) and transform delegation tests (`test_layer_transform_delegation`). Future PRs should strictly enforce `socket_type` verification for any new `ModulePartType`.

**Erkenntnis:** `ControlValue::validate` uses `std::path::Path::components()` to check for `ParentDir` (`..`) traversal attempts in string values.
**Aktion:** Verify this pattern is consistently applied across all user-input paths to prevent directory traversal attacks.

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

## 2026-06-25 - [TriggerSystem Memory Leak & Consistency]

**Erkenntnis:** `TriggerSystem` akkumulierte Trigger-Status für gelöschte Parts (Speicherleck) und initialisierte RNG in der Hot-Loop. Außerdem gab es keine Garantie, dass `AudioFFT` Socket-Indizes in `TriggerSystem` mit `module.rs` übereinstimmen.
**Aktion:** Garbage Collection und RNG-Optimierung in `TriggerSystem::update` implementiert. `test_trigger_system_garbage_collection` und `test_audio_fft_socket_consistency` hinzugefügt, um Lecks und Inkonsistenzen zu verhindern.
## 2025-03-01 - Testabdeckung im mapmap-core/layer verbessert
 **Erkenntnis:** Der mapmap-core layer manager (manager.rs) hatte keine Testabdeckung. Dieser Code ist entscheidend für das Handling von Layer-Sichtbarkeit, Grouping und Z-Ordering und muss verlässlich funktionieren.
 **Aktion:** Umfangreiche Tests für LayerManager in layer_tests.rs hinzugefügt, inklusive CRUD, Grouping, Z-Order, Visible-Filter und CoW-Klonverhalten. In Zukunft bei neuem Code immer den zugehörigen Test-File prüfen, insbesondere bei zentralen Managern in mapmap-core.

## 2026-06-25 - [Trigger Logic Inconsistency]

**Erkenntnis:** `TriggerSystem` und `ModuleEvaluator` implementierten unterschiedliche Logik für `Fixed` und `Random` Trigger.
`ModuleEvaluator` implementierte `Random` Trigger als statenloses Rauschen (Ignorieren von Intervallen), während `TriggerSystem` Intervalle nutzte, aber `probability` ignorierte.
Zudem ignorierte `TriggerSystem` den `offset_ms` Parameter für `Fixed` Trigger.
**Aktion:** `TriggerSystem` wurde korrigiert, um `offset_ms` (als initiale Verzögerung) und `probability` (als Filter bei Intervall-Ende) zu respektieren.
Die Inkonsistenz im `ModuleEvaluator` bleibt bestehen, da dieser statenlos ist und keine Intervalle korrekt abbilden kann. Dies sollte bei zukünftigen Refactorings adressiert werden.

## 2024-05-24 - Split Logic in ModuleEvaluator
**Erkenntnis:** The application of `TriggerTarget` logic is split between two separate methods in `ModuleEvaluator`. `evaluate()` handles `SourceCommand` modification (for Bevy/Media inputs), while `trace_chain_into()` handles `RenderOp` modification (for visual properties like Opacity/Scale). This separation increases the risk of regression if one path is updated without the other.
**Aktion:** Ensure both paths are explicitly tested for `TriggerTarget` application. Future refactoring should consider unifying this logic.

## 2024-03-04 - Ungetestete ModuleManager Funktion
**Erkenntnis:** Die `ModuleManager` Struktur in `mapmap-core/src/module/manager.rs` war komplett ungetestet. Dies ist kritische Core-Logik.
**Aktion:** Unit Tests für die Modul-Erstellung, -Löschung, -Umbenennung und -Duplizierung hinzugefügt, inklusive Behandlung von Namenskonflikten.
## 2026-03-08 - Zusammensetzung Standardwerte und Grenzen
**Was:** Die `Composition` Struktur und ihre Initialisierung in `crates/mapmap-core/src/layer/composition.rs` wurde intensiv durch Unit-Tests abgedeckt.
**Warum:** Um sicherzustellen, dass die Boundary Conditions, Master Speed/Opacity Limits (0.1 - 10.0, 0.0 - 1.0) und Default-Werte nicht regressieren.
**Abdeckung:** Erreicht vollständige Testabdeckung der Initialisierungslogik.
**Neue Tests:** `test_composition_default_values`, `test_composition_new_initialization`, `test_composition_set_master_opacity_bounds`, `test_composition_set_master_speed_bounds`, `test_composition_with_description_builder`.
**Geänderte Tests:** Keine.


## 2025-03-11 - Testabdeckung im mapmap-core/layer verbessert
**Erkenntnis:** Der mapmap-core layer manager (manager.rs) hatte keine direkte Testabdeckung für diverse Extrem- oder Fehlerfälle (z.B. ID out-of-bounds, `remove_layer` von nicht existierenden IDs).
**Aktion:** Umfangreiche Tests für LayerManager direkt in `crates/mapmap-core/src/layer/manager.rs` hinzugefügt, insbesondere für Edge-Cases und extrem-Szenarien (`move_layer_up_down_extremes`, `duplicate_nonexistent_layer`). In Zukunft bei neuem Code immer den zugehörigen Test-File prüfen, insbesondere bei zentralen Managern in mapmap-core.

## 2026-03-15 - [AudioAnalyzerV2 Test Coverage Improvement]

**Erkenntnis:** The `AudioAnalyzerV2` module had gaps in its test coverage, particularly in edge cases of the `calculate_bpm` and `try_receive` functions. The `calculate_bpm` lacked tests for clamped ranges, a zero average interval calculation (when dividing), and handling an empty valid interval collection (caused by few samples or bad input timestamps).
**Aktion:** Handled these explicitly by injecting manual timestamps to bypass earlier signal processing logic and hit only the mathematical branches in `calculate_bpm`. This isolates tests and exposes pure logic edge cases in `audio/analyzer_v2.rs`.
## 2026-03-17 - Modul-Umbenennung erlaubt Identität
**Erkenntnis:** Die Funktion `rename_module` im `ModuleManager` erlaubt das Umbenennen eines Moduls auf seinen eigenen, bestehenden Namen und gibt dabei `true` zurück, anstatt fehlzuschlagen oder eine Duplikats-Prüfung auszulösen.
**Aktion:** Dieses Muster sollte in UI-Test-Szenarien berücksichtigt werden, bei denen Benutzer den Umbenennen-Dialog bestätigen, ohne den Namen zu ändern (Silent Success vs Error).

## 2024-05-24 - [State Clone & Equality Coverage]
**Erkenntnis:** The `AppState` and `AppSettings` structures lacked basic Clone and PartialEq test coverage. This is essential for ensuring that deep fields are correctly duplicated and evaluated during CoW (Copy-on-Write) and UI diffing operations.
**Aktion:** Implemented `test_app_settings_clone_equality`, `test_app_settings_partial_eq`, and `test_app_state_clone_equality` in `crates/mapmap-core/src/state.rs` to secure state management semantics.

## 2024-03-24 - Module Connection Validation Tests Update
**Erkenntnis:** Tests asserting `module.add_connection` failed silently because it properly validates parts/sockets internally before adding. Also `update_part_sockets` relies on `repair_graph` to do connection cleanup.
**Aktion:** Use `module.connections.push()` in tests to explicitly bypass `add_connection` validation when asserting connection removal/cleanup logic for invalid states, and call `repair_graph()` when expecting `update_part_sockets` to clean up connections.

## 2026-03-21 - Diagnostic Coverage Improvement
**Erkenntnis:** The `check_module_integrity` function contained untested logic paths regarding validation of connected sockets (`from_socket` and `to_socket`). This omission left an undetected gap for topology error handling.
**Aktion:** I introduced dedicated Unit Tests (`test_check_module_integrity_invalid_from_socket`, `test_check_module_integrity_invalid_to_socket`, and `test_check_module_integrity_connected_output_no_warning`) to guarantee consistent and verified issue generation whenever these topology constraints are violated.

## 2026-03-23 - [Test coverage of `normalize_inserted_part_type` in MapFlowModule]
**Erkenntnis:** The function `normalize_inserted_part_type` in `crates/mapmap-core/src/module/types/module.rs` lacked test coverage. This function is critical structural resolution logic responsible for resolving ID and Name conflicts when inserting specific parts (`Output` and `Layer`). Without tests, edge cases like resolving ID increments or generating fallback names when given an empty name were left untested, posing a risk of regressions causing ID collisions and node mismatches.
**Aktion:** Added `test_normalize_inserted_part_type_resolves_output_id_conflict`, `test_normalize_inserted_part_type_resolves_layer_id_conflict`, and `test_normalize_inserted_part_type_preserves_other_types` to `crates/mapmap-core/src/module/types/module_tests.rs`. These tests ensure ID increments happen correctly, blank/empty names trigger auto-generation, and other variants aren't incorrectly modified.

## 2026-03-24 - [Test coverage of `repair_module` and `repair_modules` in ModuleManager]
**Erkenntnis:** The `repair_module` and `repair_modules` functions in `crates/mapmap-core/src/module/manager.rs` lacked direct unit test coverage in `manager_tests.rs`. These functions are critical for maintaining graph integrity, as they trigger repairs on modules and mark the `ModuleManager` as dirty when changes occur. Without these tests, regressions could silently break the repair mechanism or fail to trigger dirty flags, leading to stale states.
**Aktion:** Added `test_manager_repair_module_repairs_graph_and_marks_dirty` and `test_manager_repair_modules_repairs_multiple` to `crates/mapmap-core/src/module/manager_tests.rs`. These tests ensure that dangling connections are properly cleaned up, and the `ModuleManager`'s dirty flag (`graph_revision`) is appropriately incremented.
