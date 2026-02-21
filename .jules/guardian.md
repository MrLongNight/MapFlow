# Guardian's Journal 🧪

## 2025-05-24 - Initial Insights

**Erkenntnis:** `TriggerConfig::apply` in `mapmap-core` creates a new `rand::rng()` instance on every call for `RandomInRange` mode. This is likely a performance bottleneck in hot paths (e.g., audio triggers).
**Aktion:** Consider refactoring `TriggerConfig::apply` to accept a mutable reference to an RNG or use `thread_rng()` more efficiently. For now, testing acknowledges this behavior.

**Erkenntnis:** `VideoFrame` in `mapmap-io` uses `FrameData::Gpu(Arc<wgpu::Texture>)`, making it difficult to unit test without a GPU context.
**Aktion:** Use `#[ignore]` for GPU-dependent tests or separate logic from resource holding where possible. Ensure CPU fallback paths are robustly tested.

**Erkenntnis:** `ControlValue::validate` uses `std::path::Path::components()` to check for `ParentDir` (`..`) traversal attempts in string values.
**Aktion:** Verify this pattern is consistently applied across all user-input paths to prevent directory traversal attacks.

**Erkenntnis:** `MidiMappingKey` implements `From<&MidiMessage>` returning `Option<MidiMappingKey>`. This is unconventional (vs `TryFrom`) but enables ergonomic `let key: Option<_> = msg.into()` in event loops.
**Aktion:** Document this pattern in `MidiMappingKey` docs to avoid confusion during future refactoring.
