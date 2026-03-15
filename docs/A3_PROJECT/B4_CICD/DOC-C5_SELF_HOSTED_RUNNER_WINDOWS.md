# Self-Hosted Runner auf Windows 10 fuer Post-Merge-Jobs

Stand: 2026-03-14

## Ziel

Diese Anleitung bereitet einen lokalen GitHub Actions Runner auf einem aelteren Windows-10-PC fuer MapFlow vor, ohne ihn sofort produktiv zu aktivieren.

Der geplante Einsatz ist bewusst eingeschraenkt:

- kein Einsatz fuer normale Commits oder Pushes
- kein Einsatz vor dem Merge
- nur fuer Post-Merge-Jobs nach einem bereits erfolgreich geprueften Pull Request
- standardmaessig deaktiviert, bis die Repo-Variable zur Aktivierung gesetzt wird

Die vorbereiteten Repo-Dateien sind:

- Workflow: `.github/workflows/CICD-DevFlow_Job03_PostMergeSelfHosted.yml`
- Runner-Skript: `scripts/build/self-hosted-post-merge.ps1`
- Visual-Capture-Readiness: `docs/A3_PROJECT/B2_QUALITY/DOC-C9_VISUAL_CAPTURE_READINESS.md`

## Kurzentscheidung zur Plattform

Ja, der Runner kann auf Windows 10 laufen, auch wenn die Entwicklung auf Windows 11 stattfindet.

Wichtig ist nicht, dass beide Maschinen exakt dieselbe Windows-Version haben, sondern dass der Runner:

- 64-Bit Windows nutzt
- aktuelle Sicherheitsupdates und GPU-Treiber hat
- eine fuer MapFlow brauchbare GPU besitzt
- dieselben kritischen Toolchains und Laufzeitabhaengigkeiten bereitstellt wie die Entwicklungsmaschine

Fuer MapFlow sind in der Praxis wichtiger als Windows 10 vs. Windows 11:

- Rust-Toolchain
- Git
- Visual Studio C++ Build Tools
- FFmpeg-/Media-Abhaengigkeiten
- GPU-Treiber und WGPU-faehiger Grafikstack

## Gewuenschter Workflow

Der vorbereitete Ablauf sieht so aus:

1. Ein Pull Request laeuft zuerst nur auf GitHub-hosted Runnern.
2. Die Standard-PR-Checks muessen erfolgreich sein.
3. Erst nach dem Merge wird ein separater Post-Merge-Workflow relevant.
4. Dieser Workflow darf nur auf dem lokalen self-hosted Runner laufen.
5. Die Aktivierung erfolgt spaeter ueber eine Repo-Variable und bleibt bis dahin aus.

Die technische Umsetzung im neuen Workflow ist:

1. Trigger auf `pull_request_target` mit `closed`
2. sofortiger Abbruch fuer nicht gemergte PRs
3. Pruefung der bereits erfolgreichen Standard-PR-Checks auf dem gemergten PR-Head
4. Checkout des echten `merge_commit_sha`
5. Ausfuehrung nur auf `runs-on: [self-hosted, windows, x64, mapflow-post-merge]`

## Empfohlene Runner-Rolle

Empfohlene Labels:

- `self-hosted`
- `windows`
- `x64`
- `mapflow-post-merge`

Damit kann der Workflow spaeter gezielt nur diesen Runner verwenden, statt alle selbst gehosteten Windows-Runner zu treffen.

## Vorbereitung des Windows-10-PCs

### 1. Betriebssystem und Laufzeit stabilisieren

- Energiesparmodus und automatischen Standby deaktivieren
- automatische Neustarts waehrend geplanter CI-Zeiten vermeiden
- feste lokale Admin-Rechte fuer die Runner-Installation sicherstellen
- Windows Update und GPU-Treiber vor der ersten Aktivierung aktualisieren

### 2. Entwicklungsabhaengigkeiten angleichen

Mindestens installieren oder pruefen:

- Git
- Rust stable Toolchain
- Visual Studio Build Tools mit C++-Werkzeugen
- alle fuer MapFlow lokal benoetigten SDKs oder Laufzeitbibliotheken

Optional, aber empfehlenswert:

- gleiche Rust-Edition und Cargo-Konfiguration wie auf der Entwicklungsmaschine
- gleiche FFmpeg- oder Medien-Tooling-Versionen
- gleiche GPU-Herstellerfamilie wie auf dem Hauptrechner, falls Grafikfehler repliziert werden sollen

### 3. Modus frueh festlegen

Es gibt zwei sinnvolle Betriebsarten:

### A. Build- und Script-Runner

Geeignet fuer:

- Builds
- Tests ohne sichtbare Fenster
- allgemeine Post-Merge-Pruefungen

Hier kann der Runner als Windows-Service laufen.

### B. Sichtbarer GUI-Runner

Geeignet fuer:

- spaetere sichtbare MapFlow-Fenster
- Screenshots
- Videoaufnahmen
- multimodale Auswertung

Hier sollte der Runner nicht nur als Service in Session 0 laufen, sondern interaktiv in einer angemeldeten Benutzersitzung. Fuer echte sichtbare GUI-Automation ist das die sichere Variante.

## Runner in GitHub registrieren

Repository:

1. `Settings`
2. `Actions`
3. `Runners`
4. `New self-hosted runner`
5. Windows auswaehlen

