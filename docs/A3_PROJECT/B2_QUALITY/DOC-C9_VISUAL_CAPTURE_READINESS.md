# Visual-Capture-Readiness fuer grafische Tests

Stand: 2026-03-14

## Ziel

Diese Notiz beschreibt den aktuellen Stand und die noetigen Anpassungen, damit spaeter automatisierte grafische Tests mit sichtbarem MapFlow-Fenster, Screenshots und optionalen Videoaufnahmen moeglich werden.

Das Zielbild ist:

- MapFlow startet in einem deterministischen Testmodus
- eine definierte Szene oder Fixture wird geladen
- die App fuehrt reproduzierbare Schritte aus
- relevante Fenster oder Outputs werden als Bild oder Video aufgenommen
- die Artefakte koennen automatisiert von einem multimodalen LLM ausgewertet werden

## Aktueller Stand

Der aktuelle Testbestand ist fuer dieses Ziel noch nicht ausreichend, aber ein erster lokaler visueller Regressionstestpfad ist jetzt vorhanden.

### Was bereits umgesetzt ist

- dediziertes Binary `mapflow_visual_harness`
- Screenshot-Export aus einem echten sichtbaren `winit`-Fenster
- Pixelvergleich gegen feste Referenzbilder
- optional fester Screenshot-Ausgabeordner ueber `MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR`
- drei erste lokale Szenarien:
  - `checkerboard`
  - `gradient`
  - `alpha_overlay`

### Was heute schon gut ist

- echte Fenster und echte `wgpu::Surface`-Praesentation existieren im normalen App-Lauf
- Main- und Projektorfenster sind fuer `COPY_SRC` vorbereitet
- es gibt bereits GPU-Readback-Code im Renderpfad
- Output-Previews und Texturvorschauen existieren schon intern

### Was heute noch fehlt

- kein dedizierter GUI-Test-Harness fuer die komplette MapFlow-App auf Basis des echten `run_app`-Pfads
- keine stabile CLI oder Test-API fuer reproduzierbare UI-Szenarien
- kein allgemeiner Screenshot- oder Videoexport fuer Tests
- keine Artefakt-Konvention fuer spaetere automatische Auswertung
- keine Trennung zwischen headless GPU-Tests und sichtbaren GUI-Tests

## Technischer Ist-Befund

Die meisten aktuellen Tests pruefen Logik, nicht sichtbare UI.

Beispiele:

- `crates/mapmap-ui/tests/timeline_automation_tests.rs`
- diverse `#[cfg(test)]`-Bereiche in UI-Panels und Editoren

Die vorhandenen GPU-Tests laufen offscreen und sind derzeit ignoriert:

- `crates/mapmap-render/tests/effect_chain_tests.rs`
- `crates/mapmap-render/tests/effect_chain_integration_tests.rs`

Der echte Fensterpfad liegt im Produktionslauf:

- `crates/mapmap/src/main.rs`
- `crates/mapmap/src/app/core/init.rs`
- `crates/mapmap/src/window_manager.rs`
- `crates/mapmap/src/app/loops/render.rs`

Wesentliche Schlussfolgerung:

- fuer die volle MapFlow-App gilt das weiterhin
- als erster Zwischenschritt existiert jetzt aber ein kleiner lokaler visueller Harness fuer echte sichtbare Fenster und Screenshot-Regressionen

## Wichtigste Blocker

### 1. App-Lifecycle ist nicht testfreundlich gekapselt

Der sichtbare GUI-Pfad haengt am echten `ApplicationHandler` und `event_loop.run_app(...)`.

Fuer spaetere GUI-Automation braucht es einen dedizierten Einstiegspunkt, der:

- den Event-Loop kontrolliert startet
- eine Fixture oder Testszene laedt
- feste Aktionen ausfuehrt
- Capture-Trigger setzt
- sauber wieder beendet

### 2. App-Initialisierung ist fuer GUI-Tests zu schwergewichtig

Der aktuelle Startpfad initialisiert unter anderem:

- Audio
- MCP
- Hue
- MIDI
- Bevy

Fuer robuste visuelle Tests sollte es einen abgespeckten Testmodus geben, der diese Seiteneffekte gezielt deaktiviert oder mockt.

### 3. Capture ist nur teilweise vorhanden

Der bestehende Readback-Code im Renderloop ist eine gute Grundlage, aber noch kein allgemeiner Test-Capture-Mechanismus.

Es fehlt noch:

- PNG-Speicherung
- Serienaufnahme fuer mehrere Frames
- optional Videoexport
- klare Benennung und Metadaten fuer Artefakte

### 4. Timing ist noch nicht deterministisch genug

Interne Previews werden aktuell gedrosselt aktualisiert. Fuer reproduzierbare Screenshots muss der Capture-Punkt exakt steuerbar sein, zum Beispiel:

- nach N stabilen Frames
- nach geladenem Fixture-Zustand
- nach explizitem Automationsschritt

## Erforderliche Anpassungen

### Phase 1: GUI-Test-Harness schaffen

Noetig:

- sichtbaren Test- oder Automationsmodus fuer MapFlow einfuehren
- Einstiegspunkt aus `main.rs` in wiederverwendbare Bausteine extrahieren
- Fixture-Projekte automatisiert laden koennen
- Testschritte von aussen parametrisieren

Empfohlene Richtung:

- eigener CLI-Schalter fuer Automation
- klarer Testmodus statt Missbrauch normaler `#[test]`-Funktionen

### Phase 2: Capture im Renderpfad generalisieren

Noetig:

