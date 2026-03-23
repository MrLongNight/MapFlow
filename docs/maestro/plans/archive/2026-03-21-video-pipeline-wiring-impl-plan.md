---
task_complexity: medium
topic: video-pipeline-wiring
date: 2026-03-21
---

# Phasenplan: Video Pipeline Wiring (FramePipeline)

## Plan-Überblick
Dieser Plan integriert die bereits existierende, aber bisher nur in Tests verwendete `FramePipeline` (aus `mapflow-media`) in die Haupt-Rendering-Schleife der Anwendung.

- **Gesamtphasen**: 2
- **Beteiligte Agenten**: `coder`, `code_reviewer`

## Ausführungsstrategie

| Phase | Agent | Typ | Beschreibung |
|-------|-------|-----|--------------|
| 1 | `coder` | Core | Refactoring des `MediaPlayerHandle` zur Nutzung der `FramePipeline`. |
| 2 | `code_reviewer` | QA | Überprüfung der Integration und des Threadings. |

## Phasen-Details

### Phase 1: FramePipeline Integration
- **Ziel**: `create_player_handle` in `media.rs` soll eine `FramePipeline` starten (decode_thread + upload_thread) statt einer einfachen `std::thread::spawn` Schleife.
- **Agent**: `coder`
- **Dateien ändern**:
  - `crates/mapflow/src/orchestration/media.rs`:
    - Ersetze den manuellen `std::thread::spawn` Block durch `pipeline.start_decode_thread(player)` und `pipeline.start_upload_thread(...)`.
    - Das `upload_fn` Closure muss die Textur via `pool.upload_data(...)` in die WGPU Queue schreiben.
    - Speichere die `FramePipeline` (oder einen Wrapper) im `MediaPlayerHandle`, damit sie bei Bedarf gestoppt werden kann.
- **Validierung**: `cargo check -p mapflow`

### Phase 2: Quality & Review
- **Ziel**: Sicherstellen, dass keine Deadlocks oder verwaisten Threads entstehen.
- **Agent**: `code_reviewer`
- **Dateien prüfen**:
  - `crates/mapflow/src/orchestration/media.rs`
- **Validierung**: Lesendes Review, keine Code-Änderungen.
