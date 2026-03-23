# MapFlow Render Queue & Runtime Plan

Stand: 2026-03-19

## 1. Zweck

Diese Datei beschreibt den aktuell implementierten Runtime-Pfad fuer das Node-System.

Wichtig:

- Die visuelle Runtime-Queue ist **nicht** `FramePipeline`.
- `FramePipeline` bleibt eine interne Media-Decode/Upload-Pipeline.
- Die eigentliche App-Runtime arbeitet jetzt mit einer expliziten **Runtime Render Queue**.

## 2. Single Source of Truth

Primaere Implementierungsstellen:

- `crates/mapflow-core/src/module/types/socket.rs`
- `crates/mapflow-core/src/module/types/schema.rs`
- `crates/mapflow-core/src/module/types/module.rs`
- `crates/mapflow-core/src/module/manager.rs`
- `crates/mapflow-core/src/module_eval/evaluator/mod.rs`
- `crates/mapflow/src/app/core/app_struct.rs`
- `crates/mapflow/src/orchestration/evaluation.rs`
- `crates/mapflow/src/app/loops/logic.rs`
- `crates/mapflow/src/app/loops/render/content.rs`
- `crates/mapflow/src/app/loops/render/mod.rs`
- `crates/mapflow/src/app/loops/render/previews.rs`
- `crates/mapflow/src/orchestration/media.rs`

Ergaenzende Architekturreferenz:

- `docs/A1_SYSTEM/B1_ARCHITECTURE/DOC-C5_OUTPUT_WINDOW_LIFECYCLE.md`

## 3. Aktuelle Runtime-Architektur

Es gibt jetzt zwei klar getrennte Ebenen:

1. `mapflow-media::FramePipeline`
   - dekodiert und uploadet Media-Frames
   - ist ein interner Source-Transport
   - ist **nicht** die globale Render Queue

2. `mapflow::RuntimeRenderQueue`
   - ist die visuelle Queue fuer den aktuellen Frame
   - enthaelt `RuntimeRenderQueueItem { module_id, render_op }`
   - wird pro Frame aus den Evaluator-Ergebnissen neu aufgebaut

## 4. Ablauf pro Frame

### 4.1 Graph Repair

In `crates/mapflow/src/app/loops/logic.rs` gilt:

- Wenn `graph_revision` geaendert wurde, ruft die App vor der Evaluation `ModuleManager::repair_modules(...)` auf.
- Dadurch werden inkonsistente Socket-Schemata, ungultige Trigger-Mappings und kaputte oder doppelte Connections bereinigt.
- Jede automatische Reparatur wird geloggt.

### 4.2 Evaluation

In `crates/mapflow/src/orchestration/evaluation.rs`:

- `ModuleEvaluator::evaluate(...)` liefert `render_ops` pro Modul.
- Daraus baut die App die konsolidierte `RuntimeRenderQueue`.
- Die Queue speichert die erzeugende `graph_revision`.

### 4.3 Rendering

In `crates/mapflow/src/app/loops/render/content.rs`:

- Der Render-Loop filtert `RuntimeRenderQueueItem`s pro Output.
- Danach werden Effekte, Mesh-Rendering und Output-spezifische Post-Schritte angewandt.

### 4.4 Media

In `crates/mapflow/src/orchestration/media.rs`:

- Media-Player und GPU-Uploads laufen weiterhin getrennt.
- Die Render Queue referenziert nur die bereits verfuegbaren Texturen.

## 5. Node-/Socket-Basis

Die Basis fuer Nodes, Verbinder und Inspector ist jetzt zentralisiert:

- `ModuleSocket` enthaelt jetzt:
  - stabile `id`
  - `direction`
  - `supports_trigger_mapping`
  - `is_primary`
  - `accepts_multiple_connections`
- `ModulePart::schema()` liefert ein konsolidiertes Runtime-Schema fuer:
  - Node-Kategorie
  - Inputs
  - Outputs
  - Inspector-relevante Socket-Indizes

Das Ziel ist, dass Canvas, Inspector und Validierung dieselbe Schemaquelle verwenden.

## 6. Connection-Regeln

`MapFlowModule` validiert Verbindungen jetzt ueber:

- Part-Existenz
- gueltige Socket-Indizes
- Richtung Output -> Input
- Socket-Typ-Kompatibilitaet
- Verbot von Self-Connections

Die Hauptmethoden dafuer:

- `validate_connection(...)`
- `connect_parts(...)`
- `repair_graph()`

## 7. Selbstheilung

`repair_graph()` fuehrt aktuell diese Schritte aus:

- Socket-Sets aller Parts aus `compute_sockets()` neu ableiten
- invalide `trigger_targets` entfernen
- doppelte oder ungueltige Connections entfernen
- doppelte/ungueltige Projector-IDs normalisieren
- doppelte/ungueltige Layer-IDs normalisieren

Das ist bewusst defensiv gebaut:

