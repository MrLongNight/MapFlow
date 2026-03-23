## Roadmap Referenzen

*   **MF-020-TIMELINE-KEYFRAME-INTERACTION:** Betrifft Phase 1 (Keyframe-Editor) dieses Dokuments.
*   **MF-044-TIMELINE-ADJUSTMENTS:** Betrifft Phase 0 (Datenmodell-Schluss) und Phase 2 (Transport-Optionen).

# DOC-C10_TIMELINE_GAP_ANALYSIS

Stand: 2026-03-10

## Ziel

Dieses Dokument beschreibt den aktuellen Implementierungsstand der Timeline in VJMapper, vergleicht ihn mit den auf den offiziellen Webseiten dokumentierten Timeline-/Animationsfunktionen von MadMapper und Resolume und leitet daraus die naechsten noetigen Anpassungen und Erweiterungen ab.

## Gepruefte Referenzen

### Lokale Codebasis

- `crates/mapflow-ui/src/editors/timeline_v2.rs`
- `crates/mapflow-core/src/animation.rs`
- `crates/mapflow-core/src/effect_animation.rs`
- `crates/mapflow/src/app/loops/logic.rs`
- `crates/mapflow/src/app/actions.rs`
- `crates/mapflow-ui/tests/timeline_automation_tests.rs`
- `crates/mapflow-ui/src/panels/cue_panel.rs`
- `docs/A3_PROJECT/B2_QUALITY/DOC-C1_TEST_MATRIX.md`

### Offizielle externe Referenzen

- MadMapper Home: https://madmapper.com/
- MadMapper v6 Doku: https://docs.madmapper.com/madmapper/6/what%27s-new/madmapper-v6
- Resolume Parameter Animation: https://resolume.com/support/en/parameter-animation
- Resolume Dashboard: https://resolume.com/support/en/dashboard
- Resolume SMPTE: https://resolume.com/support/en/smpte

## Aktueller Implementierungsstand in VJMapper

### Bereits vorhanden

- Bottom-panel Timeline UI ist in der Haupt-App eingebunden.
- Basis-Transport ist vorhanden: `Play`, `Pause`, `Stop`, `Seek`.
- Scrubbing ueber den Time-Ruler ist vorhanden.
- Globales `Loop` fuer den Clip ist vorhanden.
- Snap in Sekunden ist vorhanden.
- Zoom der Timeline ist vorhanden.
- Keyframes und Kurven werden visualisiert.
- Das Core-Datenmodell unterstuetzt `Constant`, `Linear`, `Smooth` und `Bezier` Interpolation.
- Eine einfache Module-Arrangement-Spur ist vorhanden.
- Die Show-Control-Spur unterstuetzt drei Modi:
  - `FullyAutomated`
  - `SemiAutomated`
  - `Manual`
- Im Runtime-Loop kann die Timeline die aktive Modul-Auswertung auf ein einzelnes Modul begrenzen.
- Zwei grundlegende Automations-Tests existieren und laufen erfolgreich:
  - `test_timeline_fully_automated_switch`
  - `test_timeline_manual_mode_no_auto_switch`

### Teilweise vorhanden / strukturell vorbereitet

- `TimelineV2` enthaelt State fuer `selected_keyframes`, `show_curve_editor` und `pan_offset`, diese werden aber aktuell nicht als vollwertige Editor-Funktionen genutzt.
- `EffectParameterAnimator` besitzt APIs fuer `bind_parameter`, `add_keyframe`, `remove_keyframe`, `set_duration` und Playback.
- Das Animationsmodell ist damit vorhanden, die Authoring-/Playback-Kette ist aber nicht geschlossen.

### Fehlende oder unvollstaendig verdrahtete Bereiche

- Kein vollwertiges Keyframe-Authoring in der Timeline:
  - keine Auswahl-Logik im UI
  - kein Drag/Move
  - kein Delete
  - kein Add-Keyframe-Workflow im Editor
  - kein Copy/Paste oder Duplicate