Die von GitHub angezeigten Befehle dann lokal auf dem Windows-10-PC ausfuehren.

Empfehlung:

- Runner in einen eigenen Ordner wie `C:\actions-runner-mapflow-post-merge` legen
- bei der Konfiguration einen sprechenden Namen vergeben
- die zusaetzlichen Labels um `mapflow-post-merge` erweitern

## Aktivierungsstrategie

Die vorbereitete CI/CD-Aenderung ist absichtlich deaktiviert.

Die spaetere Aktivierung soll nur ueber eine Repo-Variable erfolgen:

- `MAPFLOW_ENABLE_SELF_HOSTED_POST_MERGE=true`

Solange diese Variable nicht gesetzt ist oder auf `false` steht, bleibt der neue Post-Merge-Pfad inaktiv.

Optionale Spaeter-Schalter:

- `MAPFLOW_SELF_HOSTED_RUN_IGNORED_GPU_TESTS=true`
- `MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION=true`

Das ist bewusst besser als ein direkt aktivierter Workflow, weil:

- die Workflow-Dateien bereits im Repo liegen koennen
- die Aktivierung ohne weiteren Code-Commit erfolgen kann
- die Deaktivierung ebenso schnell wieder moeglich ist

## Einfache temporaere Deaktivierung

Empfohlene Reihenfolge:

1. Repo-Variable auf `false` setzen oder entfernen
2. optional den Runner in GitHub auf `offline` gehen lassen
3. optional den lokalen Runner-Dienst oder die Runner-App stoppen

Praktische Varianten:

- global aus: Repo-Variable deaktivieren
- lokal gewartet: Runner-Dienst oder Runner-App stoppen
- dauerhaft aus dem Routing nehmen: Label `mapflow-post-merge` entfernen oder Runner aus GitHub abmelden
- einzelnen PR ausnehmen: Label `skip-self-hosted-post-merge` setzen

## Was der vorbereitete Job aktuell macht

Der Workflow ruft dieses Skript auf:

- `scripts/build/self-hosted-post-merge.ps1`

Aktuell erledigt der Job:

- Pruefung auf `git`, `cargo`, `rustup`, `LLVM/Clang` und `vcpkg`
- Bootstrap von `vcpkg`, falls noetig
- Installation der Manifest-Abhaengigkeiten fuer Windows
- Release-Build von `mapmap` mit `audio,ffmpeg`
- optional spaeter ignorierte GPU-Tests
- optionale lokale visuelle Screenshot-Regressionstests

Lokaler Start fuer die visuellen Tests:

```powershell
$env:MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR = "artifacts/visual-capture"
cargo test -p mapmap --no-default-features --test visual_capture_tests -- --ignored --nocapture
```

Der vorbereitete Self-hosted-Job setzt diesen Ordner automatisch und kann die erzeugten
Screenshots spaeter als Workflow-Artefakt hochladen.

Relative Pfade fuer `MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR` werden gegen die Repo-Wurzel aufgeloest.

Schneller Not-Aus direkt auf dem Runner:

```powershell
Stop-Service "actions.runner.*"
```

Wieder starten:

```powershell
Start-Service "actions.runner.*"
```

## Empfehlung fuer MapFlow

Kurzfristig:

- Runner vorbereiten
- Workflow-Dateien einchecken
- Aktivierung noch nicht setzen

Mittelfristig:

- zuerst nur Build- und Smoke-Pruefungen ueber den self-hosted Runner einschalten
- die vorhandenen Harness-Screenshot-Tests bei Bedarf aktivieren
- den spaeteren kompletten App-GUI-Testmodus erst danach aufbauen

Spaeter fuer Visual Capture:

- interaktive Benutzersitzung statt reiner Service-Ausfuehrung
- Bildschirmsperre, Sleep und Auto-Logout konsequent verhindern
- dedizierten Runner moeglichst nicht fuer parallele Alltagsnutzung verwenden

## Minimale Aktivierungs-Checkliste

- Windows-10-PC stabil und gepatcht
- Git und Rust installiert
- MapFlow lokal baubar
- Runner mit Label `mapflow-post-merge` registriert
- Repo-Variable noch nicht gesetzt
- interaktiver Modus fuer spaetere GUI-Tests eingeplant

## Offizielle GitHub-Dokumentation

- Adding self-hosted runners:
  - <https://docs.github.com/actions/hosting-your-own-runners/adding-self-hosted-runners>
- Self-hosted runner reference and supported platforms:
  - <https://docs.github.com/en/actions/reference/runners/self-hosted-runners>
- Using self-hosted runners in a workflow:
  - <https://docs.github.com/actions/hosting-your-own-runners/using-self-hosted-runners-in-a-workflow>
- Using labels with self-hosted runners:
  - <https://docs.github.com/actions/hosting-your-own-runners/using-labels-with-self-hosted-runners>
- Configuring the self-hosted runner application as a service:
  - <https://docs.github.com/actions/hosting-your-own-runners/configuring-the-self-hosted-runner-application-as-a-service>
- Choosing the runner for a job:
  - <https://docs.github.com/en/actions/writing-workflows/choosing-where-your-workflow-runs/choosing-the-runner-for-a-job>
- Events that trigger workflows:
  - <https://docs.github.com/actions/learn-github-actions/events-that-trigger-workflows>
- Removing self-hosted runners:
  - <https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners/removing-self-hosted-runners>
