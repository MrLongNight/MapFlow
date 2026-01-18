# Guardian's Journal 🧪

## 2026-01-14 - Untested Core Logic
**Erkenntnis:** `crates/mapmap-core/src/module_eval.rs` enthält die Kernlogik für Evaluation (Triggers, Signal Propagation, Render Ops), hat aber **KEINE** Unit-Tests. Das ist ein kritisches Risiko, da es die gesamte Show-Ausführung steuert.
**Aktion:** Umfassende Tests für `ModuleEvaluator` implementieren, die Trigger-Evaluation, Signal-Propagation und Chain-Tracing abdecken.

## 2026-01-14 - GPU Testing Strategy
**Erkenntnis:** GPU-abhängige Tests in `mapmap-render` sind in der CI instabil (flaky).
**Aktion:** GPU-Tests immer mit `#[ignore]` markieren und bei Bedarf manuell ausführen. Mocking für Logik verwenden, die keinen strikten GPU-Kontext benötigt.