- Kein echter Curve-Editor trotz vorbereitetem State.
- Kein Track-Management fuer Automation aus der Timeline heraus.
- Keine sichtbare produktive Bindung von Effektparametern an den `EffectParameterAnimator` ausserhalb von Tests.
- Die von `effect_animator.update()` erzeugten Parameter-Updates werden im Hauptloop aktuell nicht auf Effekte zurueckgeschrieben.
- Cue-System und Timeline sind nicht integriert.
- Die Cue-UI erzeugt `UIAction::AddCue`, `GoCue`, `NextCue`, `PrevCue`, `StopCue`, `UpdateCue`, `RemoveCue`, aber in `crates/mapflow/src/app/actions.rs` ist dafuer aktuell kein Handler sichtbar.
- Testabdeckung ist sehr schmal:
  - keine Tests fuer `SemiAutomated`
  - keine UI-Interaktionstests
  - keine Tests fuer Keyframe-CRUD
  - keine Tests fuer Persistenz
  - keine Tests fuer Cue-/Timeline-Integration

## Kurzvergleich mit MadMapper und Resolume

| Bereich | VJMapper heute | MadMapper | Resolume | Delta fuer VJMapper |
| --- | --- | --- | --- | --- |
| Basis-Transport | Vorhanden | Vorhanden | Vorhanden | Kein Hauptgap |
| Parameteranimation | Teilweise vorhanden | Stark ausgebaut | Stark ausgebaut | Authoring und Anwendung fehlen |
| Kurven / Interpolation | Datenmodell vorhanden, UI schwach | Echtzeit-Kurven + Interpolation | Umfangreiche Parameteranimation | Editor und Workflow fehlen |
| Keyframe-CRUD | Fehlt praktisch | Vorhanden | Vorhanden | Muss kurzfristig geschlossen werden |
| Mehrere Timelines / Show-Uebersicht | Fehlt | Timelines Grid + Conductor | Eher clip-/layer-zentriert, aber stark steuerbar | Mittelfristig noetig |
| Clips / Montage Tracks | Fehlt | Montage Tracks + Animation Clips | Clip-/Layer-zentrierte Animation | Mittelfristig noetig |
| Marker / Follow Actions / Goto | Fehlt | Vorhanden | Start-/Trigger-Logik und Transportmodi vorhanden | Hoch relevant fuer Show-Steuerung |
| Audio-Track / Beat Snap | Fehlt | FFT + BeatTracking zu Snap-Punkten | Audio Analysis + BPM Sync | Hoch relevant |
| In/Out, Reverse, Ping-Pong, Random, One-Shot | Fehlt | Teilweise ueber Timeline/Clips dokumentiert | Vorhanden | Hoher UX-Gap |
| Dashboard-/Performance-Macro-Steuerung | Fehlt in Timeline | Nicht Kernfokus | Dashboard stark | Mittel |
| Externe Zeitsynchronisation | Fehlt | Nicht Kernreferenz hier | SMPTE vorhanden | Wichtig fuer Pro-Show-Control |
| Export / Bake / Offline Render | Fehlt | Export aus Timeline vorhanden | Nicht Kern des Vergleichs hier | Mittel bis hoch |

## Wichtige Erkenntnisse aus dem Vergleich

### 1. VJMapper hat aktuell eher einen Timeline-Viewer als einen Timeline-Editor

Die aktuelle Timeline kann anzeigen, scrubben und ein einfaches Modul-Arrangement fahren. Sie bietet aber noch nicht die Editor-Tiefe, die Anwender von MadMapper oder Resolume bei Animation und Show-Steuerung erwarten.

### 2. Das Datenmodell ist weiter als die UI- und Runtime-Integration

Interpolation, Tracks und ein Animator existieren bereits im Core. Der eigentliche Engpass ist nicht das Modell, sondern:

- fehlende Authoring-UI
- fehlende Bindings aus dem Produktivcode
- fehlende Anwendung der Animator-Updates auf echte Parameter

