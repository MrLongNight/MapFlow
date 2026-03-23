# DOC-C14: Task Briefs fuer Media Decoder, Undo/Redo und NDI

## Status
- Status: Proposed
- Zweck: umsetzungsnahe Detailvorgaben fuer `MF-059`, `MF-056` und `MF-021`
- Ergaenzt:
  - GitHub Project Issues
  - `DOC-C11_VIDEO_IO_STRATEGY.md`

## 1. Einsatz dieses Dokuments

Dieses Dokument ist kein Ersatz fuer die Roadmap. Es dient als technischer Ausfuehrungsbrief fuer Tasks, die bereits in der Roadmap existieren, dort aber noch nicht praezise genug fuer eine saubere Delegation an Subagents beschrieben sind.

Ziel ist:
- Scope verengen
- eine klare technische Richtung festlegen
- DoD und Validierung vorgeben
- unnoetige Parallel-Interpretationen vermeiden

## 2. Gemeinsame Guardrails

- `QA-Status` in den GitHub Project Issues nicht aendern.
- Keine grossen Neben-Refactors ohne direkten Bezug zu `MF-059`, `MF-056` oder `MF-021`.
- Doku und Code muessen am Ende denselben Status ausdruecken.
- Wenn ein Task nur teilweise geschlossen werden kann, muessen Restblocker explizit dokumentiert werden.
- Bestehende funktionierende Pfade duerfen nicht fuer neue Architektur-Ideen geopfert werden.

## 3. MF-059-MEDIA-DECODER-STABILITY

### Problem

Bei VP8/VP9 tritt im Medienpfad der Fehler `Decoder error: Input changed` auf. Der Fehler ist besonders kritisch, weil er in einem Kernpfad fuer Playback und Vorschau liegt und damit reale Show-Workflows destabilisiert.

### Beobachteter Ist-Zustand

- In `crates/mapflow-media/src/decoder.rs` existiert der konkrete Fehlerpfad:
  - `Decoder error: Input changed? Scaler run failed: {}`
- Der `seek()`-Pfad flusht den Decoder bereits, aber es gibt aktuell keinen belastbaren Reinit-Pfad fuer geaenderte Frame- oder Scaler-Parameter.
- Der Task ist auf Stabilisierung des bestehenden Decoder-Pfads ausgerichtet, nicht auf neue Codec-Features.

### Ziel

- VP8/VP9 duerfen bei typischen Parameter- oder Zustandswechseln nicht in einen dauerhaft fehlerhaften Zustand laufen.
- Der Decoder soll bei geaenderten Frame-Eigenschaften kontrolliert reinitialisieren oder klar fallbacken.
- Bereits funktionierende Pfade fuer andere Codecs duerfen nicht regressieren.

### Nicht-Ziele

- kein neuer Codec-Support
- kein Rewrite des gesamten Decoder-Subsystems
- keine macOS- oder `VideoToolbox`-Arbeit in diesem Task
- keine Aenderung des generellen FFmpeg-Buildmodus ausser wenn zur Fehlerbehebung zwingend noetig

### Primaere Dateien

- `crates/mapflow-media/src/decoder.rs`
- bei Bedarf angrenzende Playback-/Reload-Aufrufer im App-Medienpfad

### Empfohlene Umsetzung

1. Reproduzierbaren Fehlerfall festziehen.
   - VP8/VP9-Material mit Seek, Reload oder Parameteraenderung gezielt testen.
   - Vor dem eigentlichen Fix die ausloesenden Uebergaenge protokollieren: Breite, Hoehe, Pixel-Format, PTS/DTS, Seek/Flush-Reihenfolge.

2. Decoder- und Scaler-Zustaende trennen.
   - Die Logik fuer "Frame-Eigenschaften haben sich geaendert" als explizite Pruefung modellieren.
   - Nicht erst im Fehlerfall implizit auf `sws_scale` verlassen.

3. Sauberen Reinit-Pfad implementieren.
   - Wenn Eingangs-Frame-Eigenschaften vom aktiven Scaler abweichen, Scaler kontrolliert neu aufbauen.
   - Wenn der Codec-Kontext selbst inkonsistent wird, einen klar begrenzten Decoder-Reinit mit erhaltener Fehlerdiagnostik vorsehen.
   - Nach `seek()` oder Reload alte Frames nicht weiterverwenden.

