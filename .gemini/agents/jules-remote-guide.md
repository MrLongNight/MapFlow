# 🚀 Jules Remote Agent Guide

Jules ist nun als Remote-Agent für **MapFlow** konfiguriert. Er eignet sich hervorragend für Aufgaben, die zu groß für den lokalen Kontext sind.

## Beispiel: AppUI Refactoring
Um das große `AppUI` Struct in `mapmap-ui` aufzuteilen, kannst du Jules wie folgt beauftragen:

1. **Sitzung erstellen**:
   ```bash
   gemini agent spawn jules-remote --objective "Refactor AppUI in crates/mapmap-ui/src/lib.rs. Split it into modular state components like ViewSettings, ProjectState, and IOConfig to reduce technical debt."
   ```

2. **Plan prüfen**:
   Jules wird einen detaillierten Plan erstellen. Du kannst ihn mit `jules_get_session` einsehen.

3. **Genehmigen**:
   Sobald der Plan steht, genehmige ihn mit `jules_approve_plan`.

4. **Überwachen**:
   Nutze `jules_get_activities`, um den Fortschritt der Dateiänderungen in Echtzeit zu verfolgen.

## Wann Jules nutzen?
- Große Refactorings über mehrere Crates hinweg.
- Implementierung neuer komplexer Module (z.B. ein komplett neues Output-System).
- Tiefgehende Code-Analysen und Sicherheits-Audits des gesamten Workspaces.
