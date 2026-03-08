## 2024-03-04 - Ungetestete ModuleManager Funktion
**Erkenntnis:** Die `ModuleManager` Struktur in `mapmap-core/src/module/manager.rs` war komplett ungetestet. Dies ist kritische Core-Logik.
**Aktion:** Unit Tests für die Modul-Erstellung, -Löschung, -Umbenennung und -Duplizierung hinzugefügt, inklusive Behandlung von Namenskonflikten.
## 2026-03-08 - Zusammensetzung Standardwerte und Grenzen
**Was:** Die `Composition` Struktur und ihre Initialisierung in `crates/mapmap-core/src/layer/composition.rs` wurde intensiv durch Unit-Tests abgedeckt.
**Warum:** Um sicherzustellen, dass die Boundary Conditions, Master Speed/Opacity Limits (0.1 - 10.0, 0.0 - 1.0) und Default-Werte nicht regressieren.
**Abdeckung:** Erreicht vollständige Testabdeckung der Initialisierungslogik.
**Neue Tests:** `test_composition_default_values`, `test_composition_new_initialization`, `test_composition_set_master_opacity_bounds`, `test_composition_set_master_speed_bounds`, `test_composition_with_description_builder`.
**Geänderte Tests:** Keine.
