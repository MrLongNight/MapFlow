# DOC-C11: Audit Open Points on `main` (2026-03-21)

Stand: 2026-03-21

Kontext:
- Geprueft wurden die Audit-/Analyse-Dokumente unter `docs/A3_PROJECT/B2_QUALITY`.
- Der Abgleich erfolgte gegen den aktuellen lokalen Stand von `main`.
- Parallel laufen Merges aus Feature-Branches. Dieses Dokument ist daher ein Snapshot und kein eingefrorener Endstand.

## Kurzfazit

Ein grosser Teil der frueheren Audit-Punkte ist inzwischen erledigt oder zumindest abgesichert:
- CI/CD ist deutlich weiter als im alten Audit.
- die Doku-Migration der "loose files" ist weitgehend abgeschlossen.
- die groessten P0-Basisprobleme aus dem Module-Canvas-Audit wurden bereits adressiert.
- Visual-Automation und Screenshot-Capture existieren inzwischen real im Code.

Offen sind vor allem noch:
- Video-/Interop-Pfade ausserhalb des heute genutzten Standard-Renderwegs
- mehrere end-to-end Luecken im Module-Canvas/Renderpfad
- Doku-/Roadmap-Drift bei NDI, HAP und "Feature fertig" vs. "Feature existiert nur teilweise"
- einige Performance- und QA-Follow-ups

## Bereits erledigt seit den Audits

### Aus DOC-C4_CICD_AUDIT

- Die redundante Release-Workflow-Datei ist weg.
- `.github/dependabot.yml` existiert.
- Release-Automation reagiert auf Tag-Pushes (`.github/workflows/CICD-MainFlow_Job03_Release.yml`).

### Aus DOC-C6_DOCUMENTATION_AUDIT

- Die frueheren Root-"Loose Files" wurden umgezogen:
  - `docs/A3_PROJECT/B1_PLANNING/DOC-C9_HAP_INTEGRATION.md`
  - `docs/A4_USER/B1_MANUAL/DOC-C10_MCP_API_REF.md`
  - `docs/A4_USER/B1_MANUAL/DOC-C7_MIDI_CONTROL.md`
- Die alten Root-Dateien `docs/cleanup-summary.md`, `docs/INDEX.md` und `docs/CODE_ANALYSIS_REPORT.md` liegen nicht mehr dort.
- Ein zentraler Einstieg ist heute ueber `docs/README.md` und `docs/DOC-A0_README.md` vorhanden.

### Aus DOC-C5_CODE_AUDIT_REPORT

- `paint_texture_cache.rs` laedt Bilder inzwischen aus `source_path`; das alte `TODO` fuer Image-Loading ist nicht mehr der Stand von `main`.
- Das alte UX-Problem "Detect socket under pointer" ist im aktuellen Module-Canvas umgesetzt.
- Die frueher genannten Dead-Code-Hinweise sind in den konkret genannten Dateien nicht mehr in derselben Form sichtbar:
  - `crates/mapflow-render/src/mesh_buffer_cache.rs` ist heute aktiv im Renderpfad benutzt.
  - in `crates/mapflow-ui/src/editors/mesh_editor/*` ist die fruehere pauschale `dead_code`-Lage in dieser Form nicht mehr offensichtlich.

### Aus DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18

