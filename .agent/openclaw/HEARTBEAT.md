# HEARTBEAT.md - Der operative Rhythmus

## Status-Check Zyklus (Operativer Loop)

1. **Roadmap-Alignment:**
   - Abgleich des aktuellen Branch-Status mit `ROADMAP.md`.
   - Sind wir auf Kurs für Phase 7 (Timeline V3)?

2. **Debt-Audit:**
   - Scanne `TECHNICAL_DEBT_AND_BUGS.md` nach neuen Einträgen oder kritischen Fix-Möglichkeiten (z.B. GPU-Upload-Thread).

3. **PR-Maintenance:**
   - Prüfe `PR_MAINTENANCE_OVERVIEW.md`.
   - Können PR #742 oder #741 gemerged werden?
   - Gibt es Konflikte bei den IN PROGRESS Sessions?

4. **Health Check:**
   - Laufen CI-Checks fehlerfrei?
   - Gibt es Stagnation in offenen Jules-Sessions?

## Autonome Aktionen
- **Priority Fix:** Erstelle autonom einen Jules-Task, wenn ein kritischer Bug in `TECHNICAL_DEBT_AND_BUGS.md` identifiziert wurde.
- **Auto-Merge:** Führe den Merge durch, sobald ein PR im Maintenance Overview auf "READY" steht und alle Checks grün sind.
- **Konflikt-Lösung:** Beauftrage Jules aktiv mit Rebase/Merge bei Konflikten in hochpriorisierten PRs.
