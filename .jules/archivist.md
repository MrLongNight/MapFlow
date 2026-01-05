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

## 2026-01-05 - Initialer Cleanup

**Erkenntnis:** Das Root-Verzeichnis war mit temporären Dateien (`check_*.txt`, `test_results.txt`) und falsch platzierten Skripten (`jules-setup.sh`, `run_mapflow.bat`) überfüllt.
**Aktion:**
- Temporäre Dateien gelöscht.
- Skripte nach `scripts/` verschoben.
- `SECURITY.md` nach `.github/` verschoben (konform mit GitHub Standard).
- `.vscode/` erstellt für Workspace-Settings.
- `VERSION.txt` und `.mapmap_autosave` archiviert.
