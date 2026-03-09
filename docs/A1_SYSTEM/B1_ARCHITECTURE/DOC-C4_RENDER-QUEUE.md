# MapFlow Render Queue & Pipeline Architektur (Single Source of Truth)

> Diese Datei ist die **konsolidierte Gesamtdokumentation** für:
> - System-/Thread-Architektur rund um Rendering und Media
> - den konkreten Decode → Upload → Render Queue-Laufzeitpfad
> - den logischen PAP-Datenfluss (Trigger/Source/Modulizer/Layer/Output)
>
> Ziel: Alle relevanten Informationen strukturiert an einem Ort zusammenführen.

---

## 1. Ziel und Scope

Diese Dokumentation beschreibt den tatsächlichen und geplanten Render-Pfad in MapFlow mit Fokus auf Video-Frames:

1. **Decode-Thread** erzeugt `PipelineFrame`s aus `VideoPlayer`.
2. **Upload-Thread** lädt Frame-Daten in GPU-Texturen.
3. **Render/Main-Thread** nutzt die aktualisierten Texturen für Komposition und Ausgabe.

Sie kombiniert:

- **Ist-Implementierung** im Code (threading, queues, upload-path)
- **Systemkontext** (Crates, Rollen, Verantwortlichkeiten)
- **Fachlichen Fluss** (PAP: Trigger → Source → Modulizer → Layer → Output)

Primäre Code-Referenzen:

- `crates/mapmap-media/src/pipeline.rs` (`FramePipeline`, `FrameScheduler`)
- `crates/mapmap/src/orchestration/media.rs` (Player-Orchestrierung, Queue-Drain)
- `crates/mapmap-render/src/uploader.rs` (`WgpuFrameUploader`)

---

## 2. Systemarchitektur (Render/Media-relevant)

### 2.1 High-Level Komponenten

- **mapmap-core**: Domänenmodell (Paint/Mapping/Shape), Geometrie, Projektzustand
- **mapmap-render**: wgpu-Backend, Texturverwaltung, Shader/Pipelines, Renderer
- **mapmap-media**: Decoder/Player, Frame-Pipeline, Playback-Steuerung
- **mapmap-ui**: UI-Zustand, Panels/Controls, Darstellung von Playback-Infos
- **mapmap (binary/app)**: Orchestrierung, Event-Loop, Main-Renderloop

### 2.2 Threading-Modell (IST)

Aktuell ist für Media bereits eine asynchrone Pipeline aktiv:

- Decode in eigenem Thread (`start_decode_thread`)
- Upload in eigenem Thread (`start_upload_thread`)
- Render + Present im Main-Thread
- Synchronisation via bounded `crossbeam_channel`

Damit liegt faktisch bereits ein „Phase 2-artiger“ Pipeline-Teil vor (zumindest für Decode/Upload), auch wenn andere Bereiche weiterhin im Main-Thread orchestriert werden.

### 2.3 Threading-Modell (Roadmap / Zielbild)

Langfristig: stärkere Entkopplung und Parallelisierung weiterer Render- und Processing-Schritte, inkl. robuster Backpressure-Strategien, deterministischer Shutdown-Semantik und erweiterter Telemetrie.

---

## 3. Logischer PAP-Datenfluss (fachlich)

### 3.1 End-to-End Kette

```text
TRIGGER → SOURCE → MODULIZER → LAYER → OUTPUT
```

- **Trigger**: Audio FFT, Beat, MIDI, OSC, Keyboard, Timer
- **Source**: Media File, Live Input/Camera, NDI Input, Shader Generator, Image Sequence
- **Modulizer**: Effekte, Blend Modes, Audio Reactive, Masken
- **Layer**: Layering, Gruppen, Compositing
- **Output**: Projector Window, Preview, NDI, Spout, Hue

### 3.2 Wo die Render Queue liegt

Die hier beschriebene Queue-Logik ist der **operative Transportpfad für Media-Frames** innerhalb dieser PAP-Kette:

- Source/Media erzeugt Frames (Decode)
- Render-nahe Vorbereitung lädt nach GPU (Upload)
- Render/Main konsumiert den aktuellen Zustand (Draw/Present)

---

## 4. Konkrete Render-Queue-Architektur (Decode → Upload → Render)

## 4.1 `FramePipeline`

`FramePipeline` enthält:

- `decode_tx/decode_rx`: Decode → Upload
- `upload_tx/upload_rx`: Upload → Render/Main
- `running: Arc<AtomicBool>`: gemeinsamer Lifecycle
- `stats: Arc<RwLock<PipelineStats>>`: Laufzeitmetriken
- `decode_thread: Option<JoinHandle<()>>`: Decode-Thread-Handle

**Wichtig (IST):** Upload-Thread-Handle wird derzeit nicht als Feld gehalten/gejoint.

## 4.2 Queue-Strategie

Beide Queues werden via `bounded(config.queue_depth)` erstellt.

Default:

- `queue_depth = 3` (Triple Buffering)
- `enable_frame_drop = true`

Hinweis: `queue_depth` wird aktuell nicht auf `>= 1` geklemmt. `0` hätte Rendezvous-Verhalten.

## 4.3 Decode-Thread Ablauf

`start_decode_thread(player)`:

1. `player.update(1/fps)`
2. Frame → `PipelineFrame { frame, sequence, priority }`
3. Senden an Decode-Queue:
   - Drop-Modus: `try_send`
   - ohne Drop-Modus: `send` (blockierend)
