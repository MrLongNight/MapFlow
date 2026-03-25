# Jules PowerShell

PowerShell-Skripte fuer die Jules REST API mit optionaler GitHub-Issue-Synchronisation.

## Voraussetzungen

- `JULES_API_KEY` als Umgebungsvariable oder per `-ApiKey`
- `gh` CLI fuer Issue-Integration
- Repository muss in Jules als Source verbunden sein

## Skripte

- `create-jules-session.ps1`: Erstellt Jules Sessions direkt aus `-IssueNumber` oder `-Prompt`
- `monitor-jules-sessions.ps1`: Listet Sessions, Status, Attention-Zustaende und letzte Aktivitaeten
- `respond-jules-session.ps1`: Reagiert auf Sessions mit `approvePlan` und `sendMessage`
- `run-master-issue-cycle.ps1`: Liest alle Issue-Paare aus einem Master-Issue und startet die komplette Kette automatisch
- `sync-project-manager.ps1`: Synchronisiert alle relevanten Issues auf einen kompakten, verlaesslichen `MapFlow Project Manager`-Block

## Beispiele

```powershell
./scripts/jules/create-jules-session.ps1 -IssueNumber 123 -AutoCreatePr
./scripts/jules/create-jules-session.ps1 -Prompt "Refactor the OSC routing layer" -Repository MrLongNight/MapFlow
./scripts/jules/monitor-jules-sessions.ps1 -OnlyActive -IncludeActivities
./scripts/jules/monitor-jules-sessions.ps1 -IssueNumber 123 -SyncIssueBody
./scripts/jules/respond-jules-session.ps1 -IssueNumber 123 -ApprovePlan
./scripts/jules/respond-jules-session.ps1 -IssueNumber 123 -Message "Bitte nutze die bestehenden UI-Komponenten in crates/mapmap-ui."
./scripts/jules/run-master-issue-cycle.ps1 -MasterIssueNumber 1203 -Repository MrLongNight/MapFlow
./scripts/jules/sync-project-manager.ps1 -Repository MrLongNight/MapFlow -FailOnAttention
```

## GitHub-Issue-Felder

Standard-GitHub-Issues haben per REST API keine frei definierbaren Custom Fields. Die Skripte schreiben deshalb einen verwalteten, bewusst kompakten Block `## MapFlow Project Manager` in den Issue-Body und koennen zusaetzlich Kommentare posten. Sichtbar gehalten werden nur die verlaesslichen Kernfelder `Queue State`, `Remote State`, `Work Branch`, `Linked PR` und `Last Update`.

## Optionale GitHub-Project-V2-Synchronisation

Wenn die Project-Ansicht direkt gepflegt werden soll, koennen dieselben Werte optional in ein GitHub Project V2 geschrieben werden. Dazu muessen mindestens diese Umgebungsvariablen gesetzt sein:

- `MAPFLOW_PROJECT_OWNER`
- `MAPFLOW_PROJECT_NUMBER`

Optional koennen abweichende Feldnamen konfiguriert werden:

- `MAPFLOW_PROJECT_STATUS_FIELD`
- `MAPFLOW_PROJECT_QUEUE_STATE_FIELD`
- `MAPFLOW_PROJECT_REMOTE_STATE_FIELD`
- `MAPFLOW_PROJECT_WORK_BRANCH_FIELD`
- `MAPFLOW_PROJECT_LAST_UPDATE_FIELD`
- `MAPFLOW_PROJECT_LINKED_PR_FIELD`

Wenn diese Konfiguration fehlt, bleibt der Project-Sync deaktiviert und nur der Issue-Sync wird ausgefuehrt.