- vorhandenen GPU-Readback-Code aus `crates/mapmap/src/app/loops/render.rs` verallgemeinern
- Screenshots fuer Main-Window und Projektorfenster speichern
- Artefakte als PNG ablegen
- optional Serienaufnahme fuer spaetere Videobildung bereitstellen

Sinnvolle Metadaten pro Artefakt:

- Testfall-ID
- Szene oder Fixture-Name
- Frame-Index
- Fenster-Typ
- Timestamp
- optional Hash oder Build-Info

### Phase 3: Deterministische Testfaelle definieren

Noetig:

- kleine Fixture-Projekte fuer Kernfeatures
- stabile Testschritte
- definierte Erwartungsbilder oder Erwartungsbereiche
- klare Zuordnung zur bestehenden Testmatrix

Empfohlene erste Zielkandidaten:

- Main-UI Startbild
- Output-Preview
- einzelner Projektor-Output
- Medienwiedergabe nach Laden einer Testszene
- Renderzustand eines Kern-Features aus der Regressionsmatrix

## Priorisierte visuelle Testszenarien

Die folgenden Faelle sind fuer einen spaeteren echten MapFlow-App-Harness besonders wertvoll,
weil headless Runner oder reine Logiktests dort nicht genug Signal liefern:

- Main-Window Startzustand mit leerem Testprojekt
  - prueft Fensteraufbau, Docking-Zustand, Initial-Rendering und fehlende Panels
- Output-Preview mit Test-Grid
  - prueft den sichtbaren Renderpfad bis ins Preview-Fenster statt nur Datenmodelle
- Projektor-Output mit Keystone-, Warp- oder Maskenfixture
  - findet Geometrie- und Sampling-Regressionsfehler, die in Unit-Tests leicht unsichtbar bleiben
- Medienwiedergabe mit festem Referenzframe
  - prueft Decoding, Texturupload, Present und sichtbare Farb-/Alpha-Ergebnisse zusammen
- Timeline- oder Automationsschritt mit definierter Vorher/Nachher-Aufnahme
  - prueft, dass nicht nur der State korrekt ist, sondern der sichtbare Zustand wirklich umschaltet

Die drei bereits implementierten Harness-Szenarien decken den unteren technischen Unterbau ab:

- `checkerboard`: Orientierung, harte Kanten, Kanalvertauschungen, echte Surface-Praesentation
- `gradient`: weiche Verlaeufe, Capture-Readback, sichtbare Orientierung
- `alpha_overlay`: Alpha-Blending im echten Fensterpfad

### Phase 4: CI-Integration auf self-hosted Runner

Sichtbare GUI-Automation sollte spaeter nur auf einem geeigneten self-hosted Windows-Runner laufen.

Wichtige Betriebsbedingung:

- fuer wirklich sichtbare Fenster sollte der Runner interaktiv in einer angemeldeten Sitzung laufen, nicht nur als reiner Hintergrunddienst

Zusaetzlich sicherstellen:

- Sleep aus
- Bildschirm nicht gesperrt
- keine konkurrierende manuelle Nutzung waehrend Testlaeufen
- fester Capture-Ordner fuer Artefakte

## Konkrete Minimal-Roadmap

1. Testmodus fuer App-Start und Fixture-Load extrahieren
2. Screenshot-Export aus dem vorhandenen Render-Readback ableiten
3. einen ersten sichtbaren Smoke-Test fuer das Hauptfenster erstellen
4. Artefakte als CI-Artefakte ablegen
5. erst danach Serienaufnahme oder multimodale Auswertung erweitern

### Lokale Nutzung des Automation-Modus

Der neue `run_app`-basierte Automationsmodus kann lokal zum Erstellen von deterministischen Screenshots verwendet werden.

Die benoetigten CLI-Parameter sind:

*   `--mode automation`: Aktiviert den Automationsmodus, welcher schwergewichtige Dienste wie MIDI, Hue, MCP und Audio-Ausgabe umgeht.
*   `--fixture <PFAD_ZUM_PROJEKT>`: (Optional) Laedt sofort beim Start die angegebene `.mflow` Projektdatei.
*   `--exit-after-frames <ANZAHL>`: (Optional) Beendet die Applikation automatisch, nachdem exakt diese Anzahl an Frames gerendert wurde.
*   `--screenshot-dir <PFAD_ZUM_ORDNER>`: (Optional) Wenn angegeben, wird *direkt vor dem automatischen Beenden* (also nach `exit-after-frames`) ein Frame-Buffer-Readback ausgeloest und das Bild als `automation_frame_<ANZAHL>.png` in diesem Ordner abgelegt. Alternativ kann die Umgebungsvariable `MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR` verwendet werden.

**Beispielaufruf lokal:**

```bash
cargo run --bin mapflow -- --mode automation \
  --fixture ./tests/fixtures/test_project.mflow \
  --exit-after-frames 60 \
  --screenshot-dir ./scripts/archive/logs/screenshots
```

Damit laedt MapFlow das Test-Projekt, laesst es 60 Frames laufen (ausreichend, um sicherzustellen, dass Texturen und Shader geladen und berechnet sind), speichert einen Screenshot und beendet sich vollautomatisch, ohne auf Benutzereingaben zu warten.

## Pragmatische Empfehlung

Nicht versuchen, sichtbare GUI-Tests in das aktuelle `#[test]`-Muster zu pressen.

Besser:

- ein dedizierter Automationsmodus fuer die echte App
- eigene Fixtures
- kontrollierter Capture-Punkt
- spaeter eine kleine Anzahl hochwertiger visueller Regressionstests statt vieler fragiler UI-Tests
