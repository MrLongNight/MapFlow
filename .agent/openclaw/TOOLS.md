# TOOLS.md - Werkzeuge und Wissensquellen

## Primäre Wissensquellen
- **`ROADMAP.md`**: Dein Kompass für die langfristige Entwicklung.
- **`TECHNICAL_DEBT_AND_BUGS.md`**: Dein Aufgabenkatalog für Stabilisierung und Refactoring.
- **`PR_MAINTENANCE_OVERVIEW.md`**: Deine Übersicht für den aktuellen PR-Status.
- **`Cargo.toml` (Workspace)**: Deine Übersicht über die Crate-Architektur und Features.

## Primäre Werkzeuge
- **Google Jules Cli:** Dein primärer "Coding-Executor". Du beauftragst ihn mit komplexen Refactorings und Feature-Implementierungen.
- **Google Gemini CLI:** Dein Werkzeug für schnelle Analysen, Unit-Tests und das Erstellen von PR-Beschreibungen.
- **GitHub Tools:** Zur Überwachung von Issues, PRs und automatisierten Merges.

## Delegations-Protokoll
1. **Analysiere:** Prüfe `TECHNICAL_DEBT_AND_BUGS.md` auf kritische Blocker.
2. **Plane:** Entwirf einen Plan in `.agent/plans/` (z.B. `ARCH_FIX_UNSAFE_HACKS.md`).
3. **Beauftrage Jules:** Nutze Jules CLI `create_coding_task`, um Jules Sessions zu erstellen und zu überwachen die den Plan umzusetzen.
4. **Beauftrage Gemini CLI:** Nutze Sub-Agenten, um die Änderungen gegen die Roadmap zu validieren oder Tests zu schreiben.
5. **Merge:** Führe den Merge erst aus, wenn alle PR-Checks aus `PR_MAINTENANCE_OVERVIEW.md` bestanden sind.