4. Fallback-Verhalten definieren.
   - Wenn Reinit nicht moeglich ist, muss der Fehler klar und einmalig surfaced werden.
   - Endlosschleifen mit wiederholtem "Input changed"-Spam sind nicht akzeptabel.

5. Testbarkeit einbauen.
   - Reinit-Entscheidungen moeglichst in kleine Helper auslagern und gezielt testen.
   - Wenn kein belastbares Test-Asset im Repo vorhanden ist, mindestens die Reinit-Logik automatisiert absichern und den manuellen Repro-Fall dokumentieren.

### Definition of Done

- Der bekannte VP8/VP9-Fehler ist reproduzierbar behoben oder hart eingegrenzt.
- Der Decoder reagiert auf geaenderte Eingangsparameter kontrolliert.
- Andere gaengige Medienpfade verhalten sich unveraendert stabil.
- Es gibt mindestens einen nachvollziehbaren Test- oder Repro-Nachweis.

### Validierung

- VP8- und VP9-Datei laden.
- Seek ausloesen.
- relevante Medienparameter aendern, sofern der bestehende UI-/Playback-Pfad dies ausloest.
- pruefen, dass kein dauerhafter Decoder-Abbruch entsteht.
- Smoke-Test mit mindestens einem bislang stabilen Nicht-VP8/VP9-Format.

## 4. MF-056-UNDO-REDO-PARAMS

### Problem

Aktuell existieren mehrere Undo/Redo-Ansatzpunkte parallel, aber der im Module-Canvas tatsaechlich aktive Pfad deckt im Wesentlichen nur Struktur- und Positionsaenderungen ab. Parameteraenderungen aus den Inspector-Panels sind dadurch nicht konsistent rueckgaengig machbar.

### Beobachteter Ist-Zustand

- `crates/mapflow-core/src/history.rs` verwaltet `AppState`-Snapshots.
- `crates/mapflow-ui/src/core/undo_redo.rs` enthaelt ein separates Command-System.
- Der real genutzte Module-Canvas-Pfad arbeitet ueber `ModuleCanvas.undo_stack` / `redo_stack`.
- `CanvasAction` in `crates/mapflow-ui/src/editors/module_canvas/types.rs` kennt derzeit vor allem:
  - `AddPart`
  - `DeletePart`
  - `MovePart`
  - `AddConnection`
  - `DeleteConnection`
  - `Batch`
- Viele Inspector-Aenderungen in `source.rs`, `effect.rs`, `layer.rs`, `output.rs` und `trigger.rs` mutieren Daten direkt, ohne Undo-Eintrag.

### Ziel

- Parameteraenderungen im Module-Canvas-Inspector muessen undo-/redo-faehig werden.
- Bestehende Struktur-Undo-Pfade duerfen dabei nicht brechen.
- Ein einzelner Drag oder Edit-Flow darf nicht dutzende History-Eintraege erzeugen.

### Leitentscheidung

Fuer `MF-056` ist der aktive Module-Canvas-History-Pfad die kanonische Implementierungsbasis.

Das heisst konkret:
- `CanvasAction` wird erweitert.
- Kein app-weites Undo/Redo-Rewrite in diesem Task.
- `mapflow-core/src/history.rs` und `mapflow-ui/src/core/undo_redo.rs` werden hoechstens als Referenz betrachtet, aber nicht als Pflicht-Migrationsziel fuer diesen Task.

### Nicht-Ziele

- kein globales Undo/Redo fuer alle App-Bereiche ausserhalb des Module Canvas
- keine komplette Vereinheitlichung aller vorhandenen History-Systeme
- kein grosser UI-Refactor ohne direkten Undo/Redo-Nutzen

### Primaere Dateien

- `crates/mapflow-ui/src/editors/module_canvas/types.rs`
- `crates/mapflow-ui/src/editors/module_canvas/state.rs`
- `crates/mapflow-ui/src/editors/module_canvas/controller.rs`
- `crates/mapflow-ui/src/editors/module_canvas/renderer.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/source.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/effect.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/layer.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/output.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/trigger.rs`

