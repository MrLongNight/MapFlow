# 🗂️ Archivist's Journal

Kritische Erkenntnisse aus Repository-Verwaltungsaktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2026-01-12 - Shader Inconsistency Detected
**Erkenntnis:** Inkonsistenz bei der Shader-Platzierung entdeckt. Während die meisten globalen Shader in `shaders/` liegen, befinden sich Effektspezifische Shader in `crates/mapmap-render/shaders/` und werden dort aktiv via relativen Pfaden (`include_str!("../shaders/...")`) eingebunden.
**Aktion:** `ycocg_to_rgb.wgsl` wurde nach `shaders/` verschoben, da es laut Roadmap dort liegen soll und aktuell nicht im Code referenziert wird (Safe Move). Die anderen Effekt-Shader wurden vorerst belassen, um Build-Fehler zu vermeiden, da dies Code-Änderungen erfordern würde (Scope Violation für Archivist ohne expliziten Auftrag). Zukünftige Refactorings sollten alle Shader zentral in `shaders/` konsolidieren.

## 2026-01-09 - Routine Check
**Erkenntnis:** Das Repository ist sauber. Keine fehlplatzierten Dateien im Root. `.gitignore` aktualisiert, um `/.temp-archive/` explizit zu ignorieren.
**Aktion:** Keine weiteren Aktionen erforderlich. Routine-Checks beibehalten.

## 2026-01-07 - Root Directory Cleanup
**Erkenntnis:** Mehrere `check_*.txt` Dateien und falsch platzierte Skripte (`run_mapflow.bat`, `jules-setup.sh`) im Root gefunden. `SECURITY.md` war im Root statt in `.github/`. `VjMapper.code-workspace` lag im Root statt in `.vscode/`.
**Aktion:** Temporäre Textdateien wurden nach `.temp-archive/` verschoben. Skripte wurden nach `scripts/` bewegt. Dokumentation und Workspace-Dateien wurden in ihre entsprechenden Unterordner verschoben. Zukünftige Agenten müssen angewiesen werden, Outputs in `logs/` oder Temp-Verzeichnissen zu speichern und Skripte direkt in `scripts/` zu erstellen.
