# Code Audit Report

**Datum:** 2026-01-04
**Verantwortlich:** Agent (Antigravity)

## Zusammenfassung
Die statische Analyse des Codebestands hat eine signifikante Anzahl an `TODO`-Kommentaren und `dead_code`-Annotationen aufgezeigt. Es gibt Bereiche mit unvollständiger Implementierung, insbesondere in `mapflow-ui` und `mapflow-io`.

## Kritische Funde (TODOs)
 Diese Punkte erfordern sofortige Aufmerksamkeit, da sie Kernfunktionalitäten betreffen könnten.

| Datei | Zeile | Inhalt | Priorität | Kontext |
|-------|-------|--------|-----------|---------|
| `mapflow-render/src/paint_texture_cache.rs` | 109 | `TODO: Load from video decoder` (Resolved: Legacy path) | **ERLEDIGT** | Geklärt: Standardpfad nutzt mapflow::orchestration::media |
| `mapflow-render/src/paint_texture_cache.rs` | 105 | `TODO: Load from source_path` (Resolved) | **ERLEDIGT** | |
| `mapflow-media/src/pipeline.rs` | 218 | `TODO: Upload to GPU here` (Resolved: Obsolete path) | **ERLEDIGT** | Geklärt: Obsoleter Legacy-Pfad |
| `mapflow-io/src/ndi/mod.rs` | 338 | `TODO: Implement actual frame sending` | Mittel | NDI Feature unvollständig |
| `mapflow-io/src/stream/srt.rs` | 137 | `TODO: Implement frame sending` | Mittel | SRT Feature unvollständig |
| `mapflow-ui/src/node_editor.rs` | 560 | `TODO: Detect socket under pointer` | Mittel | UX Problem |

## Dead Code Analyse
Viele Dateien nutzen `#[allow(dead_code)]`. Während dies während der Entwicklung normal ist, deutet die Menge auf Aufräumbedarf hin.

- **mapflow-ui/src/mesh_editor.rs**: Enthält viele ungenutzte Felder in Structs. Vermutlich Relikte aus einer früheren Implementierung.
- **mapflow-render/src/mesh_buffer_cache.rs**: Mehrere ungenutzte Funktionen.

## Empfohlene Maßnahmen
1.  **Video Pipeline Fix:** Die TODOs in `paint_texture_cache.rs` und `pipeline.rs` müssen implementiert werden. Dies ist wahrscheinlich der Fix für das Blackscreen-Problem.
2.  **NDI/SRT Fertigstellung:** Wenn diese Features beworben werden, müssen sie implementiert werden. Andernfalls Feature-Flag standardmäßig deaktivieren.
3.  **Cleanup:** Ein dedizierter "Jules Clean" Task sollte alle `dead_code` Warnungen prüfen.

## Jules Tasks
Ich schlage vor, Jules folgende Tasks zuzuweisen:
- [x] Implement Video Decoder Loading (`paint_texture_cache.rs`) - Resolved as Legacy Path
- [x] Implement GPU Upload Placeholder in (`pipeline.rs`) - Resolved as Obsolete Path
