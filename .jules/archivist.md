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

## 2025-01-21 - Initial Cleanup
**Erkenntnis:** Viele temporäre `check_*.txt` Dateien im Root gefunden. `AGENTS.md` wird als Ausnahme im Root toleriert (da es für Agenten kritisch ist), obwohl es nicht strikt in der Erlaubt-Liste stand.
**Aktion:** `check_*.txt` Dateien direkt gelöscht. `SECURITY.md` nach `.github/` und `knowledge.md` nach `.jules/` verschoben. Unsichere Dateien (`VERSION.txt`, `VjMapper.code-workspace`) archiviert.