- Die Foundation-Phase (Tracker #1243) ist abgeschlossen. Die Basis-Architektur (`repair_graph`, `RuntimeRenderQueue`, Socket-Schema-Metadaten) ist implementiert und stabil. Sub-Issues haben aktuellen Scope und Basisannahmen stimmen ueberein.

- `ModuleManager::get_module_mut()` dirty-markiert nicht mehr implizit.
- Core-Validierung fuer Verbindungen ist aktiv.
- `repair_graph()` ist eingebaut.
- Presets nutzen keine driftende UI-Socket-Kopie mehr als primaere Wahrheit.
- Output-Fenster-Cleanup ist inzwischen implementiert (`crates/mapflow/src/orchestration/outputs.rs` entfernt stale windows).
- Projector-Advanced-Settings (`output_width`, `output_height`, `output_fps`) sind inzwischen im Inspector vorhanden.
- Automation-Capture / Screenshot-Pfad ist real vorhanden, inklusive Tests und self-hosted Artifact-Upload.
- Source-Transforms sind im aktuellen Renderpfad nicht mehr nur UI-Attrappe; der aktive Renderpfad wendet sie an.
- Module-Canvas-Katalog und Add-Node-Menue blenden aktuell nicht unterstuetzte Nodes inzwischen capabilities-basiert aus, statt sie pauschal sichtbar zu lassen.

## Noch offene Punkte

### 1. ERLEDIGT: Legacy- und Parallelpfade fuer Video sind weiter unvollstaendig

Quelle:
- `DOC-C5_CODE_AUDIT_REPORT`
- `DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18`

Status:
- Der heute aktive App-Pfad nutzt fuer Medien `crates/mapflow/src/orchestration/media.rs` und laedt Frames direkt in den `TexturePool`.
- Damit ist der alte Audit-Hinweis zum Blackscreen nicht mehr 1:1 der produktive Hauptpfad.
- Trotzdem bleiben parallele oder alte Video-Pfade offen:
  - `crates/mapflow-render/src/paint_texture_cache.rs`: `PaintType::Video` und `PaintType::Camera` TODOs wurden entfernt und als Legacy-Pfade dokumentiert.
  - `crates/mapflow-media/src/pipeline.rs`: `FramePipeline` wurde explizit als obsolete (`#[deprecated]`) markiert.

Bewertung:
- Kein akuter Beleg dafuer, dass `main` deswegen im Standardpfad schwarz bleibt.
- Aber die alte Architekturgeschichte ist nicht sauber abgeschlossen, sondern teils umgangen.

### 2. NDI, SRT und HAP sind nicht end-to-end produktionsreif

Quelle:
- `DOC-C5_CODE_AUDIT_REPORT`
- `DOC-C7_PROJECT_PLANNING_AUDIT`
- `DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18`

Status:
- `crates/mapflow-io/src/stream/srt.rs` ist weiterhin explizit als Stub markiert.
- `crates/mapflow-io/src/ndi/mod.rs` hat kein altes `TODO: Implement actual frame sending` mehr, enthaelt aber weiterhin den Hinweis, dass die Sender-Implementierung ein Placeholder/zu verifizierender Pfad ist.
- `crates/mapflow-ui/src/editors/module_canvas/inspector/source.rs` zeigt weiterhin explizite Hinweise, dass bestimmte Source-Typen nicht voll an die Runtime angeschlossen sind:
  - Shader
  - LiveInput
  - SpoutInput
- `crates/mapflow-media/src/hap_player.rs` sagt weiterhin selbst, dass der Pfad noch eine FFmpeg-Integration als Platzhalter-Luecke hat.
- Default-Feature-Gating ist verbessert:
  - `mapflow-io` hat `default = []`
  - `mapflow` aktiviert `ndi` nicht standardmaessig

Bewertung:
- "Standardmaessig deaktivieren, wenn nicht fertig" ist fuer NDI/SRT weitgehend eingehalten.
- "Feature existiert im Code" ist aber weiterhin nicht dasselbe wie "produktive end-to-end Runtime vorhanden".

### 3. Oeffentliche Doku und Planungsstand ueberzeichnen Feature-Reife

Quelle:
- `DOC-C6_DOCUMENTATION_AUDIT`
- `DOC-C7_PROJECT_PLANNING_AUDIT`

Status:
- Die prominentesten User-/Tech-Dokumente wurden inzwischen auf den tatsaechlichen Reifegrad heruntergestuft, insbesondere:
  - `docs/A4_USER/B1_MANUAL/DOC-C5_FEATURES.md`
  - `docs/A4_USER/B1_MANUAL/DOC-C4_UI_OVERVIEW.md`
  - `docs/A4_USER/B1_MANUAL/DOC-C2_QUICKSTART.md`
  - `docs/A2_DEVELOPMENT/B4_TECHNICAL/DOC-C1_RENDERING.md`
- Planungsdokumente verweisen auf GitHub Project Issues als "source of truth". Die alte `ROADMAP.md` wurde entfernt.

Bewertung:
- Das eigentliche Doku-Umraeumen ist weitgehend geschafft.
- Der sichtbarste User-/Tech-Doku-Drift ist reduziert, aber die konsistente Bereinigung ueber Planung/Roadmap ist noch offen.

### 4. Module-Canvas und Renderpfad sind weiter nicht fuer alle sichtbaren Features end-to-end geschlossen

Quelle:
- `DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18`

Status:
- Masken:
  - `crates/mapflow/src/orchestration/evaluation.rs` erzeugt weiter `masks_unsupported`.
  - `crates/mapflow-ui/src/editors/module_canvas/inspector/capabilities.rs` markiert Masken explizit als nicht voll unterstuetzt.
- Blend Modes:
  - `crates/mapflow/src/orchestration/evaluation.rs` erzeugt weiter `blend_mode_unsupported`.
  - `crates/mapflow-ui/src/editors/module_canvas/inspector/capabilities.rs` behandelt praktisch nur `Normal` als wirklich unterstuetzt.
- Sichtbarkeit nicht fertiger Nodes:
  - Der Node-Katalog und das Add-Node-Menue filtern nicht unterstuetzte Source-/Mask-/Blend-/Effect-Nodes inzwischen capabilities-basiert heraus.
  - Offene Runtime-Luecken bleiben davon getrennt weiter bestehen, insbesondere fuer Shader, LiveInput und Spout im Inspector-/Runtime-Pfad.
- Hue:
  - `crates/mapflow-ui/src/editors/module_canvas/inspector/output.rs` enthaelt fuer Hue-Pairing/Areas weiterhin TODOs.

Bewertung:
- Die groben P0-Architekturprobleme wurden reduziert.
- Die noch sichtbaren Funktionsflaechen sind aber nicht alle ehrlich durchgeroutet.

### 5. QA-/Visual-Capture-Faehigkeit ist vorhanden, aber nicht voll "always-on"

Quelle:
- `DOC-C9_VISUAL_CAPTURE_READINESS`
- `DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18`

Status:
- Positiv:
  - `mapflow_visual_harness` existiert.
  - Automation-Mode mit `--exit-after-frames` und `--screenshot-dir` existiert.
  - self-hosted Post-Merge Workflow kann Visual-Capture-Artefakte hochladen.
- Offen:
  - relevante GUI-/GPU-Tests sind weiter `#[ignore]` und brauchen eine interaktive Windows-GPU-Session.
  - die self-hosted Visual-Automation ist variablen-/runner-abhaengig und nicht als immer aktive Standard-Absicherung erkennbar.
  - im Module-Canvas-Audit ist der finale Release-Smoke-Test weiterhin explizit offen genannt.

Bewertung:
- Der Unterbau ist jetzt real.
- Die harte, dauerhafte Release-Absicherung ueber diesen Pfad ist noch nicht voll geschlossen.

### 6. Performance-Verbesserungen aus dem alten Analysebericht sind nur teilweise eingelost

Quelle:
- `DOC-C8_PERFORMANCE_ANALYSIS_BOLT`
- `DOC-C3_CODE_ANALYSIS_REPORT_2025-12-29`

Status:
- Erledigt:
  - `visible_layers()` in `crates/mapflow-core/src/layer/manager.rs` liefert heute einen Iterator statt eines neu allokierten `Vec`.
- Weiter offen:
  - `AppState` ist weiterhin `Clone`-lastig.
  - im Rendercode werden weiter BindGroups/Uniform-Buffer in Hot-Paths erzeugt, z. B. in:
    - `crates/mapflow-render/src/effect_chain_renderer/apply.rs`
    - `crates/mapflow-render/src/compositor.rs`
    - `crates/mapflow-render/src/mesh_renderer.rs`

Bewertung:
- Es gab Verbesserungen.
- Die eigentliche Render-Hot-Path-Optimierung aus dem Report ist aber nicht als abgeschlossen erkennbar.

## Berichte ohne aktuell uebernommene Restpunkte

### DOC-C2_AUDITS

Dieses Dokument ist ein Index. Es enthaelt selbst keine neuen offenen Punkte.

### DOC-C3_CODE_ANALYSIS_REPORT_2025-12-29

Der Report ist stark branch-/PR-bezogen (`feature/node-menu-overhaul`, PR #131/#133/#129).
Die dortigen konkreten PR-Hinweise werden nicht als aktuelle `main`-Restpunkte uebernommen.
Uebernommen wurden nur die allgemeinen, weiterhin sichtbaren Technikthemen:
- Refactoring-/Komplexitaetsdruck
- Performance-Profiling / Render-Hot-Path

### DOC-C4_CICD_AUDIT

Die konkret dokumentierten harten Punkte aus dem Audit sind auf dem heutigen Stand weitgehend erledigt.
Eventuelle Restthemen dort sind eher Maintainability-Verbesserungen als offene Qualitaetsblocker.

### DOC-C6_DOCUMENTATION_AUDIT

Das eigentliche Umraeumen der Dateien ist weitgehend erledigt.
Offen bleibt nicht mehr der Dateiumzug, sondern die inhaltliche Drift zwischen Doku, Planung und Runtime.

## Priorisierte Restliste

### P0

- Produkt-/Doku-Drift fuer NDI, HAP, Virtual Outputs und "Feature fertig" aufraeumen.
- Module-Canvas nur noch Features sichtbar machen lassen, die end-to-end funktionieren, oder die Runtime schliessen.
- [x] Legacy-/Parallelpfade fuer Video (`paint_texture_cache`, `FramePipeline`) wurden klar deklassiert und obsolet markiert.

### P1

- Masken und Blend Modes end-to-end schliessen oder aus UI/Inspector noch klarer deklassieren.
- NDI/SRT/HAP-Reifegrad pro Feature sauber dokumentieren.
- finalen Release-Smoke-Test fuer den jetzt vorhandenen Automation-/Capture-Pfad nachziehen.

### P2

- Render-Hot-Path-Optimierungen aus dem Performance-Report systematisch abarbeiten.
- verbleibende Architektur-/Refactoring-Themen aus den alten Analyseberichten in separate Tech-Debt-Tasks ueberfuehren.