### Empfohlene Umsetzung

1. Generische Parameter-Aktion einfuehren.
   - `CanvasAction` um eine Snapshot-basierte Variante erweitern, bevorzugt auf voller Part-Ebene:
     - Beispiel: `UpdatePart { part_id, before, after }`
   - Volle `ModulePart`-Snapshots sind hier robuster als Feld-fuer-Feld-Aktionen.

2. Einen klaren Commit-Punkt fuer Parameteredits definieren.
   - Undo-Eintraege nur dann anlegen, wenn sich der Wert wirklich geaendert hat.
   - Keine neuen Eintraege pro Render-Frame.
   - Drag-, Slider- und Text-Edits muessen zu sinnvollen Undo-Schritten zusammengefasst werden.

3. Struktur- und Parameter-History zusammenspielen lassen.
   - Bestehende `MovePart`-, Add/Delete- und Connection-Aktionen beibehalten.
   - Parameter-Aktionen muessen mit `Ctrl+Z` / `Ctrl+Y` ueber denselben aktiven Canvas-Pfad funktionieren.

4. Inspector-Pfade systematisch abdecken.
   - `source.rs`, `effect.rs`, `layer.rs`, `output.rs`, `trigger.rs` auf direkte Mutationen pruefen.
   - Fuer jede relevante UI-Kontrolle klar entscheiden, wann der Vorher- und Nachher-Snapshot geschrieben wird.

5. History-Spam vermeiden.
   - Ein einzelner Slider-Drag oder Text-Edit-Flow soll im Regelfall genau einen Undo-Schritt erzeugen.
   - Checkbox-, Combo- und Toggle-Aenderungen sollen jeweils genau einen Undo-Schritt erzeugen.

### Definition of Done

- Parameteraenderungen im Module-Canvas-Inspector sind undo-/redo-faehig.
- Bestehende Struktur-Undo-Funktionen bleiben intakt.
- Ein einzelner Edit-Flow erzeugt keine unkontrollierte History-Flut.
- Es bleibt klar, dass der Scope auf Module-Canvas-Parameter begrenzt ist.

### Validierung

- Mindestens je ein Testfall fuer:
  - Slider oder `DragValue`
  - Checkbox oder Toggle
  - ComboBox-Auswahl
  - Text-/Pfad-Aenderung
- Manuell pruefen:
  - Parameter aendern -> `Ctrl+Z` -> alter Zustand
  - `Ctrl+Y` -> neuer Zustand
  - danach Struktur-Aktion (z. B. Node bewegen) weiterhin funktionsfaehig

## 5. MF-021-NDI-DISCOVERY-UI

### Problem

Die NDI-Discovery ist strategisch beschrieben und in Teilen bereits im Code vorhanden, aber die aktuelle Umsetzung ist nicht sauber abgeschlossen: UI-Logik ist doppelt vorhanden, Discovery-State ist verteilt und die Verknuepfung von Auswahl und tatsaechlicher Runtime-Aktion ist unklar bzw. unvollstaendig.

### Beobachteter Ist-Zustand

- `DOC-C11_VIDEO_IO_STRATEGY.md` beschreibt die Architektur auf Strategie-Ebene.
- `crates/mapflow-io/src/ndi/mod.rs` enthaelt bereits:
  - `discover_sources`
  - `discover_sources_async`
  - `connect`
- Im Module Canvas existiert bereits NDI-bezogener UI-State:
  - `ndi_sources`
  - `ndi_discovery_rx`
  - `pending_ndi_connect`
- `UIAction::ConnectNdiSource` existiert bereits in `crates/mapflow-ui/src/lib.rs`.
- Die eigentliche Runtime-Verarbeitung des Connects existiert bereits in `crates/mapflow/src/app/actions.rs`.
- `pending_ndi_connect` wird aktuell jedoch nicht sichtbar weiterverarbeitet.
- NDI-Inspector-Logik ist mindestens in `inspector/source.rs` und `inspector/mod.rs` doppelt bzw. ueberlappend vorhanden.