### 3. Show-Control ist doppelt angelegt, aber nicht vereinheitlicht

Es gibt:

- eine Module-Show-Spur in der Timeline
- ein separates Cue-System mit Crossfades und Triggern

Diese beiden Systeme arbeiten derzeit nicht als ein gemeinsames Show-Control-Modell. Im Vergleich dazu wirken MadMapper und Resolume kohaerenter.

### 4. Der groesste Wettbewerbsabstand liegt nicht im Rendering, sondern im Workflow

MadMapper und Resolume liefern vor allem:

- schnellere Authoring-Workflows
- mehr Playback-Modi
- Trigger-/Follow-/Clock-Logik
- bessere Improvisations- und Live-Steuerungsmoeglichkeiten

Genau dort ist VJMapper aktuell am weitesten entfernt.

## Priorisierter Ausbauplan

## Phase 0 - Bestehenden Pfad schliessen

Ziel: Aus vorhandenem Datenmodell eine funktionierende End-to-End Automation machen.

Arbeitspakete:

1. Produktive Bindings fuer animierbare Parameter einfuehren.
2. `effect_animator.update()` Ergebnisse auf echte Effekt-/Modulparameter anwenden.
3. Timeline-Dirty-State und Projektpersistenz fuer Automation pruefen und absichern.
4. Offene Cue-UI-Action-Handler in der Haupt-App vervollstaendigen.

Definition of Done:

- Ein animierter Parameter laesst sich im Projekt anlegen, speichern, abspielen und nach Reload unveraendert wiedergeben.
- Cue-Aktionen aus dem UI fuehren zu realem Runtime-Verhalten.

## Phase 1 - Keyframe-Editor fertigstellen

Ziel: Mindeststandard eines brauchbaren Timeline-Editors.

Arbeitspakete:

1. Keyframe anlegen.
2. Keyframe selektieren.
3. Keyframe verschieben.
4. Keyframe loeschen.
5. Multi-Select.
6. Box-Selection.
7. Copy/Paste und Duplicate.
8. Undo/Redo fuer Timeline-Operationen.
9. Sichtbarer Curve-Editor pro Track.

Definition of Done:

- F-020 aus `DOC-C1_TEST_MATRIX.md` ist technisch geschlossen.
- UI-Interaktionen sind per Tests oder reproduzierbaren manuellen Testfaellen abgesichert.

## Phase 2 - Transport und Timing professionalisieren

Ziel: Aufholen gegen Resolume im Bereich Playback-Optionen.

Arbeitspakete:

1. In/Out-Punkte.
2. Reverse-Playback.
3. Ping-Pong.
4. One-Shot.
5. Per-Track oder per-Clip Loop-Modi.
6. BPM-Sync-Modus.
7. Raster/Snap in Takten und musikalischen Divisionen, nicht nur in Sekunden.
8. Zeitlineal mit Bars/Beats umschaltbar.

Definition of Done:

- Eine Animation kann in Sekunden oder Beats editiert werden.
- Playback-Modi sind im UI explizit sichtbar und reproduzierbar testbar.

## Phase 3 - Timeline von "Trackliste" zu "Show-Struktur" ausbauen

Ziel: Aufholen gegen MadMapper im Bereich Show-Komposition.

Arbeitspakete:

1. Mehrere Timelines oder Sequences pro Projekt.
2. Clips/Segments auf Tracks.
3. Montage-/Arrangement-Tracks fuer Medien und Automation.
4. Marker.
5. Follow Actions.
6. Goto/Jump Logik.
7. Bird's-eye Show Overview.
8. Exklusivitaetsregeln fuer konkurrierende Sequenzen.

Definition of Done:

- Ein Nutzer kann ein ganzes Show-Set nicht nur als lineare Parameterkurve, sondern als organisierte Sequenzstruktur bauen.

## Phase 4 - Cue-System und Timeline zusammenfuehren

Ziel: Ein gemeinsames Show-Control-Modell statt zweier halber Systeme.

Empfohlene Richtung:

