# Guardian's Journal 🧪

## 2026-01-14 - Untested Core Logic
**Erkenntnis:** `crates/mapmap-core/src/module_eval.rs` enthält die Kernlogik für Evaluation (Triggers, Signal Propagation, Render Ops), hat aber **KEINE** Unit-Tests. Das ist ein kritisches Risiko, da es die gesamte Show-Ausführung steuert.
**Aktion:** Umfassende Tests für `ModuleEvaluator` implementieren, die Trigger-Evaluation, Signal-Propagation und Chain-Tracing abdecken.

## 2026-01-14 - GPU Testing Strategy
**Erkenntnis:** GPU-abhängige Tests in `mapmap-render` sind in der CI instabil (flaky).
**Aktion:** GPU-Tests immer mit `#[ignore]` markieren und bei Bedarf manuell ausführen. Mocking für Logik verwenden, die keinen strikten GPU-Kontext benötigt.

## 2026-01-14 - Audio Analysis Testing
**Erkenntnis:** Tests für `AudioAnalyzerV2` (insb. BPM-Erkennung) sind sehr empfindlich bzgl. Signal-Timing und FFT-Fenstergrößen.
**Aktion:** Deterministische Waveforms mit präzisen Samples generieren, statt echten Audio-Input zu simulieren. Edge-Cases (z.B. BPM-Doubling) benötigen saubere Signale über längere Intervalle (>4 Beats), damit die Heuristik greift.

## 2026-01-14 - State Serialization Testing
**Erkenntnis:** `AppState` Serialisierung überspringt `dirty`-Flag (korrektes Verhalten), aber Tests haben das bisher nur implizit geprüft.
**Aktion:** Explizite Tests hinzufügen (`test_app_state_serialization_skip_dirty`), die sicherstellen, dass transiente Felder *wirklich* nicht im JSON landen.
