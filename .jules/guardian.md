# 🧪 Guardian's Journal

Kritische Erkenntnisse aus Unit-Test-Aktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2025-02-19 - Critical Gap in Module Logic
**Erkenntnis:** Das Herzstück der Anwendung, `MapFlowModule` (in `mapmap-core`), hatte null Unit-Tests für seine interne Logik (Parts hinzufügen, Sockets berechnen, Verbindungen verwalten). Die existierenden Tests prüften nur triviale Erstellung/Löschung via `ModuleManager`.
**Aktion:** Bei "High Priority" Modulen immer zuerst die *interne* Logik prüfen, nicht nur die Manager-API. Komplexe Structs mit vielen Methoden (`impl MapFlowModule`) brauchen dedizierte Test-Suites (wie jetzt `module_logic_tests.rs`).
