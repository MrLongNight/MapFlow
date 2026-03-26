# Jules/Gemini Issue Orchestration

## Ziel

Dieser Ablauf steuert ein Master-Issue mit paarweise gekoppelten Subissues:

1. ungerades Subissue = Implementierung durch Jules
2. gerades Subissue = Verifikation durch Gemini CLI

Die Reihenfolge ist strikt. Das naechste Jules-Issue darf erst starten, wenn das zugehoerige Verify-Issue fachlich abgeschlossen ist und ein PASS vorliegt.

## Definierter Soll-Prozess

### 1. Paar aus Master-Issue lesen

- `run-master-issue-cycle.ps1` liest aus dem Master-Issue die beiden Sektionen:
  - `Implementation Subissues (Jules)`
  - `Verification Subissues (4-eyes/Review immediately after each implementation)`
- Die Paare werden nur per Positionsindex gebildet:
  - `Implementation[0] <-> Verification[0]`
  - `Implementation[1] <-> Verification[1]`
  - usw.

### 2. Bereits vollstaendig erledigte Paare ueberspringen

- Ein Implementierungs-Issue gilt als abgeschlossen, wenn:
  - `Status` im Issue-Body final ist (`Done`, `Completed`, `Closed`, `Merged`)
  - und `remote_state = merged`
- Ein Verify-Issue gilt als PASS-abgeschlossen, wenn:
  - `Status` final ist
  - und der juengste Verifikationskommentar `PASS` signalisiert
- Nur wenn beide Bedingungen erfuellt sind, wird das Paar uebersprungen.

### 3. Jules-Implementierung starten oder fortsetzen

- Wenn das Implementierungs-Issue noch nicht abgeschlossen ist:
  - vorhandene Jules-Session wiederverwenden, falls eine Session-Referenz im Issue existiert
  - sonst neue Session per `create-jules-session.ps1` starten
- Der Wrapper darf nur mit aktiv bestaetigtem `AUTO_CREATE_PR` weitermachen.
- Wenn Jules stattdessen in einen Attention-Status laeuft (`PAUSED`, `FAILED`, `AWAITING_*`), stoppt der Ablauf sofort.

### 4. Warte- und Polling-Regeln

- Nach dem erfolgreichen Start einer lauffaehigen Jules-Session:
  - erster Folgecheck nach 15 Minuten
  - danach Polling alle 5 Minuten
- Wenn Jules schneller fertig ist, wird das beim naechsten Poll erkannt.
- Wenn die Session nicht fertig ist, bleibt der Wrapper im Polling.

### 5. PR-Ermittlung

- Primaer wird der PR-Link direkt aus der Jules-Session gelesen.
- Falls Jules keinen PR-Link an die Session haengt, sucht der Wrapper ersatzweise in GitHub nach einem existierenden PR:
  - zuerst ueber die Session-ID im PR-Body
  - dann ueber `Fixes #<Issue>`
  - danach ueber `#<Issue>` im PR-Body
- Damit kann ein manuell aus Jules nach GitHub gesendeter PR trotzdem noch korrekt uebernommen werden.

### 6. PR-Gate

- Der Wrapper wartet anschliessend auf den Merge des PR.
- Vor und waehrend des Wartens wird das erwartete Naming geprueft:
  - PR-Titel = `PR` + Issue-Titel
  - Work-Branch = `B-Jules/` + Issue-Titel
- Bei roten Required Checks stoppt der Ablauf sofort.
- Erst nach Merge wird das Implementierungs-Issue auf den finalen Stand gespiegelt.

### 7. Verify-Issue durch Gemini CLI bearbeiten

- Verifikation passiert immer auf einem sauberen Worktree gegen `origin/main`.
- Gemini CLI bekommt nicht nur eine Review-Aufgabe, sondern muss das Verify-Issue selbst aktualisieren:
  - Checkboxes setzen
  - `Status` im Issue-Body setzen
  - Managed-Block/Felder schreiben
  - PASS- oder REJECT-Kommentar schreiben
  - Ergebnis per `gh issue view` selbst gegenpruefen
- Dafuer wird `set-managed-issue-state.ps1` als fester Repo-Befehl verwendet.

### 8. Verify-Semantik

