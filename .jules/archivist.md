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

## 2026-01-14 - Duplicate Documentation Cleanup
**Erkenntnis:** Eine redundante Kopie von `CHANGELOG.md` wurde in `docs/08-CHANGELOG/` gefunden. Dies verstößt gegen das "Single Source of Truth"-Prinzip.
**Aktion:** Datei gelöscht. `README.md` im Ordner verweist bereits korrekt auf die Root-Datei. Zukünftige Checks sollten explizit auf Dateiduplikate in `docs/` prüfen.

## 2026-01-09 - Routine Check
**Erkenntnis:** Das Repository ist sauber. Keine fehlplatzierten Dateien im Root. `.gitignore` aktualisiert, um `/.temp-archive/` explizit zu ignorieren.
**Aktion:** Keine weiteren Aktionen erforderlich. Routine-Checks beibehalten.

## 2026-01-07 - Root Directory Cleanup
**Erkenntnis:** Mehrere `check_*.txt` Dateien und falsch platzierte Skripte (`run_mapflow.bat`, `jules-setup.sh`) im Root gefunden. `SECURITY.md` war im Root statt in `.github/`. `VjMapper.code-workspace` lag im Root statt in `.vscode/`.
**Aktion:** Temporäre Textdateien wurden nach `.temp-archive/` verschoben. Skripte wurden nach `scripts/` bewegt. Dokumentation und Workspace-Dateien wurden in ihre entsprechenden Unterordner verschoben. Zukünftige Agenten müssen angewiesen werden, Outputs in `logs/` oder Temp-Verzeichnissen zu speichern und Skripte direkt in `scripts/` zu erstellen.

## 2026-01-18 - Documentation Cleanup
**Erkenntnis:** "Code Analysis Report" in `.temp-archive/` gefunden und "Performance Report" mit Tippfehler/falschem Namen in `docs/07-TECHNICAL/`.
**Aktion:** Dateien nach `docs/07-TECHNICAL/audits/` verschoben und standardkonform benannt.
**Anmerkung:** `docs/04-USER-GUIDE` existiert, entspricht aber nicht dem Standard `02-USER-GUIDE`. Keine Umbenennung vorgenommen, um Links nicht zu brechen ("Erst fragen").
