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

## 2026-01-07 - Root Directory Cleanup
**Erkenntnis:** Mehrere `check_*.txt` Dateien und falsch platzierte Skripte (`run_mapflow.bat`, `jules-setup.sh`) im Root gefunden. `SECURITY.md` war im Root statt in `.github/`. `VjMapper.code-workspace` lag im Root statt in `.vscode/`.
**Aktion:** Temporäre Textdateien wurden nach `.temp-archive/` verschoben. Skripte wurden nach `scripts/` bewegt. Dokumentation und Workspace-Dateien wurden in ihre entsprechenden Unterordner verschoben. Zukünftige Agenten müssen angewiesen werden, Outputs in `logs/` oder Temp-Verzeichnissen zu speichern und Skripte direkt in `scripts/` zu erstellen.