- Ein Verify-Issue ist fachlich abgeschlossen, wenn Gemini den Review wirklich beendet hat.
- Das Verify-Issue wird dabei auf `Status = Done` gesetzt, egal ob PASS oder REJECT.
- Der Unterschied zwischen PASS und REJECT liegt nicht im Status, sondern im Verifikationskommentar und in den gesetzten Checkboxen.
- Nur bei PASS darf der Wrapper mit dem naechsten Jules-Issue weitermachen.
- Bei REJECT stoppt der Wrapper nach Abschluss des Verify-Issues.

### 9. Keine manuellen Close-Aktionen im Normalfall

- Der Wrapper soll Issues nicht aktiv auf `closed` setzen.
- Der fachliche Abschluss wird ueber die Felder gespiegelt:
  - `Status`
  - `remote_state`
  - Projekt-Felder im `@Vorce Project Manager`, insbesondere `jules_session_status` und `pr_checks_status`
- Falls externe Automatisierung Issues auf Basis von `Status = Done` schliesst, ist das zulaessig.

## Technische Umsetzung im Repo

### `scripts/jules/run-master-issue-cycle.ps1`

Dieses Skript ist der Orchestrator.

Es setzt um:

- Auslesen und Paarbildung aus dem Master-Issue
- 15-Minuten-Initialdelay und 5-Minuten-Polling
- Session-Reuse oder Session-Start
- Pflicht-Check auf bestaetigtes `AUTO_CREATE_PR`
- PR-Fallback-Suche, wenn Jules keinen Link an die Session haengt
- Merge-Wartephase
- Start von Gemini CLI fuer das Verify-Issue
- harte Stop-Bedingung bei Jules-Attention, roten Checks oder Gemini-REJECT

### `scripts/jules/create-jules-session.ps1`

Dieses Skript startet Jules-Sessions aus einem GitHub-Issue.

Es setzt um:

- Umwandlung des Issue-Contents in einen Jules-Prompt
- Setzen von `automationMode = AUTO_CREATE_PR`, wenn gewuenscht
- Vorgabe des verbindlichen PR-/Branch-Namings an Jules
- Rueckgabe der Session-ID und des aufgeloesten Automation-Mode
- Kommentar und Issue-Tracking fuer die gestartete Session

### `scripts/jules/set-managed-issue-state.ps1`

Dieses Skript schreibt den finalen Zustand eines Issues.

Es setzt um:

- Update der sichtbaren Issue-Form-Felder:
  - `Status`
  - `agent`
  - `jules_session`
  - `remote_state`
  - `work_branch`
  - `last_update`
- Update des Managed-Blocks im Issue-Body
- direkte Synchronisation in das GitHub-Projekt `@Vorce Project Manager`
  - `jules_session_status`
  - `pr_checks_status`

### `scripts/jules/jules-github.ps1`

Diese Datei enthaelt die GitHub-/Project-Helfer.

Hier wurde explizit nachgezogen:

- automatische Erkennung des Projekts `@Vorce Project Manager`, auch ohne gesetzte `VORCE_PROJECT_NUMBER`
- robustere User-/Org-Erkennung fuer GitHub Projects
- GraphQL-Fehlerbehandlung ohne Strict-Mode-Absturz bei fehlender `errors`- oder `options`-Property

## Bekannte Stop-Bedingungen

Der Wrapper stoppt absichtlich in diesen Faellen:

- Jules-Session braucht Input oder haengt in einem Attention-State
- `AUTO_CREATE_PR` ist nicht bestaetigt
- kein PR ist auffindbar
- Required Checks sind rot
- Gemini hat das Verify-Issue nicht wirklich auf einen finalen Review-Zustand gebracht
- Gemini liefert REJECT

## Warum das absichtlich streng ist

Der Ablauf soll nicht scheinbar erfolgreich weiterlaufen, wenn der aktuelle Schritt in Wahrheit inkonsistent ist.

Die gewuenschte Reihenfolge ist:

1. Jules liefert
2. PR wird sauber erzeugt und gemerged
3. Gemini prueft und dokumentiert das Review-Issue
4. nur bei PASS startet das naechste Jules-Issue

Genau auf diese Reihenfolge sind die aktuellen Skriptanpassungen ausgerichtet.
