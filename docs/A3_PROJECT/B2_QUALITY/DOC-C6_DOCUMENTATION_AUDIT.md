# Dokumentations Audit

## Struktur Analyse

Das `docs/` Verzeichnis ist grundsätzlich gut strukturiert (semantische Ordner wie user/, dev/, project/).

### Vorhandene Ordner
- `user/getting-started`: Einstieg
- `CONTRIBUTING`: Guidelines
- `dev/architecture`: Architektur-Docs
- `user/manual`: Handbuch
- `project/roadmap`: Planungsunterlagen
- `user/tutorials`: Anleitungen
- `dev/technical`: Technische Details
- `CHANGELOG`: Historie
- `project/resources`: Externe Ressourcen
- `dev/features/OSC-CONTROL`: Spezifische Doku für OSC

### Auffälligkeiten & "Loose Files"
Folgende Dateien liegen direkt im Root von `docs/` und sollten verschoben werden:
- `HAP_INTEGRATION_PLAN.md` -> Nach `docs/dev/technical/` oder `docs/project/roadmap/` (wenn abgeschlossen Archiv).
- `MCP-API.md` -> Nach `docs/dev/technical/` oder neuer Ordner `docs/dev/api/`.
- `MIDI_USER_GUIDE.md` -> Nach `docs/user/manual/` oder `docs/dev/features/dev/features/OSC-CONTROL/`.
- `cleanup-summary.md` -> Vermutlich alt/temporär. Archivieren oder löschen.
- `CODE_ANALYSIS_REPORT.md` -> (Vom User?) Ggf. konsolidieren mit `CODE_AUDIT_REPORT.md`.

## Optimierungsvorschläge

1.  **Verschieben von Loose Files:** (Siehe oben).
2.  **Namenskonventionen:** Verwende einheitliche Kleinschreibung und Bindestriche (kebab-case) für Ordnernamen (z.B. `user-manual` statt `Manual`).
3.  **Update Index:** Stelle sicher, dass `INDEX.md` alle neuen Dokumente referenziert.

## Geplante Tasks
- [ ] Move `HAP_INTEGRATION_PLAN.md` to `docs/dev/technical/`
- [ ] Move `MCP-API.md` to `docs/dev/technical/`
- [ ] Move `MIDI_USER_GUIDE.md` to `docs/user/manual/`