- Cue = Snapshot/Trigger-Ereignis
- Timeline Clip/Marker = zeitliche Ausloesung oder Segment
- Module Arrangement = Spezialfall eines Cue-/Sequence-Blocks

Arbeitspakete:

1. Gemeinsames Datenmodell fuer Cue, Marker, Sequence, Clip.
2. `GO`, `NEXT`, `PREV`, `HOLD`, `AUTO-FOLLOW`.
3. Crossfades und Follow Actions in Timeline-Marker integrieren.
4. OSC/MIDI/Time-Trigger direkt in Timeline-Events nutzbar machen.

Definition of Done:

- Ein Operator kann zwischen manueller Show-Fahrt, halbautomatischer Show und vollautomatischer Timeline umschalten, ohne verschiedene Systeme mental auseinanderhalten zu muessen.

## Phase 5 - Audio-, Clock- und External-Sync

Ziel: Pro-Show-Control und Musik-Synchronisation.

Arbeitspakete:

1. Audio-Tracks oder mindestens Audio-Referenzspuren in der Timeline.
2. Beat Detection sichtbar in der Timeline.
3. Snap to Beat / transient markers.
4. MIDI Clock oder MTC.
5. SMPTE-Input.
6. Optional SMPTE-/Clock-Offset und Delay-Kompensation.

Definition of Done:

- Timeline kann entweder intern laufen oder sich an externe Show-Clock haengen.

## Phase 6 - Export und Produktionsmodus

Ziel: MadMapper-nahe Produktionsfaehigkeit.

Arbeitspakete:

1. Offline Export/Bake von Timeline-Sequenzen.
2. Export ganzer Show-Segmente.
3. Optional Loop-Range Export.
4. Reproduzierbarer "Playback only" Modus fuer Live-Betrieb.

## Konkreter Backlog-Vorschlag

### Kurzfristig

- TL-01: Keyframe CRUD in `timeline_v2.rs`
- TL-02: Produktive Parameter-Bindings fuer Animation
- TL-03: Anwenden von Animator-Updates auf Laufzeitparameter
- TL-04: Cue-Actions in `actions.rs` verdrahten
- TL-05: Semi-Auto Tests und Persistenztests erweitern

### Mittelfristig

- TL-06: Curve Editor
- TL-07: BPM-Snap / Beat-Ruler
- TL-08: In/Out, Reverse, Ping-Pong, One-Shot
- TL-09: Marker + Follow Actions
- TL-10: Sequence-/Clip-Struktur statt nur Track-Kurven

### Spaeter / Pro-Funktionen

- TL-11: Conductor-/Overview-Ansicht
- TL-12: SMPTE/MTC
- TL-13: Offline Export
- TL-14: Vereinheitlichtes Cue-/Timeline-Modell

## Empfohlene Reihenfolge fuer die Umsetzung

1. Phase 0 und Phase 1 zuerst.
2. Danach Phase 2, damit die Timeline im Alltag benutzbar wird.
3. Anschliessend Phase 4, damit nicht zwei konkurrierende Show-Control-Systeme weiterwachsen.
4. Phase 3, 5 und 6 danach als Ausbau auf Pro-Niveau.

## Fazit

VJMapper besitzt bereits ein brauchbares Fundament fuer Timeline-Datenmodell, Playback-Basis und einfache modulbasierte Show-Steuerung. Gegenueber MadMapper und Resolume fehlt aber noch der groesste Teil der eigentlichen Produktions- und Live-Workflow-Faehigkeit.

Die wichtigste naechste Entscheidung sollte daher nicht "noch mehr Timeline-Features" sein, sondern:

- erstens den bestehenden Automationspfad funktional schliessen
- zweitens Keyframe-Editing wirklich fertigstellen
- drittens Cue-System und Timeline zu einem gemeinsamen Show-Control-Konzept zusammenziehen

Erst danach lohnt sich der Ausbau in Richtung Montage Tracks, Marker-Logik, Audio-Snap und externe Zeitsynchronisation.
