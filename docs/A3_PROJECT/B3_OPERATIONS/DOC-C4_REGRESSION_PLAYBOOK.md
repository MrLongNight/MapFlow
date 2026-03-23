# DOC-C4: Regression Playbook

Stand: 2026-03-13

## Ziel

Diese Datei sammelt groessere Fixes und schwer auffindbare Ursachen, die mit hoher
Wahrscheinlichkeit spaeter erneut auftreten koennen. Fokus:

- stille Fehler ohne brauchbare Logs
- Ursachen, die man beim ersten Blick an der falschen Stelle sucht
- Bugs, die bereits einmal "weg" waren und spaeter wieder auftauchen

## Empfohlener Eintrag

Jeder Eintrag sollte mindestens enthalten:

1. Symptom
2. Echte Ursache
3. Warum die Suche lange gedauert hat
4. Fix-Ort im Code
5. Regressionstest oder fehlende Testluecke

---

## 2026-03-13 - Output/Preview blackscreen trotz sichtbarer Media-Node-Vorschau

### Symptom

- Media-Node-Vorschau zeigt Bild
- Output-Fenster und Preview bleiben schwarz
- kein klarer Fehler im Log

### Echte Ursache

`ModuleEvaluator::trace_chain_into()` und die RenderOp-Erzeugung haben bisher
einfach die erste eingehende Verbindung eines Nodes verfolgt. Bei Layern mit
zusaetzlicher Trigger-Verkabelung konnte dadurch statt der visuellen Kette
(Socket 0) die Trigger-Leitung (Socket 1) genommen werden.

Effekt:

- Source-Texture existiert
- Node-Vorschau funktioniert
- RenderOp verliert aber den eigentlichen visuellen Upstream
- Output rendert schwarz

### Warum die Suche lange gedauert hat

- Das Verhalten sah zunaechst wie ein Problem in der Media-/Render-Queue aus.
- Die Quelle war bereits sichtbar, dadurch wirkte Decode/Upload unauffaellig.
- Im Log gab es keinen harten Fehler.

### Fix-Ort

- `crates/mapflow-core/src/module_eval.rs`
- Helfer: `primary_render_connection_idx(...)`
- Visual chain folgt jetzt explizit `to_socket == 0`

### Regressionstest

- `test_render_trace_prefers_layer_visual_input_over_trigger_input`

---

## 2026-03-13 - Projektor-Output stale/black ausserhalb von Timeline-Playback

### Symptom

- Main-UI bzw. Node-Vorschau aktualisiert sich
- Projektorfenster bleiben schwarz oder stale
- Preview-Panel zeigt kein vernuenftiges Output-Bild

### Echte Ursache

Es lagen zwei getrennte Fehler vor:

1. Output-Fenster wurden im Idle/Edit-Modus nicht neu gezeichnet, sondern nur
   bei Timeline-Playback.
2. Das Preview-Panel hing an `output_assignments`, obwohl diese Map im App-Pfad
   nicht sinnvoll befuellt wurde.

### Warum die Suche lange gedauert hat

- Das Verhalten wirkte wie ein Render-Queue- oder Upload-Problem.
- Die Node-Vorschau war bereits da, dadurch war die Quelle nicht offensichtlich
  verdaechtig.
- Preview und physischer Output hatten unterschiedliche, still kaputte Pfade.

### Fix-Ort

- `crates/mapflow/src/main.rs`
- `crates/mapflow/src/app/loops/render.rs`

### Regressionstests

- `tests::redraws_all_windows_when_projector_outputs_exist`
- `tests::keeps_main_window_only_when_idle_without_projectors`
- `preview_flag_targets_same_projector_render_ops`

---

## Pflege-Regel

Wenn ein Bug mehr als einen Suchpfad gebraucht hat oder spaeter plausibel
wiederkehren koennte, hier kurz eintragen. Lieber 10 klare Eintraege als
nochmal dieselbe Ursache blind suchen.
