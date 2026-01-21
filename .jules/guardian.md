# Guardian's Journal 🧪

## 2026-01-14 - Untested Core Logic
**Erkenntnis:** `crates/mapmap-core/src/module_eval.rs` enthält die Kernlogik für Evaluation (Triggers, Signal Propagation, Render Ops), hat aber **KEINE** Unit-Tests. Das ist ein kritisches Risiko, da es die gesamte Show-Ausführung steuert.
**Aktion:** Umfassende Tests für `ModuleEvaluator` implementieren, die Trigger-Evaluation, Signal-Propagation und Chain-Tracing abdecken.
**Aktion:** Bei zukünftigen Implementierungen von Rendering-Logik immer das Koordinatensystem (Normalized vs Pixel, Centered vs Top-Left) explizit validieren. Neue Tests sollten Transformationen mit nicht-trivialen Ankern prüfen.

## 2026-01-14 - GPU Testing Strategy
**Erkenntnis:** GPU-abhängige Tests in `mapmap-render` sind in der CI instabil (flaky).
**Aktion:** GPU-Tests immer mit `#[ignore]` markieren und bei Bedarf manuell ausführen. Mocking für Logik verwenden, die keinen strikten GPU-Kontext benötigt.

## 2026-01-20 - [Implicit Fallback Logic Coverage]
**Erkenntnis:** In `AudioTriggerOutputConfig::generate_outputs` gab es eine implizite Fallback-Logik (wenn alle Outputs deaktiviert sind, wird "Beat Out" erzwungen), die bisher nur durch Default-Tests abgedeckt war. Ein expliziter Test `test_audio_trigger_output_config_fallback_enforcement` stellt sicher, dass dieses Sicherheitsnetz auch bei bewusster Fehlkonfiguration greift.
**Aktion:** Bei Konfigurationsobjekten mit `generate_...` Methoden immer gezielt den "Leeren" Zustand testen, um implizite Fallbacks zu verifizieren.

## 2026-01-24 - [AppState Serialization Guard]
**Erkenntnis:** `AppState` ist die Source-of-Truth und wächst schnell. Ein einfacher Roundtrip-Test (`serde_json`) fängt fehlende `Default`-Implementierungen oder invalide `serde`-Attribute sofort ab, bevor sie zur Laufzeit crashen.
**Aktion:** Bei jedem neuen komplexen Struct, das in `AppState` aufgenommen wird, sicherstellen, dass `Default` und `PartialEq` derived sind, damit der Roundtrip-Test automatisch greift.

## 2026-10-25 - Module Evaluator Tests Implemented
**Erkenntnis:** `ModuleEvaluator` Tests erfolgreich implementiert covering Fixed/Audio Triggers, Signal Propagation, und Full Pipeline. Private Methoden wie `compute_trigger_inputs` wurden implizit über `evaluate` getestet, was gut funktioniert.
**Aktion:** Bei zukünftigen Core-Modulen sicherstellen, dass `evaluate` genügend State exponiert (z.B. via `ModuleEvalResult`), um interne Logik ohne `pub` leaks zu testen.