4. Stats-Update (`decoded_frames`, `decode_time_ms`)
5. FPS-angepasstes Throttling via `sleep`

## 4.4 Upload-Thread Ablauf

`start_upload_thread(upload_fn)`:

1. `recv_timeout(100ms)` aus Decode-Queue
2. `upload_fn(&PipelineFrame)` ausführen
3. Bei Fehler: loggen, **nicht** in Upload-Queue weiterleiten
4. Bei Erfolg: `upload_tx.send(pipeline_frame)`
5. Stats-Update (`uploaded_frames`, `upload_time_ms`)

Semantik: Render-Seite bekommt nur Metadaten für erfolgreich verarbeitete Uploads.

## 4.5 Render/Main-Thread Ablauf

In `update_media_players`:

- `while let Ok(frame) = upload_rx.try_recv()`
- `current_time` wird auf neuesten Timestamp gesetzt

Die Textur ist bereits hochgeladen; die Queue dient primär als Synchronisations-/Statuskanal (UI/Playback).

---

## 5. Upload-Pfad auf GPU (WGPU)

`WgpuFrameUploader::upload(texture, data, width, height)`:

1. `unpadded_bytes_per_row = width * 4`
2. 256-Byte Alignment berechnen
3. Staging Buffer erzeugen (`create_buffer_init`)
   - direkt bei bereits passender Alignment
   - sonst gepaddete Repack-Daten
4. `copy_buffer_to_texture` encoden
5. via `queue.submit(...)` einreichen

Eigenschaft: expliziter staging-buffer-basierter Copy-Pfad statt direktem `queue.write_texture`.

---

## 6. Scheduler-Verhalten (`FrameScheduler`)

Aktuell:

- interne Struktur: `Vec<PipelineFrame>`
- `push`:
  - bei voller Queue: niedrigste Priorität via linearem Scan entfernen
  - danach komplette Sortierung nach Priorität
- `pop`: entfernt Element 0 (höchste Priorität)

Komplexität:

- `push`: O(n) + O(n log n)
- `pop`: O(n)

Für kleine Queue-Tiefen praktikabel; bei höherer Last optimierbar.

---

## 7. Lifecycle, Shutdown, Fehlersemantik

## 7.1 Stärken

- Gemeinsames `running`-Flag für Decode/Upload-Loops
- Decode-Thread wird in `stop()` gejoint
- Upload-Schleife blockiert nicht dauerhaft (`recv_timeout`)
- Upload-Fehler werden isoliert (kein Forwarding fehlerhafter Frames)

## 7.2 Aktuelle Risiken/Lücken

1. Kein `upload_thread`-Handle im `FramePipeline`
2. Potenziell blockierendes `send(...)` ohne Timeout im Non-Drop-Modus
3. `queue_depth == 0` nicht abgefangen

Diese Punkte beeinflussen vor allem deterministisches Shutdown-Verhalten und Backpressure unter Last.

---

## 8. Metriken, Monitoring, Betrieb

`PipelineStats` aktuell:

- `decoded_frames`
- `uploaded_frames`
- `rendered_frames` (derzeit nicht vollständig als End-to-End-Zähler genutzt)
- `dropped_frames`
- `decode_time_ms` / `upload_time_ms` (letzte Messung)

Empfohlen für nächste Ausbaustufe:

- getrennte Drop-Metriken pro Queue-Segment
- Queue-Occupancy-Sampling
- rollierende Latenzstatistiken (avg, p95, p99)

### Betriebsprofile

- **Niedrige Latenz (Live):** `enable_frame_drop=true`, kleine Queue-Tiefe (2–3)
- **Integrität (offline-näher):** `enable_frame_drop=false`, größere Queue-Tiefe

---

## 9. Konkreter Orchestrierungsfluss im App-Code

Beim Anlegen eines Players (`create_player_handle`):

1. Media öffnen (`open_path`)
2. Play starten
3. Pipeline erzeugen
4. Decode-Thread starten
5. Zieltextur im Pool absichern (`ensure_texture`)
6. Upload-Thread mit Closure starten:
   - CPU-Frame extrahieren (`FrameData::Cpu`)
   - Texturgröße absichern (`ensure_texture`)
   - Upload (`WgpuFrameUploader::upload`)

Im Tick (`update_media_players`):

- Status-Events drainen (`PlaybackStatus`)
- Upload-Queue drainen (`try_recv`) und Zeitstempel aktualisieren
- UI-Playerinfos synchronisieren

---

## 10. Offene Verbesserungen (technisch)

1. `upload_thread: Option<JoinHandle<()>>` speichern + joinen
2. `queue_depth.max(1)` erzwingen
3. `send_timeout` statt blockierendem `send` in kritischen Pfaden
4. effizienterer Scheduler (z. B. binäre Einfügung / Heap mit stabiler Prioritätsstrategie)
5. observability-Ausbau (occupancy, latency distribution, segmentierte Drops)

---

## 11. Warum eine konsolidierte Doku sinnvoll ist

Für diesen Bereich ist eine **einzelne Source of Truth** sinnvoll, weil Architektur, PAP-Flow und Queue-Laufzeitsemantik in der Praxis eng verzahnt sind. Die frühere Trennung führte zu Redundanz und Inkonsistenzrisiko.

Darum gilt ab jetzt:

- Diese Datei enthält die vollständige Referenz.
- Andere Dateien verweisen nur noch auf diese zentrale Doku.