### Ziel

- NDI-Quellen sollen in der relevanten UI sichtbar, asynchron discoverbar und verbindbar sein.
- Die Auswahl einer Quelle muss in einen realen Connect-Pfad fuehren.
- Disconnect und Feature-Gating muessen klar und konsistent sein.

### Leitentscheidung

Fuer `MF-021` ist `crates/mapflow-ui/src/editors/module_canvas/inspector/source.rs` die kanonische UI-Stelle fuer `SourceType::NdiInput`.

Das heisst konkret:
- Keine zweite parallele NDI-Inspector-Implementierung pflegen.
- Kein toter Zwischenzustand ueber `pending_ndi_connect`, wenn die Aktion direkt in `UIAction` ausgedrueckt werden kann.

### Nicht-Ziele

- kein NDI-Output-/Sender-Ausbau
- keine automatische Shader-Graph-Node-Erzeugung
- keine allgemeine Netzwerkoptimierung ausserhalb des Discovery-/Connect-Flows
- keine Spout-Arbeit in diesem Task

### Primaere Dateien

- `crates/mapflow-io/src/ndi/mod.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/source.rs`
- `crates/mapflow-ui/src/editors/module_canvas/inspector/mod.rs`
- `crates/mapflow-ui/src/editors/module_canvas/state.rs`
- `crates/mapflow-ui/src/lib.rs`
- `crates/mapflow/src/app/actions.rs`

### Empfohlene Umsetzung

1. NDI-UI an einer Stelle konzentrieren.
   - `source.rs` als kanonischen Renderer fuer `SourceType::NdiInput` verwenden.
   - Doppelte oder veraltete Logik in `inspector/mod.rs` entfernen oder auf `source.rs` delegieren.

2. Discovery asynchron und sichtbar halten.
   - Discovery ueber `discover_sources_async` starten.
   - Spinner, Empty State und Ergebnisliste sauber anzeigen.
   - Keine blockierende Netzwerksuche im UI-Thread.

3. Auswahl in echte Actions ueberfuehren.
   - Bei Quellenauswahl direkt `UIAction::ConnectNdiSource { part_id, source }` ausloesen oder ueber genau einen klaren Dispatcher-Punkt weiterreichen.
   - `pending_ndi_connect` nicht als toten Pufferspeicher stehen lassen.

4. Disconnect explizit modellieren.
   - Wenn `None` gewaehlt wird, muss auch der Runtime-Receiver aufgeraeumt werden.
   - Bevorzugte Richtung: explizite Disconnect-Action statt stiller UI-only-Entkopplung.

5. Build-Feature sauber darstellen.
   - Bei deaktiviertem `ndi`-Feature klare Disabled-Message statt halb funktionaler Controls.

### Definition of Done

- `F-021` aus `DOC-C1_TEST_MATRIX.md` ist technisch geschlossen oder nur mit klar dokumentierter Restkante offen.
- Discovery laeuft asynchron.
- Gefundene Quellen werden angezeigt und koennen ausgewaehlt werden.
- Eine Auswahl fuehrt in einen echten Connect-Pfad.
- Ein Disconnect raeumt auch den Runtime-Zustand konsistent auf.
- Es bleibt keine doppelte NDI-Inspector-Implementierung als aktive Wahrheit zurueck.

### Validierung

- Build mit aktiviertem `ndi`-Feature.
- Discovery starten.
- Ergebnisliste pruefen.
- Quelle auswaehlen.
- Connect-Erfolg oder klaren Fehlerpfad pruefen.
- `None`/Disconnect pruefen.
- Build ohne `ndi`-Feature pruefen: klare Disabled-UI, keine toten Buttons.

## 6. Empfohlene Nutzung im Orchestrator

Wenn diese Tasks delegiert werden, sollte der Orchestrator die jeweilige `MF-ID` weitergeben und dieses Dokument als Detailreferenz nutzen. Fuer `MF-021` bleibt `DOC-C11_VIDEO_IO_STRATEGY.md` die Architekturreferenz, waehrend `DOC-C14` die operative Umsetzungsreferenz ist.
