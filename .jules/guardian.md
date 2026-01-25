## 2024-05-23 - [Initial Assessment]
**Erkenntnis:** Codebase has high test coverage in core logic (`module_eval.rs`, `analyzer_v2.rs`). Gap identified in `TriggerConfig::apply` logic within `module.rs` which handles critical value transformation.
**Aktion:** Implement specific unit tests for `TriggerConfig::apply` covering all modes (Direct, Fixed, Random) and edge cases (Invert, Thresholds).

## 2024-05-23 - [Control Logic]
**Erkenntnis:** `MappingCurve::apply` in `midi/mapping.rs` defines the "feel" of controller inputs but relied on implicit testing. Explicit mathematical verification of curves (Exp, Log, S-Curve) ensures hardware controls behave predictably.
**Aktion:** Added dedicated test suite for all `MappingCurve` variants to guarantee correct response curves and bounds clamping.

## 2024-05-24 - [State Persistence]
**Erkenntnis:** `AppState` serialization tests were only checking a subset of fields, risking silent data loss for new features. Deep checking of default states revealed nested managers must also be verified. `dirty` flag exclusion must be explicitly tested to avoid false positive "unsaved changes" warnings.
**Aktion:** Refactored `test_app_state_serialization_roundtrip` to use `assert_eq!` on the full struct (via `PartialEq`). Added specific test `test_dirty_flag_excluded` to guarantee transient flags are not persisted.