- ein defektes Modul soll moeglichst degradiert weiterlaufen
- kaputte Kanten sollen entfernt werden statt spaeter zu crashen
- jede Reparatur soll sichtbar loggen

## 8. Dirty-Semantik

`ModuleManager::get_module_mut()` markiert den Graphen nicht mehr automatisch dirty.

Grund:

- Canvas und Inspector greifen sehr haeufig lesend oder opportunistisch mutierend auf das Modul zu
- die alte Semantik hat `graph_revision` praktisch in jedem Frame veraendert
- dadurch liefen unnötige Re-Evaluierungen und Syncs

Dirty wird jetzt explizit gesetzt, wenn:

- der Canvas wirklich editiert wurde
- der Inspector eine Part-Konfiguration veraendert hat
- `repair_graph()` bzw. `repair_modules()` echte Aenderungen vorgenommen hat

## 9. Abgrenzung zu `FramePipeline`

`FramePipeline` bleibt weiterhin relevant, aber nur fuer Source-Frames:

- Decode-Thread
- Upload-Thread
- Status-/Backpressure-Logik

Sie ist kein Ersatz fuer die Render Queue des Node-Systems.

Kurz:

- `FramePipeline` = Medien-Transport
- `RuntimeRenderQueue` = visueller Frame-Plan

## 10. Bekannte Restluecken

Die neue Basis ist implementiert, aber der Umbau ist noch nicht vollstaendig:

- nicht alle Inspector-Panels nutzen schon das neue Schema vollstaendig
- `RenderOp.masks` und `blend_mode` sind im Renderpfad weiterhin nur teilweise umgesetzt
- mehrere Node-Typen bleiben funktional unvollstaendig
- der native Windows-Start inklusive Automation-Capture ist im Debug-Build verifiziert, sollte vor einem Release-Artefakt aber noch einmal separat als Release-Smoke-Test geprueft werden

### 10.1 Start- und Stabilitaetsfixes 2026-03-19

- `crates/mapflow/build.rs` kopiert FFmpeg-Runtime-DLLs jetzt bevorzugt aus `vcpkg_installed/x64-windows/bin` und validiert PE-Header, bevor Dateien in `target/<profile>` uebernommen werden.
- `crates/mapflow-bevy/src/lib.rs` deaktiviert im eingebetteten Runner explizit `bevy::winit::WinitPlugin`, damit MapFlow und Bevy nicht konkurrierende Event-Loops erzeugen.
- `crates/mapflow/src/app/loops/render/mod.rs` erhoeht `frame_counter` wieder im primaeren Renderpfad, sodass `--exit-after-frames` im Automation-Modus tatsaechlich greift.
- `crates/mapflow-bevy/src/systems.rs` schliesst GPU-Readback-Mappings jetzt innerhalb desselben Frames ab und entmappt den Buffer deterministisch, statt gemappte Buffer in spaeteren Frames wiederzuverwenden.
- `crates/mapflow/src/app/core/init.rs` erzeugt die `composite`-Textur jetzt mit `wgpu::TextureUsages::COPY_SRC`, sodass der Automation-Screenshot-Pfad keine WGPU-Validierungspanik mehr ausloest.
- Verifiziert am 2026-03-19:
  - `target/debug/MapFlow.exe --help` -> `EXIT=0`
  - `target/debug/MapFlow.exe --mode automation --exit-after-frames 1` -> `EXIT=0`
  - `target/debug/MapFlow.exe --mode automation --exit-after-frames 1 --screenshot-dir <dir>` -> `EXIT=0`

### 10.2 Output-/Window-Lifecycle

- Der Runtime-Pfad fuer Output-Fenster ist noch nicht vollstaendig vereinheitlicht.
- `sync_output_windows(...)` ist aktuell der aktive Projector-Window-Pfad.
- `crates/mapflow/src/window_manager.rs` enthaelt dabei nicht nur tote Altlasten, sondern auch vorbereitete Lifecycle-Infrastruktur.
- Der geplante Soll-Zustand fuer `OutputManager`, `Projector`-Nodes und `WindowManager` ist in `DOC-C5_OUTPUT_WINDOW_LIFECYCLE.md` beschrieben.

## 11. Naechste Ausbaupunkte

P0:

- Renderpfad fuer `blend_mode`, Masken und Source-Transform wirklich end-to-end schliessen
- Inspector fuer Output- und Spezial-Source-Typen weiter auf Core-Schema umstellen
- Trigger-/Connector-Konzept weiter in Richtung `Media` vs `Control` vs `Event` schaerfen

P1:

- Socket-Verbindungen langfristig von Indexen auf stabile Socket-IDs migrieren
- Render Queue um strukturierte Diagnostics pro Item erweitern
- Fault-Isolation pro Node/Modul weiter ausbauen
- Output-/Window-Lifecycle zwischen `OutputManager`, Graph und `WindowManager` vollstaendig konsolidieren
