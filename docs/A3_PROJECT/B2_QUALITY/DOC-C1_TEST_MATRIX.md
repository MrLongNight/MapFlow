# TEST_MATRIX (No-Loss)

Stand: 2026-03-06
Bezug: GitHub Project Issues (MF-ID gesteuert)

## Roh-Inventar (unveraendert)

Alle Quellpunkte bleiben als `SRC-###` erhalten.

| SRC-ID | Quelle | Originalpunkt (unveraendert) |
| --- | --- | --- |
| SRC-001 | ROADMAP_alt | UI-Architektur: Komplette Modularisierung von MenuBar und Inspector. |
| SRC-002 | ROADMAP_alt | UI-Stabilitaet: Fix von Layout-Deadlocks durch separate Toolbar-Orchestrierung in ui_layout.rs. |
| SRC-003 | ROADMAP_alt | Canvas Node Graph: Verbindungen funktionieren wieder einwandfrei (Radius: 30px). |
| SRC-004 | ROADMAP_alt | Audio-Analyse: Echtzeit-Sync zwischen Engine und UI repariert; Peak-Decay fuer Level-Meter implementiert. |
| SRC-005 | ROADMAP_alt | Show Automation: Timeline um Modi Fully Auto, Semi Auto und Manual erweitert. |
| SRC-006 | ROADMAP_alt | HAP Video Engine: HAP Q Alpha Support implementiert und Syntaxfehler behoben. |
| SRC-007 | ROADMAP_alt | Settings & About: Dialoge vollstaendig rekonstruiert und integriert. |
| SRC-008 | ROADMAP_alt | CI/CD: Job01 Validation Fehler (toolchain input & hap_decoder syntax) behoben. |
| SRC-009 | ROADMAP_alt | Verbleibender Blocker: Hue-Stabilitaet (HueFlow/DTLS). |
| SRC-010 | ROADMAP_alt | Verbleibender Blocker: Spout Support Update auf aktuelle wgpu-Version. |
| SRC-011 | ROADMAP_alt | Verbleibender Blocker: Timeline Interaktion (Keyframes verschieben/loeschen). |
| SRC-012 | ROADMAP_alt | Vollstaendiges Splitting der God-Files: menu_bar.rs und inspector/mod.rs erfolgreich zerlegt. |
| SRC-013 | ROADMAP_alt | NDI-Discovery UI: Integration der Quellensuche im Sidebar-Tab. |
| SRC-014 | ROADMAP_alt | Shader-Graph Expansion: Weitere Node-Typen (Math, Noise, Filter). |
| SRC-015 | ROADMAP_alt | Testing: 100% Passrate erreicht (400+ Tests). |
| SRC-016 | ROADMAP_alt | Fehler-Handling: Toast-Notifications fuer Engine-Fehler steht noch aus. |
| SRC-017 | ROADMAP_alt | Sicherheit (#934): Validierung der Pfad-Traversierung - Ready for Merge. |
| SRC-018 | ROADMAP_alt | UX-Sicherheit (#937): Hold-to-Confirm - CI failing (pre-commit). |
| SRC-019 | ROADMAP_alt | Core-Tests (#933): Unit-Tests fuer ModuleManager & Kernlogik - Ready for Merge. |
| SRC-020 | ROADMAP_alt | Performance (#935): VecDeque-Optimierung fuer History-Stack - Ready for Merge. |
| SRC-021 | ROADMAP_alt | UI-Polishing (#936): Muted Styling fuer leere Zustaende - Ready for Merge. |
| SRC-022 | User_2026-03-06 | Audio Analyse wird doppelt angezeigt. |
| SRC-023 | User_2026-03-06 | Schwebendes Audio Analyse Panel muss entfernt werden. |
| SRC-024 | User_2026-03-06 | Audioanalyse funktioniert immer noch nicht wieder. |
| SRC-025 | User_2026-03-06 | Implementiere in allen Nodes eine optionale Vorschau im Inspector. |
| SRC-026 | User_2026-03-06 | Stelle sicher, dass man nur typen-gleiche Ein- und Ausgaenge verbinden kann. |
| SRC-027 | User_2026-03-06 | Grafik & Performance in Settings ist nicht funktionsfaehig. |
| SRC-028 | User_2026-03-06 | Theme Switcher inkl. Auswahlmenue unterschiedlicher Themes fehlt. |
| SRC-029 | User_2026-03-06 | Fixe ERROR mapflow_control::hue::controller: Failed to connect to Hue Bridge DTLS... OpenSSL not compiled in. |
| SRC-030 | User_2026-03-06 | Stelle sicher, dass bereits gespeicherte Bridge Verbindungsdaten verwendet werden. |
| SRC-031 | User_2026-03-06 | Mesh Editor in der Layer Node ist nicht verfuegbar. |
| SRC-032 | User_2026-03-06 | Erfasse alle Fehler/Probleme in der Roadmap inkl. Feld fuer Dev-Fix und Feld fuer User-Testbestaetigung. |
| SRC-033 | User_2026-03-06 | ERROR mapflow_control::hue::controller ... OpenSSL support is not compiled in (erneut genannt). |
| SRC-034 | User_2026-03-06 | Pruefe, ob "Wird das wirklich benoetigt?" eine sinnvolle Funktion hat oder weg kann. |
| SRC-035 | User_2026-03-06 | Feature-Status-Uebersicht ist nicht vollstaendig. |
| SRC-036 | User_2026-03-06 | Viele Punkte haben Haken, funktionieren aber in Wirklichkeit nicht. |
| SRC-037 | User_2026-03-06 | Erstelle vollstaendige Liste aller Features mit Statusfeld fuer Dev und Statusfeld fuer User. |
| SRC-038 | User_2026-03-06 | Liste logisch sortieren und gruppieren. |
| SRC-039 | User_2026-03-06 | Liste als Grundlage fuer festes Test- und Pruefszenario nach grossen Aenderungen verwenden. |
| SRC-040 | User_2026-03-06 | In Layer Node ist der grafische Mesh Editor im Inspector nicht mehr verfuegbar. |
| SRC-041 | User_2026-03-06 | Es fehlen diverse bereits funktionierende Features wie grafisches Feedback bei Trigger Nodes. |
| SRC-042 | User_2026-03-06 | Analoges Level Meter nicht mehr verfuegbar. |
| SRC-043 | User_2026-03-06 | Einige UI Panels werden gar nicht, an falscher Stelle oder nicht mit allen Funktionen angezeigt. |
| SRC-044 | User_2026-03-06 | Design Themes und Umschaltfunktion fehlen. |
| SRC-045 | walkthrough.md | TechLead passt nur Task-ID an jules_dispatcher (keine redundanten Tasktexte). |
| SRC-046 | walkthrough.md | automationMode: AUTO_CREATE_PR ist verpflichtend. |
| SRC-047 | walkthrough.md | GitHub Project Issues spiegeln den PR-Status. |
| SRC-048 | walkthrough.md | Einheits-Schema MF-[NR]-[SLUG], Branch-Namen und PR-Prefix strikt nach ID. |
| SRC-049 | walkthrough.md | pr_branch_manager bestaetigt gruene Checks vor Statuswechsel auf Abnahme erforderlich. |
| SRC-050 | walkthrough.md | Architect markiert delegierbare Tasks mit Roadmap Task-ID. |
| SRC-051 | roadmap_proposal.md | Identifikation: TechLead waehlt TASK_ID aus ROADMAP. |
| SRC-052 | roadmap_proposal.md | Context-Lookup: Dispatcher liest Task-Details direkt aus ROADMAP. |
| SRC-053 | roadmap_proposal.md | Jules-Start mit AUTO_CREATE_PR und Task-ID als Arbeitsanweisung. |
| SRC-054 | roadmap_proposal.md | Tracking-Workflow: Jules -> Review -> Abnahme -> Abgeschlossen. |
| SRC-055 | roadmap_proposal.md | ROADMAP-Struktur mit Task-ID, Bereich, Typ, Status, Beschreibung, Session ID, PR, Branch. |
| SRC-056 | roadmap_proposal.md | Statusschluessel inkl. Nacharbeit. |
| SRC-057 | roadmap_proposal.md | Feedback- und Historien-Log fuer Nacharbeiten. |
| SRC-058 | roadmap_proposal.md | Token-Effizienz: keine Wiederholung kompletter Taskbeschreibungen in Delegationsprompts. |
| SRC-059 | Codeanalyse_2026-03-06 | Doppelte Audio-Analyse Renderpfade in ui_layout.rs (Sidebar + Floating Window). |
| SRC-060 | Codeanalyse_2026-03-06 | UIAction SelectAudioDevice/UpdateAudioConfig/ToggleAudioPanel in mapflow-ui definiert, in app/actions nicht behandelt. |
| SRC-061 | Codeanalyse_2026-03-06 | audio_backend vorhanden, aber get_samples/process_samples nicht im Hauptloop verdrahtet. |
| SRC-062 | Codeanalyse_2026-03-06 | module_evaluator.update_audio existiert, wird im App-Loop nicht aufgerufen. |
| SRC-063 | Codeanalyse_2026-03-06 | Connection-Drop in module_canvas/renderer.rs ohne harte Socket-Typpruefung. |
| SRC-064 | Codeanalyse_2026-03-06 | last_trigger_values wird fuer Visualisierung gelesen, aber nicht aus Eval-Ergebnis befuellt. |
| SRC-065 | Codeanalyse_2026-03-06 | Settings Graphics/Performance verwendet lokale Dummy-Werte ohne Persistenz/Wirkung. |
| SRC-066 | Codeanalyse_2026-03-06 | Theme-System vorhanden (core/theme.rs), aber Theme-Switcher nicht integriert. |
| SRC-067 | Codeanalyse_2026-03-06 | Hue DTLS in dtls.rs als OpenSSL-disabled Stub. |
| SRC-068 | Codeanalyse_2026-03-06 | Hue-Konfig wird aus user_config geladen; Auto-Connect nur bedingt robust. |
| SRC-069 | Codeanalyse_2026-03-06 | Panel-Flags und Renderpfade inkonsistent (OSC/Preview/Control/Assignment). |
| SRC-070 | Loganalyse_2026-03-06 | FFmpeg Feature nicht aktiviert / Decoder-Fallback-Warnungen in Logs. |
| SRC-071 | Loganalyse_2026-03-06 | Autosave-Load-Fehler: missing field `master_blackout` in Composition. |
| SRC-072 | Loganalyse_2026-03-06 | Wiederkehrende trace_chain Warnungen: Node not found in part_index. |

### Normalisierte Zuordnung

| Quellbereich | Feature-ID | Zugehoerige MF-Task(s) |
| --- | --- | --- |
| SRC-001..003 | F-029 | MF-029 |
| SRC-004 | F-002 | MF-002 |
| SRC-005..008 | F-029 | MF-029 |
| SRC-009, SRC-029, SRC-033, SRC-067 | F-009 | MF-009 |
| SRC-010 | F-019 | MF-019 |
| SRC-011 | F-020 | MF-020 |
| SRC-012, SRC-015 | F-029 | MF-029 |
| SRC-013 | F-021 | MF-021 |
| SRC-014 | F-022 | MF-022 |
| SRC-016 | F-023 | MF-023 |
| SRC-017 | F-030 | MF-040 |
| SRC-018 | F-024 | MF-024 |
| SRC-019 | F-031 | MF-041 |
| SRC-020 | F-028 | MF-042 |
| SRC-021 | F-032 | MF-043 |
| SRC-022, SRC-023, SRC-059 | F-001 | MF-001 |
| SRC-024, SRC-060..062 | F-002 | MF-002 |
| SRC-025 | F-003 | MF-003 |
| SRC-026, SRC-063 | F-004 | MF-004 |
| SRC-027, SRC-065 | F-007 | MF-007 |
| SRC-028, SRC-044, SRC-066 | F-008 | MF-008 |
| SRC-030, SRC-068 | F-010 | MF-010 |
| SRC-031, SRC-040 | F-005 | MF-005 |
| SRC-032, SRC-035..037 | F-016 | MF-016 |
| SRC-034, SRC-048 | F-018 | MF-018, MF-028 |
| SRC-038, SRC-039 | F-017 | MF-017 |
| SRC-041, SRC-064 | F-006 | MF-006 |
| SRC-042 | F-012 | MF-012 |
| SRC-043, SRC-069 | F-011 | MF-011 |
| SRC-045, SRC-046, SRC-050..053, SRC-058 | F-025 | MF-026 |
| SRC-047, SRC-049, SRC-057 | F-026 | MF-025 |
| SRC-054..056 | F-027 | MF-027 |
| SRC-070 | F-013 | MF-013 |
| SRC-071 | F-014 | MF-014 |
| SRC-072 | F-015 | MF-015 |

## Gruppierte Feature-/Testmatrix

Spaltenstandard:
`Feature-ID | Zugehoerige MF-Task(s) | Feature | Preconditions | Testschritte | Erwartet | Dev-Status | QA-Status (User) | Nacharbeit`

| Feature-ID | Zugehoerige MF-Task(s) | Feature | Preconditions | Testschritte | Erwartet | Dev-Status | QA-Status (User) | Nacharbeit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F-001 | MF-001 | Audio UI Konsolidierung | App startet mit aktivem Dashboard | 1. Audio-Sektion im Sidebar oeffnen. 2. Audio-Fenster-Trigger betaetigen. 3. UI-Elemente zaehlen. | Genau ein Audio-Analyse-Panel sichtbar. | 🟠 In Analyse | 🔴 Offen | - |
| F-002 | MF-002 | Audio Pipeline Recovery | Audio-Input Device vorhanden | 1. Device waehlen. 2. Signal einspeisen. 3. Analyzer/Trigger beobachten. | Pegel/Baender/Trigger aktualisieren live ohne manuelles Refresh. | 🟠 In Analyse | 🔴 Offen | - |
| F-003 | MF-003 | Inspector Vorschau fuer alle Nodes | Module Canvas geoeffnet | 1. Mehrere Node-Typen selektieren. 2. Inspector pruefen. | Optionale Vorschau fuer jeden Node-Typ verfuegbar. | 🔴 Offen | 🔴 Offen | - |
| F-004 | MF-004 | Typgleiche Verbindungen | Zwei Nodes mit unterschiedlichen Socket-Typen | 1. Ungleichartige Verbindung ziehen. 2. Gleichartige Verbindung ziehen. | Ungleichartige Verbindung blockiert, gleichartige erfolgreich. | 🟠 In Analyse | 🔴 Offen | - |
| F-005 | MF-005 | Layer Mesh Editor im Inspector | Layer Node vorhanden | 1. Layer Node selektieren. 2. Inspector auf Mesh-Bereich pruefen. 3. Punkte bewegen. | Grafischer Mesh-Editor sichtbar und editierbar. | 🟠 In Analyse | 🔴 Offen | - |
| F-006 | MF-006 | Trigger Feedback Visualisierung | Trigger-Node mit eingehendem Signal | 1. Trigger aktivieren. 2. Node-Glow/Output-Meter beobachten. | Visuelles Feedback folgt Triggerwerten nachvollziehbar. | 🟠 In Analyse | 🔴 Offen | - |
| F-007 | MF-007 | Settings Grafik/Performance | Settings Dialog offen | 1. FPS/Quality aendern. 2. App neu starten. 3. Runtime-Wirkung pruefen. | Werte persistieren und wirken im Runtime-Verhalten. | 🟠 In Analyse | 🔴 Offen | - |
| F-008 | MF-008 | Theme Switcher + Auswahlmenue | Settings Dialog offen | 1. Theme wechseln. 2. Neustart. 3. UI-Farben kontrollieren. | Ausgewaehltes Theme wirkt sofort und bleibt gespeichert. | 🟠 In Analyse | 🔴 Offen | - |
| F-009 | MF-009 | Hue DTLS/OpenSSL Pfad | Hue Bridge vorhanden | 1. Hue verbinden. 2. Streaming starten. 3. Logs pruefen. | Kein OpenSSL-disabled Fehler; Verbindung stabil. | 🟠 In Analyse | 🔴 Offen | - |
| F-010 | MF-010 | Hue Config Reuse | Gespeicherte Hue-Daten vorhanden | 1. App neu starten. 2. Hue-Status pruefen. 3. Reconnect testen. | Gespeicherte Bridge-Daten werden ohne Neu-Einrichtung genutzt. | 🟠 In Analyse | 🔴 Offen | - |
| F-011 | MF-011 | UI Panel Konsistenz | View-Menue verfuegbar | 1. Panel-Flags toggeln. 2. Sichtbarkeit/Funktion pro Panel pruefen. | Alle Panels erscheinen korrekt und voll funktionsfaehig. | 🟠 In Analyse | 🔴 Offen | - |
| F-012 | MF-012 | Analoges Level Meter | Audio aktiv | 1. Meter-Style in Settings wechseln. 2. Toolbar beobachten. | Analog- und Digital-Meter auswaehlbar und sichtbar. | 🟠 In Analyse | 🔴 Offen | - |
| F-013 | MF-013 | FFmpeg Buildmode Klarheit | Videoquelle geladen | 1. Build ohne/mit ffmpeg laufen lassen. 2. Logs kontrollieren. | Verhalten eindeutig; Warnungen nur erwartbar und dokumentiert. | 🟠 In Analyse | 🔴 Offen | - |
| F-014 | MF-014 | Autosave Kompatibilitaet | Altes Autosave-File vorhanden | 1. App mit altem Autosave starten. 2. Ladeprozess beobachten. | Kein Deserialisierungsfehler fuer `master_blackout`. | 🟠 In Analyse | 🔴 Offen | - |
| F-015 | MF-015 | trace_chain Hardening | Modulgraph mit Grenzfaellen | 1. Graph evaluieren. 2. Logs auf trace_chain Warnungen pruefen. | Warnungen reduziert; verbleibende Meldungen sind diagnostisch klar. | 🟠 In Analyse | 🔴 Offen | - |
| F-016 | MF-016 | Feature-Status Baseline | ROADMAP + Matrix vorhanden | 1. Alle Features auflisten. 2. Dev/QA Status je Feature setzen. | Keine unklaren Haken; Doppelstatus vollstaendig gepflegt. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-017 | MF-017 | Regression Testmatrix Rollout | Matrix gepflegt | 1. Standard-Testlauf nach grosser Aenderung ausfuehren. 2. Ergebnisse dokumentieren. | Vollstaendiger Regressionslauf reproduzierbar. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-018 | MF-018, MF-028 | Scope/Prioritaet + MF-ID Compliance | Neue Tasks in Planung | 1. Neue Task anlegen. 2. Prioritaet/Scope setzen. 3. Branch/PR Name pruefen. | Keine Extra-Sektion noetig; MF-ID Regel eingehalten. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-019 | MF-019 | Spout Update | Spout Output aktiv | 1. Spout aktivieren. 2. Ausgabe in Empfaenger pruefen. | Spout kompatibel mit aktueller wgpu-Basis. | 🔴 Offen | 🔴 Offen | - |
| F-020 | MF-020 | Timeline Keyframe Interaction | Timeline mit Keyframes | 1. Keyframe verschieben. 2. Keyframe loeschen. | Beide Aktionen funktionieren stabil. | 🔴 Offen | 🔴 Offen | - |
| F-021 | MF-021 | NDI Discovery UI | NDI Netzwerk verfuegbar | 1. Discovery starten. 2. Quelle im Sidebar waehlen. | NDI Quellen werden gefunden und auswaehlbar. | 🔴 Offen | 🔴 Offen | - |
| F-022 | MF-022 | Shader Graph Expansion | Shader-Editor offen | 1. Neue Node-Typen hinzufuegen. 2. Verbindungen testen. | Neue Node-Typen verfuegbar und nutzbar. | 🔴 Offen | 🔴 Offen | - |
| F-023 | MF-023 | Error Toast Notifications | Fehler provozierbar | 1. Fehlerfall ausloesen. 2. Toast beobachten. | Fehler sichtbar als Toast mit brauchbarer Meldung. | 🔴 Offen | 🔴 Offen | - |
| F-024 | MF-024 | Hold-to-Confirm UX | Kritische Aktion vorhanden | 1. Button kurz klicken. 2. Button lange halten. | Nur langes Halten fuehrt Aktion aus; CI gruen. | 🟠 In Analyse | 🔴 Offen | - |
| F-025 | MF-026 | Task-ID Dispatch Auto PR | Maestro/Jules Prozess aktiv | 1. Task-ID uebergeben. 2. Dispatcher-Aufruf pruefen. | Task-Details aus ROADMAP; PR auto erstellt. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-026 | MF-025 | Roadmap/GitHub Sync Gate | PR vorhanden | 1. CI-Status pruefen. 2. Roadmap-Status setzen. | Statuswechsel nur nach gruener CI und Gate-Check. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-027 | MF-027 | Status Workflow Standard | Task-Lebenszyklus vorhanden | 1. Statusweg durchlaufen. 2. Dokumentierte Uebergaenge pruefen. | Einheitlicher Statusfluss ohne Luecken. | 🔵 In Umsetzung | 🔴 Offen | - |
| F-028 | MF-042 | Legacy Performance Validation | Historische Perf-Optimierung vorhanden | 1. Benchmark/Baseline vergleichen. | Historischer Fix weiterhin wirksam. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| F-029 | MF-029 | Legacy Rescue Deliverables | Historische Punkte dokumentiert | 1. Stichproben aus archivierten Punkten verifizieren. | Historische "done"-Aussagen sind nachvollziehbar oder als Regression markiert. | ✅ Abgeschlossen | 🔴 Offen | - |
| F-030 | MF-040 | Security #934 Validation | Sicherheitsfix vorhanden | 1. Security-Regressionstest laufen lassen. | Keine erneute Path-Traversal-Luecke. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| F-031 | MF-041 | Core Tests #933 Validation | Testsuite verfuegbar | 1. Betroffene Unit-Tests ausfuehren. | Tests bleiben stabil gruen. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| F-032 | MF-043 | UI Polish #936 Validation | UI-Ansichten vorhanden | 1. Leere Zustaende aufrufen. 2. Styling pruefen. | Muted Styling unveraendert konsistent. | 🟢 Bereit fuer QA | 🔴 Offen | - |

## Testfaelle mit festen Schritten und Erwartung

| TC-ID | Feature-ID | Zugehoerige MF-Task(s) | Testschritte | Erwartet | Dev-Status | QA-Status (User) | Nacharbeit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TC-001 | F-001 | MF-001 | App starten -> Sidebar Audio oeffnen -> Audio-Window Toggle betaetigen -> Anzahl Panels pruefen. | Maximal ein Audio-Analyse-Panel. | 🟠 In Analyse | 🔴 Offen | - |
| TC-002 | F-002 | MF-002 | Audio-Device waehlen -> Musik einspeisen -> Analyzer/Meter/Trigger pruefen. | Audiofluss aktiv bis UI sichtbar. | 🟠 In Analyse | 🔴 Offen | - |
| TC-031 | F-003 | MF-003 | Mehrere Node-Typen selektieren -> Inspector auf Preview-Option pruefen. | Vorschau-Option bei allen Node-Typen vorhanden. | 🔴 Offen | 🔴 Offen | - |
| TC-003 | F-004 | MF-004 | Unpassende Socket-Typen verbinden -> passende verbinden. | Nur passende Verbindung wird angelegt. | 🟠 In Analyse | 🔴 Offen | - |
| TC-004 | F-005 | MF-005 | Layer Node selektieren -> Inspector Mesh UI nutzen. | Mesh-Punkte lassen sich editieren/speichern. | 🟠 In Analyse | 🔴 Offen | - |
| TC-032 | F-006 | MF-006 | Trigger aktivieren -> Node-Glow/Output-Meter live beobachten. | Trigger-Feedback folgt Signalwerten stabil. | 🟠 In Analyse | 🔴 Offen | - |
| TC-005 | F-007 | MF-007 | FPS/Quality aendern -> Neustart -> Wirkung validieren. | Werte persistent und wirksam. | 🟠 In Analyse | 🔴 Offen | - |
| TC-006 | F-008 | MF-008 | Theme wechseln -> Neustart -> Theme bleibt aktiv. | Theme-Switch inkl Menue funktioniert. | 🟠 In Analyse | 🔴 Offen | - |
| TC-007 | F-009 | MF-009 | Hue verbinden -> Stream starten -> Logs beobachten. | Kein OpenSSL-disabled Fehler. | 🟠 In Analyse | 🔴 Offen | - |
| TC-008 | F-010 | MF-010 | Gespeicherte Hue-Daten laden -> Reconnect ohne Neueingabe. | Bridge-Daten werden wiederverwendet. | 🟠 In Analyse | 🔴 Offen | - |
| TC-009 | F-011 | MF-011 | View-Menue Panels toggeln -> Position/Funktionen pruefen. | Flags und Renderpfade konsistent. | 🟠 In Analyse | 🔴 Offen | - |
| TC-010 | F-012 | MF-012 | Meter-Style wechseln -> Toolbar vergleichen. | Analog/Digital umschaltbar. | 🟠 In Analyse | 🔴 Offen | - |
| TC-011 | F-013 | MF-013 | Build ohne ffmpeg -> mit ffmpeg -> Logvergleich. | Verhalten klar, keine irrefuehrenden Warnungen. | 🟠 In Analyse | 🔴 Offen | - |
| TC-012 | F-014 | MF-014 | Altes Autosave laden. | Kein missing-field Fehler. | 🟠 In Analyse | 🔴 Offen | - |
| TC-013 | F-015 | MF-015 | Problemgraph evaluieren -> Logs pruefen. | trace_chain Warnungen reduziert/verbessert. | 🟠 In Analyse | 🔴 Offen | - |
| TC-014 | F-016 | MF-016 | Featureliste und Doppelstatus vollstaendig ausfuellen. | Kein Feature ohne Dev+QA Status. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-015 | F-017 | MF-017 | Standard-Regressionstestsatz komplett laufen. | Reproduzierbare Pruefung ohne Luecken. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-016 | F-018 | MF-018, MF-028 | Neue Task anlegen und Branch/PR Benennung pruefen. | MF-ID Regel wird strikt eingehalten. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-017 | F-019 | MF-019 | Spout Output gegen Empfaenger pruefen. | Kompatible Ausgabe ohne Crash. | 🔴 Offen | 🔴 Offen | - |
| TC-018 | F-020 | MF-020 | Keyframe verschieben/loeschen. | Timeline Interaktion wieder funktionsfaehig. | 🔴 Offen | 🔴 Offen | - |
| TC-019 | F-021 | MF-021 | NDI Discovery starten und Quelle waehlen. | Quellen sichtbar, Auswahl moeglich. | 🔴 Offen | 🔴 Offen | - |
| TC-020 | F-022 | MF-022 | Neue Shader-Node-Typen einfuegen und verbinden. | Nodes funktionieren wie spezifiziert. | 🔴 Offen | 🔴 Offen | - |
| TC-021 | F-023 | MF-023 | Fehler simulieren -> Toast pruefen. | Fehlermeldung sichtbar und nutzbar. | 🔴 Offen | 🔴 Offen | - |
| TC-022 | F-024 | MF-024 | Hold-to-confirm kurz/long press pruefen. | Nur long press triggert Aktion. | 🟠 In Analyse | 🔴 Offen | - |
| TC-023 | F-025 | MF-026 | Task-ID Delegation ausfuehren, PR-Erstellung pruefen. | AUTO_CREATE_PR Ablauf korrekt. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-024 | F-026 | MF-025 | CI gruen setzen -> Status in Roadmap aktualisieren. | Status-Gate eingehalten. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-025 | F-027 | MF-027 | Statuswechsel entlang Standardworkflow testen. | Sauberer Workflow ohne Sonderfaelle. | 🔵 In Umsetzung | 🔴 Offen | - |
| TC-026 | F-028 | MF-042 | Historische Perf-Optimierung gegentesten. | Keine Regression gegen dokumentierten Fix. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| TC-027 | F-029 | MF-029 | Archivierte Punkte stichprobenartig verifizieren. | Historische Aussagen nachvollziehbar. | ✅ Abgeschlossen | 🔴 Offen | - |
| TC-028 | F-030 | MF-040 | Security Regressionstest ausfuehren. | Pfad-Traversal weiterhin geblockt. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| TC-029 | F-031 | MF-041 | Kern-Testpaket #933 ausfuehren. | Tests bleiben gruen. | 🟢 Bereit fuer QA | 🔴 Offen | - |
| TC-030 | F-032 | MF-043 | UI-Leerzustaende pruefen. | Polishing bleibt intakt. | 🟢 Bereit fuer QA | 🔴 Offen | - |

## Traceability pro Zeile

- Jede Roh-Inventar-Zeile (`SRC-###`) ist ueber `Normalisierte Zuordnung` mindestens einer MF-Task zugeordnet.
- Jede Feature-Zeile (`F-###`) ist auf mindestens eine MF-Task verlinkt.
- Jeder aktive MF-Task ist mindestens einem Testfall (`TC-###`) zugeordnet.

### Vollstaendigkeitscheck

| Pruefung | Ergebnis |
| --- | --- |
| Roh-Inventar Zeilen | 72 |
| Gemappte Quellzeilen in ROADMAP | 72 |
| Quellzeilen ohne Feature/MF Bezug | 0 |
| MF-Tasks ohne Testfall | 0 |
| Orphan-Points | 0 |
