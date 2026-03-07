# MapFlow ROADMAP (MF-ID gesteuert)

> **Stand:** 2026-03-06 23:35
Operatives Hauptdokument: `ROADMAP.md`
Sprache: Deutsch

## Legende

- MF-ID Format: `MF-###-SLUG` (Beispiel: `MF-001-AUDIO-UI-CONSOLIDATION`)
- Branch und PR muessen die MF-ID im Namen tragen.
- Pflichtstatus pro Task: `Dev-Status` und `QA-Status (User)`.
- Statuswerte:
  - 🔴 `Offen`
  - 🟠 `In Analyse`
  - 🔵 `In Umsetzung`
  - 🟢 `Bereit fuer QA`
  - 🟣 `Nacharbeit`
  - ✅ `Abgeschlossen`
- "Wird das wirklich benoetigt?" wird ueber `Beschreibung` (Scope/Prioritaet) gefuehrt, nicht als eigene Sektion.

## Task-Board

| Task-ID | Bereich | Typ | Beschreibung | Quellreferenz(en) | Dev-Status | QA-Status (User) | Session ID | Branch | PR | Letztes Update |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| MF-001-AUDIO-UI-CONSOLIDATION | UI/Audio | Bug | Prio A: Doppelte Audio-Analyse entfernen, Floating-Panel abbauen. | SRC-022, SRC-023, SRC-059 | 🟢 Bereit fuer QA | 🔴 Offen | 12625940356526816833 | `MF-001-audio-ui-consolidation` | #949 | 2026-03-06 |
| MF-002-AUDIO-PIPELINE-RECOVERY | Core/Audio | Bug | Prio A: Audioanalyse funktional wiederherstellen (Input -> Analyzer -> Evaluator -> UI). | SRC-004, SRC-024, SRC-060, SRC-061, SRC-062 | ✅ Abgeschlossen | 🟢 QA Erfolgreich | 3614160160212922544 | `MF-002-audio-pipeline-recovery` | #945 | 2026-03-06 |
| MF-004-CONNECTION-TYPE-SAFETY | Module Canvas | Bug | Prio A: Nur typgleiche Ein-/Ausgaenge verbindbar machen. | SRC-026, SRC-063 | 🟢 Bereit fuer QA | 🔴 Offen | 7230266295377882773 | `MF-004-connection-type-safety` | #947 | 2026-03-06 |
| MF-005-LAYER-MESH-EDITOR-RESTORE | Inspector/Layer | Bug | Prio A: Grafischen Mesh-Editor im Layer-Inspector sicher verfuegbar machen. | SRC-031, SRC-040 | 🔵 In Umsetzung | 🔴 Offen | 11200614113557340110 | `MF-005-layer-mesh-editor-restore` | - | 2026-03-06 |
| MF-007-SETTINGS-GFX-PERF-WIRING | Settings | Bug | Prio A: Grafik/Performance Controls wirksam, persistiert und runtime-wirksam machen. | SRC-027, SRC-065 | 🟢 Bereit fuer QA | 🔴 Offen | 14675983253663977133 | `MF-007-settings-gfx-perf-wiring` | #948 | 2026-03-06 |
| MF-008-THEME-SWITCHER-INTEGRATION | Settings/UI | Feature | Prio A: Theme-Switcher inkl. Auswahlmenue und Persistenz integrieren. | SRC-028, SRC-044, SRC-066 | 🟢 Bereit fuer QA | 🔴 Offen | 17474352207017899925 | `MF-008-theme-switcher-integration` | #950 | 2026-03-06 |
| MF-009-HUE-DTLS-OPENSSL-PATH | Control/Hue | Bug | Prio A: DTLS/OpenSSL Fehlerpfad beheben und stabile Hue-Connection herstellen. | SRC-009, SRC-029, SRC-033, SRC-067 | 🟠 In Analyse | 🔴 Offen | - | `MF-009-hue-dtls-openssl-path` | - | 2026-03-06 |
| MF-011-UI-PANEL-CONSISTENCY | UI Layout | Bug | Prio A: Fehlende/falsch platzierte Panels and Funktionsluecken konsistent machen. | SRC-043, SRC-069 | 🟠 In Analyse | 🔴 Offen | - | `MF-011-ui-panel-consistency` | - | 2026-03-06 |
| MF-014-AUTOSAVE-COMPAT | Core/IO | Bug | Prio B: Autosave-Backward-Kompatibilitaet (`master_blackout`) absichern. | SRC-071 | 🟠 In Analyse | 🔴 Offen | - | `MF-014-autosave-compat` | - | 2026-03-06 |
| MF-015-TRACE-CHAIN-HARDENING | Core/Evaluator | Bug | Prio B: `trace_chain` Warnungen reduzieren, Kettenvalidierung verbessern. | SRC-072 | 🟠 In Analyse | 🔴 Offen | - | `MF-015-trace-chain-hardening` | - | 2026-03-06 |
| MF-019-SPOUT-WGPU-UPDATE | Engine | Refactor | Prio B: Spout auf aktuelle wgpu-Version anheben. | SRC-010 | 🔴 Offen | 🔴 Offen | - | `MF-019-spout-wgpu-update` | - | 2026-03-06 |
| MF-020-TIMELINE-KEYFRAME-INTERACTION | UI/Core | Bug | Prio B: Keyframes im Timeline-UI verschieben/loeschen wiederherstellen. | SRC-011 | 🔴 Offen | 🔴 Offen | - | `MF-020-timeline-keyframe-interaction` | - | 2026-03-06 |
| MF-003-INSPECTOR-PREVIEW-ALL-NODES | Inspector | Feature | Optionale Vorschau fuer alle Node-Typen im Inspector. | SRC-025 | 🔴 Offen | 🔴 Offen | - | `MF-003-inspector-preview-all-nodes` | - | 2026-03-06 |
| MF-006-TRIGGER-FEEDBACK-RESTORE | Module Canvas | Bug | Trigger-Feedback visuell wieder belastbar machen. | SRC-041, SRC-064 | 🟠 In Analyse | 🔴 Offen | - | `MF-006-trigger-feedback-restore` | - | 2026-03-06 |
| MF-010-HUE-CONFIG-REUSE | Control/Hue | Bug | Gespeicherte Bridge-Daten robust nutzen (Init + Reconnect + Actions). | SRC-030, SRC-068 | 🟠 In Analyse | 🔴 Offen | - | `MF-010-hue-config-reuse` | - | 2026-03-06 |
| MF-012-ANALOG-METER-OPTION | Audio/UI | Feature | Analoges Level-Meter als waehlbare Option reaktivieren. | SRC-042, SRC-070 | 🟠 In Analyse | 🔴 Offen | - | `MF-012-analog-meter-option` | - | 2026-03-06 |
| MF-013-FFMPEG-BUILDMODE-CLARITY | Media/Build | Bug | FFmpeg-Buildmodus und Fallback-Verhalten eindeutig and testbar machen. | SRC-070 | 🟢 Bereit fuer QA | 🔴 Offen | - | `main` | - | 2026-03-07 |
| MF-016-FEATURE-STATUS-BASELINE | Docs/QA | Feature | Vollstaendige Feature-Status-Uebersicht mit Dev- und User-QA-Status. | SRC-032, SRC-035, SRC-036, SRC-037 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-016-feature-status-baseline` | - | 2026-03-06 |
| MF-017-TEST-MATRIX-ROLL-OUT | QA/Process | Feature | Feste, gruppierte Testmatrix fuer Regression und Freigaben. | SRC-038, SRC-039 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-017-test-matrix-rollout` | - | 2026-03-06 |
| MF-018-SCOPE-PRIORITY-GOVERNANCE | Process | Refactor | Scope-Steuerung statt eigener "Wird das benoetigt?"-Sektion. | SRC-034 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-018-scope-priority-governance` | - | 2026-03-06 |
| MF-021-NDI-DISCOVERY-UI | UI | Feature | NDI-Discovery im Sidebar-Flow integrieren. | SRC-013 | 🔴 Offen | 🔴 Offen | - | `MF-021-ndi-discovery-ui` | - | 2026-03-06 |
| MF-022-SHADER-GRAPH-EXPANSION | Core/UI | Feature | Shader-Graph um weitere Node-Typen erweitern. | SRC-014 | 🔴 Offen | 🔴 Offen | - | `MF-022-shader-graph-expansion` | - | 2026-03-06 |
| MF-023-ERROR-TOAST-NOTIFICATIONS | UI/Core | Feature | Engine-Fehler als Toast-Notifications sichtbar machen. | SRC-016 | 🔴 Offen | 🔴 Offen | - | `MF-023-error-toast-notifications` | - | 2026-03-06 |
| MF-024-HOLD-TO-CONFIRM-UX | UX/Security | Bug | "Hold-to-Confirm" stabilisieren und CI-Failure aufloesen. | SRC-018 | 🟠 In Analyse | 🔴 Offen | - | `MF-024-hold-to-confirm-ux` | #937 | 2026-03-06 |
| MF-025-ROADMAP-GITHUB-SYNC-GATE | Process | Feature | Roadmap-Status nur mit PR/CI-Gate fortschreiben. | SRC-047, SRC-049, SRC-057 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-025-roadmap-github-sync-gate` | - | 2026-03-06 |
| MF-026-TASKID-DISPATCH-AUTO-PR | Process | Feature | Task-ID-only Delegation + AUTO_CREATE_PR als Standardprozess. | SRC-045, SRC-046, SRC-050, SRC-051, SRC-052, SRC-053, SRC-058 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-026-taskid-dispatch-auto-pr` | - | 2026-03-06 |
| MF-027-STATUS-WORKFLOW-STANDARD | Process | Feature | Statusfluss Jules -> Review -> Abnahme -> Abgeschlossen vereinheitlichen. | SRC-054, SRC-055, SRC-056 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-027-status-workflow-standard` | - | 2026-03-06 |
| MF-028-MF-ID-NAMING-COMPLIANCE | Process | Feature | MF-ID Naming Pflicht fuer Branch/PR durchsetzen. | SRC-048 | 🔵 In Umsetzung | 🔴 Offen | - | `MF-028-mf-id-naming-compliance` | - | 2026-03-06 |
| MF-033-PR-933-CONFLICT-RESOLUTION | Process | Fix | Prio A: Merge Konflikte in PR #933 auflösen. | GH-933 | 🔵 In Umsetzung | 🔴 Offen | 10832908683854701416 | `guardian-module-core-tests-14161047196552787958` | #933 | 2026-03-06 |
| MF-034-CI-FIX-BATCH | CI/CD | Fix | Prio A: Behebe pre-commit/CI Fehler in PRs #928, #930, #936, #937, #944, #946. | GH-928, GH-930, GH-936, GH-937, GH-944, GH-946 | 🔵 In Umsetzung | 🔴 Offen | 13254202037157575175 | `main` | - | 2026-03-06 |
| MF-035-AUDIO-UI-SLIDER-BOUNCE | Audio/UI | Bug | Prio A: Gain/Smoothing Regler springen bei Änderung sofort auf alten Wert zurück. | - | 🔴 Offen | 🔴 Offen | - | `MF-035-audio-ui-slider-bounce` | - | 2026-03-06 |
| MF-036-AUDIO-ANALYSIS-EXT-FEATURES | Audio/UI | Feature | Prio B: Zusätzliche visuelle Darstellungen & unabhängige L/M/H Analyseanpassung. | - | 🔴 Offen | 🔴 Offen | - | `MF-036-audio-analysis-ext-features` | - | 2026-03-06 |
| MF-037-CANVAS-AUTO-TRIGGER-BUG | Module Canvas | Bug | Prio A: Automatische Fehlverbindung Media-Out -> Trigger bei Layer-Input Connect. | - | 🔴 Offen | 🔴 Offen | - | `MF-037-canvas-auto-trigger-bug` | - | 2026-03-06 |
| MF-038-TRIGGER-FUNCTION-RECOVERY | Core/Control | Bug | Prio A: Kern-Funktion der Trigger ist defekt/funktionslos. | - | 🔴 Offen | 🔴 Offen | - | `MF-038-trigger-function-recovery` | - | 2026-03-06 |
| MF-040-SECURITY-934-VALIDATION | Security | Fix | Historischer Security-Fix, finale User-Abnahme dokumentieren. | SRC-017 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-040-security-934-validation` | #934 | 2026-03-06 |
| MF-041-CORE-TESTS-933 | Core | Test | Historischer Core-Test-Fix, Regressionstest nachziehen. | SRC-019 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-041-core-tests-933` | #933 | 2026-03-06 |
| MF-042-PERF-935 | Core/Perf | Refactor | Historische Performance-Optimierung gegen aktuelle Basis pruefen. | SRC-020 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-042-perf-935` | #935 | 2026-03-06 |
| MF-043-UI-POLISH-936 | UI | Polish | Historischer UI-Polish gegen aktuelle Panel-Realitaet gegenpruefen. | SRC-021 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-043-ui-polish-936` | #936 | 2026-03-06 |
| MF-040-SECURITY-934-VALIDATION | Security | Fix | Historischer Security-Fix, finale User-Abnahme dokumentieren. | SRC-017 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-040-security-934-validation` | #934 | 2026-03-06 |
| MF-041-CORE-TESTS-933 | Core | Test | Historischer Core-Test-Fix, Regressionstest nachziehen. | SRC-019 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-041-core-tests-933` | #933 | 2026-03-06 |
| MF-042-PERF-935 | Core/Perf | Refactor | Historische Performance-Optimierung gegen aktuelle Basis pruefen. | SRC-020 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-042-perf-935` | #935 | 2026-03-06 |
| MF-043-UI-POLISH-936 | UI | Polish | Historischer UI-Polish gegen aktuelle Panel-Realitaet gegenpruefen. | SRC-021 | 🟢 Bereit fuer QA | 🔴 Offen | - | `MF-043-ui-polish-936` | #936 | 2026-03-06 |
| MF-029-ARCHIVE-RESCUE-DELIVERABLES | Legacy/Rescue | Sammelpunkt | Historisch als erledigt markierte Rescue-Lieferungen (technisch erneut validieren ueber MF-016/MF-017). | SRC-001, SRC-002, SRC-003, SRC-005, SRC-006, SRC-007, SRC-008, SRC-012, SRC-015 | ✅ Abgeschlossen | 🔴 Offen | - | - | - | 2026-03-06 |

## No-Loss-Trace (kompakt)

| Pruefung | Ergebnis |
| --- | --- |
| Quellpunkte (`SRC-###`) gesamt | 72 |
| Gemappte Quellpunkte | 72 |
| Quellpunkte ohne MF-Zuordnung | 0 |
| MF-Tasks ohne Quellreferenz | 0 |
| Detail-Nachverfolgung | `docs/project/TEST_MATRIX.md` (`Roh-Inventar` + `Normalisierte Zuordnung`) |

Warum kein grosses Migrations-Mapping in der Roadmap:
- Damit `ROADMAP.md` steuerbar bleibt und nicht aufgeblaeht wird.
- Die verlustfreie 1:1-Quelle bleibt in `docs/project/TEST_MATRIX.md` erhalten.
