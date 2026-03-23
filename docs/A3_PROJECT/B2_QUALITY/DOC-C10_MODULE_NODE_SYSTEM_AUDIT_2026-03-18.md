# DOC-C10: Module Node System Audit (IST, Probleme, SOLL)

Stand: 2026-03-18

## 1. Scope

Diese Doku analysiert den aktuellen IST-Zustand des Module-Canvas-Node-Systems in MapFlow/VJMapper:

- verfuegbare Node-Typen im Core
- Erreichbarkeit im Module Canvas
- aktuelle Inspector-Funktionen
- reale Runtime-/Render-Unterstuetzung
- gefundene Logikfehler und Drift zwischen Core, UI und Renderpfad
- Vorschlag fuer animiertes, individuelles Nodelayout
- Vorschlag fuer ein besseres Trigger-/Connector-Konzept

Primaere Referenzen:

- `crates/mapmap-core/src/module/types/*`
- `crates/mapmap-core/src/module_eval/*`
- `crates/mapmap-core/src/trigger_system.rs`
- `crates/mapmap-ui/src/editors/module_canvas/*`
- `crates/mapmap/src/orchestration/*`
- `crates/mapmap/src/app/loops/render.rs`
- `crates/mapmap-media/src/pipeline.rs`
- `docs/A1_SYSTEM/B1_ARCHITECTURE/DOC-C4_RENDER-QUEUE.md`

## 2. Kurzfazit

Der Node-Stack ist funktional nur in einem Teilbereich konsistent:

- Der Core modelliert deutlich mehr Node-Typen, Parameter und Socket-Varianten als das Canvas, der Inspector und der Renderpfad real bedienen.
- Die derzeitige Runtime ist keine einheitliche "Render Queue", sondern ein Mix aus direkter Media-Orchestrierung, `render_ops` aus dem Evaluator und einer ungenutzten Frame-Pipeline im `mapmap-media`-Crate.
- Trigger, Link-System, Hue, NDI/Spout/LiveInput, Blend-Modi, Masken und mehrere Effekte sind nur teilweise oder gar nicht end-to-end verdrahtet.
- Das derzeitige Connector-Modell ist index-basiert, zu lose typisiert und erlaubt ungueltige Verbindungen, die spaeter in Evaluator und Renderpfad zu stillen Fehlern fuehren.
- Fuer einen belastbaren SOLL-Zustand braucht das System eine Schema-getriebene Node-Definition mit stabilen Socket-IDs, klaren Connector-Klassen und einer sauberen Trennung zwischen `Media`, `Control` und `Event`.

## 3. Node-Inventar nach IST-Zustand

### 3.1 Trigger Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `Beat` | Ja | Ja | Ja | Ja | Basisfunktion ok |
| `AudioFFT` | Ja | Ja | Ja | Teilweise | `band` ist praktisch tote Metadaten; dynamische Outputs werden im UI nicht sauber nachgefuehrt |
| `Random` | Ja | Ja | Ja | Ja | ok |
| `Fixed` | Ja | Ja | Ja | Ja | ok |
| `Midi` | Ja | Ja | Ja | Teilweise | Device-Feld ist im Inspector faktisch nicht bindbar; Learn unterscheidet Note/CC nicht sauber |
| `Osc` | Ja | Ja | Ja | Ja | ok, aber nur Basisadresse |
| `Shortcut` | Ja | Ja | Ja | Teilweise | Modifier-Anzeige im Inspector ist vertauscht |

Inspector-Status Trigger:

- `Beat`: nur Info-Text.
- `AudioFFT`: Threshold, Output-Konfiguration, Invertierung.
- `Random`: Min/Max-Intervall, Probability.
- `Fixed`: Intervall, Offset, Live-Vorschau.
- `Midi`: Portliste, Channel, Note, Learn.
- `Osc`: Address.
- `Shortcut`: Key-Feld, Modifier-Statusanzeige.

Wichtige Probleme:

- `AudioFFT.band` wird im Evaluator nicht fuer die Berechnung verwendet; relevant sind nur `threshold` und `output_config`.
- Die Inspector-UI aendert `output_config`, aber die gespeicherten `part.outputs` werden danach nicht automatisch neu berechnet.
- `Shortcut` nutzt laut Evaluator `1=Shift, 2=Control, 4=Alt`, die Inspector-Anzeige labelt `1` aber als `Ctrl` und `2` als `Shift`.
- `Midi` hat nur ein Feld `note`, wird aber intern sowohl fuer Note als auch CC verwendet.

### 3.2 Source Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `MediaFile` | Ja | Ja | Ja | Ja | am weitesten verdrahtet |
| `VideoUni` | Ja | indirekt ueber Inspector | Ja | Ja | ok, aber Transform nur teilweise gerendert |
| `ImageUni` | Ja | indirekt ueber Inspector | Ja | Ja | ok, aber Transform nur teilweise gerendert |
| `VideoMulti` | Ja | indirekt ueber Inspector | Ja | Ja | ok, aber Transform nur teilweise gerendert |
| `ImageMulti` | Ja | indirekt ueber Inspector | Ja | Ja | ok, aber Transform nur teilweise gerendert |
| `Shader` | Ja | Ja | Ja | Nein gefunden | Inspector nur Name, kein echter Runtime-Pfad sichtbar |
| `LiveInput` | Ja | Nein | Ja | Nein gefunden | Core-only/Inspector-only |
| `NdiInput` | Ja | Ja | Ja | Teilweise | Connect-Action vorhanden, aber kein Frame-Poll/Upload im App-Lauf gefunden |
| `SpoutInput` | Ja | Ja (Windows) | Ja | Nein gefunden | Core/UI vorhanden, Runtime-Pfad nicht gefunden |
| `Bevy` | Ja | Nein | Ja | Teilweise | nur als generische Bevy-Szene praesent |
| `BevyAtmosphere` | Ja | Nein | Ja | Ja ueber BevyRunner | creatable im Canvas nicht sichtbar |
| `BevyHexGrid` | Ja | Nein | Ja | Ja ueber BevyRunner | creatable im Canvas nicht sichtbar |
| `BevyParticles` | Ja | Nein | Ja | Ja ueber BevyRunner | creatable im Canvas nicht sichtbar |
| `Bevy3DShape` | Ja | Ja | Ja | Ja ueber BevyRunner | relativ gut verdrahtet |
| `Bevy3DModel` | Ja | Nein | Platzhalter | Teilweise | Inspector nicht implementiert |
| `Bevy3DText` | Ja | Ja | Ja | Ja ueber BevyRunner | ok |
| `BevyCamera` | Ja | Ja | Ja | Ja ueber BevyRunner | ok |

Inspector-Status Sources:

- `MediaFile`/`VideoUni`: File Picker, Transport, Timeline, Speed, Reverse, Common Controls.
- `ImageUni`: File Picker, Common Controls.
- `VideoMulti`/`ImageMulti`: Shared-ID Auswahl, Common Controls.
- `Shader`: nur `name`.
- `LiveInput`: nur `device_id`.
- `NdiInput`: Discover/Connect/Disconnect.
- `SpoutInput`: nur `sender_name`.
- `Bevy3DText`: Text, Fontgroesse, Farbe, Alignment, Position/Rotation.
- `BevyCamera`: Active, FOV, Modus + Modus-Parameter.
- `BevyAtmosphere`: Atmosphaeren-Parameter.
- `BevyHexGrid`: Radius/Ringe/Spacing/Transform/Scale.
- `BevyParticles`: Rate/Lifetime/Speed/Farben/Transform.
- `Bevy3DShape`: Shape, Farbe, Unlit, Transform, Outline.
- `Bevy3DModel`: nur Platzhaltertext.
- `Bevy`: nur Info-Text.

Wichtige Probleme:

- Der Source-Type-Picker im Inspector kann nur zwischen `MediaFile`, `VideoUni`, `ImageUni`, `VideoMulti`, `ImageMulti` wechseln. Alle anderen Source-Typen sind zwar im `match` der Inspector-UI vorhanden, aber nicht ueber den Picker erreichbar.
- `target_width`, `target_height` und `target_fps` existieren im Core fuer mehrere File-Varianten, werden aber im Inspector nicht angeboten.
- Der "Reverse"-Button in den Transport-Controls toggelt nur den lokalen Zustand, sendet dort aber keinen `MediaCommand::SetReverse`.
- Das zusaetzliche Seek-Slider-UI nutzt eine harte `300.0`-Sekunden-Annahme statt der echten Mediendauer.
- Im Renderpfad werden `scale_x`, `scale_y`, `rotation`, `offset_x`, `offset_y` aus `SourceProperties` aktuell nicht in die Geometrie-Transformation ueberfuehrt.

### 3.3 Mask Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `MaskType::File` | Ja | Nein | Ja | Nein | UI vorhanden, Renderpfad ignoriert `op.masks` |
| `MaskType::Shape` | Ja | Ja | Ja | Nein | UI vorhanden, Renderpfad ignoriert `op.masks` |
| `MaskType::Gradient` | Ja | Ja | Ja | Nein | UI vorhanden, Renderpfad ignoriert `op.masks` |

Wichtige Probleme:

- Der Evaluator sammelt Masken in `RenderOp.masks`, der Renderpfad verwendet diese Liste aber nicht.
- `MaskType::File` ist im Core und Inspector vorhanden, im Node-Katalog bzw. im Create-Menue aber nicht sichtbar.

### 3.4 Modulizer / Effect Nodes

#### Modulizer-Typen

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `ModulizerType::Effect` | Ja | Ja | Ja | Teilweise | nur ein Teil der Effekte ist renderbar |
| `ModulizerType::BlendMode` | Ja | Ja | Ja | Nein | wird in `build_effect_chain` ignoriert |
| `ModulizerType::AudioReactive` | Ja | Nein sichtbar | Ja | Nein | Inspector vorhanden, Renderpfad ignoriert Typ |

#### EffectType-Matrix

Renderbar laut `map_effect_type`:

- `ShaderGraph`
- `Blur`
- `Invert`
- `HueShift`
- `Wave`
- `Mirror`
- `Kaleidoscope`
- `Pixelate`
- `EdgeDetect`
- `Glitch`
- `RgbSplit`
- `ChromaticAberration`
- `FilmGrain`
- `Vignette`

Im Core definiert, im Canvas teils creatable, aber im Renderpfad derzeit nicht gemappt:

- `LoadLUT`
- `Brightness`
- `Contrast`
- `Saturation`
- `Colorize`
- `Sharpen`
- `Threshold`
- `Spiral`
- `Pinch`
- `Halftone`
- `Posterize`
- `VHS`

Inspector-Parameter aktuell nur fuer:

- `Blur`
- `Pixelate`
- `FilmGrain`
- `Vignette`
- `ChromaticAberration`
- `Brightness`
- `Contrast`
- `Saturation`

Wichtige Probleme:

- Der Inspector listet nur einen Teil der `EffectType`-Varianten im Wechselmenue.
- Mehrere Effekte sind im Core deklariert und ueber Quick Create indirekt erzeugbar, aber nicht renderbar.
- `Brightness`, `Contrast`, `Saturation` haben Inspector-Controls, werden aber aktuell gerade nicht in `map_effect_type` auf den Renderer gemappt.
- `BlendMode`- und `AudioReactive`-Modulatoren werden im Evaluator zwar in die Chain aufgenommen, spaeter aber verworfen.
- Die "Opacity"-UI bei `BlendMode` und die "Smoothing"-UI bei `AudioReactive` schreiben nur in temporare lokale Werte und haben keine dauerhafte Wirkung.

### 3.5 Mesh Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `MeshType::*` als eigener Node | Ja | Nein | Ja | Teilweise | existiert im Modell und Traversal, aber nicht als regulaer creatable Canvas-Node |

Core-Mesh-Varianten:

- `Quad`
- `Grid`
- `BezierSurface`
- `Polygon`
- `TriMesh`
- `Circle`
- `Cylinder`
- `Sphere`
- `Custom`

Wichtige Probleme:

- Ein eigener `ModulePartType::Mesh` wird im Canvas nicht angeboten.
- `trace_chain_into` kann Mesh-Nodes als Geometrie-Override nutzen, aber die regulaere UI bietet keinen konsistenten Weg, solche Nodes neu anzulegen.

### 3.6 Layer Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `LayerType::Single` | Ja | Ja | Ja | Ja | Basisfall funktioniert |
| `LayerType::Group` | Ja | Ja | Ja | Teilweise | wird wie `Single` behandelt, keine Gruppenlogik |
| `LayerType::All` | Ja | Ja | Teilweise | Nein | im Evaluator explizit nicht gerendert |

Inspector-Status:

- `Single`: ID, Name, Opacity, Blend, Mapping Mode, Mesh Editor, Preview.
- `Group`: Name, Opacity, Mapping Mode, Mesh Editor.
- `All`: nur Opacity.

Wichtige Probleme:

- `LayerType::Group` hat keine echte Gruppensemantik; der Evaluator behandelt die Variante faktisch wie `Single`.
- `LayerType::All` wird im Evaluator beim Render-Op-Aufbau uebersprungen.
- Ein Layer hat nur einen Media-Eingang; damit existiert derzeit keine echte Layer-Stack-/Compositing-Struktur im Graph.

### 3.7 Hue Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `HueNodeType::SingleLamp` | Ja | Ja | Nein | Nein gefunden | halb verdrahtet |
| `HueNodeType::MultiLamp` | Ja | Nein | Nein | Nein gefunden | Core-only |
| `HueNodeType::EntertainmentGroup` | Ja | Nein | Nein | Nein gefunden | Core-only |

Wichtige Probleme:

- Der Inspector zeigt fuer `ModulePartType::Hue(_)` derzeit nur einen Platzhaltertext.
- Der Evaluator erzeugt zwar `SourceCommand::HueOutput`, aber im App-Lauf wurde kein Verbraucher fuer diese Commands gefunden.
- Der Hue-Node besitzt einen Media-Eingang `Color (RGB)`, die Trigger-/Socket-Auswertung arbeitet aber nur mit `trigger_values`; ein echter RGB-Medienpfad existiert dafuer nicht.

### 3.8 Output Nodes

| Node | Core | Canvas create | Inspector | Runtime | IST-Bewertung |
| --- | --- | --- | --- | --- | --- |
| `Projector` | Ja | Ja | Ja | Teilweise | einziges Ende-zu-Ende brauchbares Output-Konzept |
| `NdiOutput` | Ja | Nein regulaer | Ja | Nein gefunden | Core/UI vorhanden, Laufpfad fehlt |
| `Spout` | Ja | Nein regulaer | Ja | Nein gefunden | Core/UI vorhanden, Laufpfad fehlt |
| `Hue` | Ja | Nein regulaer | Teilweise | Nein gefunden | Konzept vorhanden, Runtime fehlt |

Inspector-Status `Projector`:

- Output-Nummer
- Name
- Target Screen
- Hide Cursor
- Preview-Flags
- NDI an/aus + Streamname

Fehlend im `Projector`-Inspector:

- `output_width`
- `output_height`
- `output_fps`

Wichtige Probleme:

- `sync_output_windows` erzeugt nur Projector-Fenster; alte/stale Fenster werden nicht entfernt.
- `active_window_ids` wird gesammelt, aber fuer Cleanup nicht verwendet.
- `OutputType::Hue` hat Pairing-/Area-UI mit TODOs, aber keine sichtbare End-to-End-Ausfuehrung.
- `OutputType::Hue` hat im Core Layer- und Trigger-Eingang, der Evaluator verwendet fuer das Hue-Kommando aber nur Trigger-Helligkeit und ignoriert den Layer-Inhalt.

## 4. Inspector-System: Querschnittsstatus

### 4.1 Trigger & Automation Panel

Das Panel ist aktuell ein globaler Overlay fuer alle Nodes mit Inputs.

Vorhanden:

- MIDI Learn pro Node
- Mapping-Target pro Input-Socket
- Mapping-Modi `Direct`, `Fixed`, `RandomInRange`, `Smoothed`
- Range, Threshold, Attack, Release, Invert

Derzeit angebotene Targets:

- `None`
- `Opacity`
- `Brightness`
- `Contrast`
- `Saturation`
- `HueShift`
- `ScaleX`
- `ScaleY`
- `Rotation`

Im Core vorhanden, aber im Inspector nicht angeboten:

- `OffsetX`
- `OffsetY`
- `FlipH`
- `FlipV`
- `ParticleRate`
- `Position3D`
- `Scale3D`
- `Param(String)`

Konzeptionelle Probleme:

- Das Panel wird fuer alle Inputs angezeigt, auch fuer reine `Media`- oder `Layer`-Inputs.
- Die Mapping-Logik liest nur aus `trigger_values`; auf nicht-triggerbasierenden Eingangsverbindungen ist das irrefuehrend oder wirkungslos.
- Die Node-UI arbeitet mit Socket-Indizes statt mit stabilen Socket-IDs. Das wird bei dynamischen Outputs (`AudioFFT`) oder Link-Sockets schnell fragil.

### 4.2 Stored Sockets vs. abgeleitete Sockets

`ModulePart` speichert `inputs` und `outputs` redundant, obwohl diese aus `part_type` und `link_data` ableitbar sind.

Folgen im IST:

- Inspector-Aenderungen an dynamischen Trigger-Outputs aktualisieren die sichtbaren Sockets nicht automatisch.
- Das Link-System (`LinkMode`, `LinkBehavior`, `trigger_input_enabled`) ist im Core vorhanden, im Canvas aber praktisch nicht konfigurierbar.
- Preset-Laden nutzt `utils::get_sockets_for_part_type()` statt `compute_sockets()` und kann dadurch falsche Socket-Sets erzeugen.

## 5. Render Queue / Runtime-Logik: IST-Zustand

### 5.1 Was aktuell wirklich laeuft

Der aktive Runtime-Pfad besteht aktuell aus drei Schichten:

1. `orchestration/media.rs`
   - scannt Module direkt nach Media-Quellen
   - erstellt pro Quelle einen Hintergrund-Thread
   - laedt Frames direkt in den `TexturePool`

2. `orchestration/evaluation.rs` + `module_eval`
   - bewertet den aktiven Modulgraph
   - erzeugt `render_ops`
   - synchronisiert Triggerwerte in UI und Bevy

3. `app/loops/render.rs`
   - filtert `render_ops` pro Output
   - sucht Source-Texturen im `TexturePool`
   - rendert Mesh + optionale EffectChain in das Ziel

### 5.2 Was zusaetzlich im Code existiert, aber nicht sichtbar verdrahtet ist

- `mapmap-media/src/pipeline.rs` enthaelt `FramePipeline` und `FrameScheduler`.
- `DOC-C4_RENDER-QUEUE.md` beschreibt diese Pipeline als zentrale Architektur.
- Im eigentlichen App-Lauf wurde jedoch kein aktiver Aufrufpfad auf diese Pipeline gefunden.

Bewertung:

- Die Doku beschreibt derzeit nicht den echten Runtime-Flow.
- Das Projekt hat faktisch zwei konkurrierende "Render Queue"-Geschichten: dokumentierte Frame-Pipeline vs. reale direkte Player-/Upload-Orchestrierung.

### 5.3 RenderOp-Pfad

Der Evaluator erzeugt `RenderOp` pro Output-Node und verfolgt dabei nur den primaeren Visual-Input auf Socket `0`.

Aktuelle Eigenschaften:

- ein Output verfolgt genau eine primaere Renderkette
- Effekte und Masken werden im `RenderOp` gesammelt
- Mesh kann ueberschrieben werden
- SourceProperties werden gesammelt

Der Renderpfad nutzt davon aktuell aber nur einen Teil:

- Quelle/Texture
- Opacity
- Helligkeit/Kontrast/Saettigung/Hue Shift
- Flip H/V
- EffectChain (nur fuer die oben genannte Teilmenge)
- Mesh

Aktuell ignoriert im finalen Render:

- `RenderOp.masks`
- `RenderOp.blend_mode`
- `SourceProperties.scale_x`
- `SourceProperties.scale_y`
- `SourceProperties.rotation`
- `SourceProperties.offset_x`
- `SourceProperties.offset_y`

### 5.4 Weitere Render-/Runtime-Probleme

- Fallback bei fehlender Quelltextur:
  Wenn `bevy_output` existiert, wird es als globaler Fallback verwendet, auch wenn die fehlende Quelle gar kein Bevy-Node ist.
- `render_ops` werden pro Window erneut gefiltert und geklont; das ist funktional ok, aber architektonisch nicht wirklich eine Queue.
- `app.render_ops` speichert semantisch `(ModuleId, RenderOp)`, der Typ ist aber als `Vec<(ModulePartId, RenderOp)>` deklariert.

## 6. Gefundene Probleme und Logikfehler

### P0: Core/UI/Runtime-Drift

- Viele Core-Varianten sind nicht creatable, nicht inspectable oder nicht renderbar.
- Mehrere UI-Controls existieren, ohne dass dahinter eine Runtime-Wirkung folgt.

### P0: Graph wird bei Read-Zugriffen dirty

`ModuleManager::get_module_mut()` erhoeht sofort `graph_revision`.

Folge:

- Schon das normale Rendern des Module Canvas markiert den Graphen als "dirty".
- `graph_dirty` bleibt dadurch praktisch staendig wahr.
- Bevy-Graph-Sync und andere Dirty-abhaengige Pfade koennen unnoetig jede Update-Schleife laufen.

### P0: Ungueltige Verbindungen werden zugelassen

Beim Loslassen einer neuen Verbindung wird nur geprueft:

- anderer Node
- andere Socket-Richtung

Nicht geprueft wird:

- `socket_type`
- semantische Kompatibilitaet
- max. Fan-In/Fan-Out
- Zyklen

Die Preview faerbt unpassende Ziele zwar rot, blockiert die Verbindung aber nicht.

### P0: Socket-Zustaende koennen stale werden

- `part.inputs` und `part.outputs` werden gespeichert statt zentral abgeleitet.
- Nach Inspector-Aenderungen an Triggern/Sockets werden diese Arrays nicht automatisch nachgezogen.
- Presets erzeugen Sockets ueber eine UI-Hilfsfunktion, die vom Core-Modell abweicht.

### P0: Create-Menue/Katalog umgeht Default-Fabriken

Quick Create und Kontextmenue verwenden `add_part_with_type(...)` mit hart kodierten Defaultwerten.

Folgen:

- Projector-Outputs koennen mehrfach mit `id = 1` erzeugt werden.
- `LayerType::Single` startet mehrfach mit `id = 0`.
- Die sichere Output-ID-Vergabe aus `add_part(PartType::Output, ...)` wird umgangen.

### P0: Renderpfad ignoriert deklarierte Features

- Masken werden gesammelt, aber nie angewandt.
- Blend-Modi werden gesammelt, aber nie angewandt.
- Source-Transform wird im Renderpfad nicht umgesetzt.
- `LayerType::All` wird gar nicht gerendert.
- `LayerType::Group` ist keine echte Gruppe.

### P0: Nicht verdrahtete Runtime-Konzepte

- `SourceCommand` wird im Evaluator erzeugt, aber ausserhalb davon praktisch nicht verwendet.
- `Shader`, `LiveInput`, `SpoutInput`, `Hue`-Commands haben keinen sichtbaren aktiven Runtime-Verbraucher im App-Lauf.
- NDI-Input hat Connect-UI, aber keinen sichtbaren Poll-/Upload-Pfad.

### P1: Hue-Konzept ist doppelt und inkonsistent

Es gibt:

- `OutputType::Hue`
- `ModulePartType::Hue(HueNodeType::...)`

Beide Konzepte sind nicht sauber zusammengefuehrt:

- unterschiedliche Inputs
- keine klare End-to-End-Ausfuehrung
- Inspector-Abdeckung asymmetrisch

### P1: Trigger-System ist doppelt vorhanden

- `module_eval/triggers.rs` ist der aktive Evaluationspfad.
- `trigger_system.rs` definiert ein zweites Trigger-System, das in der App-Laufzeit nicht sichtbar genutzt wird.
- `mapmap-control/src/cue/triggers.rs` definiert zusaetzlich Cue-Trigger fuer MIDI/Time/OSC.

Ergebnis:

- drei Trigger-Konzepte
- keine gemeinsame Registry oder Datenform
- hohes Risiko fuer Drift

### P1: Effekt-/Node-Creation-Flaechen driften

- Quick Create hat einen anderen Katalog als das Kontextmenue.
- Einige Nodes sind im Core und Inspector vorhanden, aber in keiner Create-Flaeche erreichbar.
- Einige Effekte sind ueber Quick Create erzeugbar, im Inspector-Wechselmenue aber nicht mehr auswaehlbar.

## 7. Soll-Zustand

### 7.1 Architektur

- Eine einzige Node-Definition als Source of Truth.
- Daraus werden generiert:
  - Create-Katalog
  - Inspector-Schema
  - Socket-Schema
  - Render-/Eval-Faehigkeit
  - Dokumentation

### 7.2 Connector-Modell

Das System sollte drei klare Connector-Klassen kennen:

- `Media`
  - Texturen, Layer-Streams, Maskenbilder, Geometriequellen
- `Control`
  - kontinuierliche Werte, Vektoren, Farben, Parameter-Busse
- `Event`
  - Impulse, Gates, Trigger-Puls, Note-On, Beat

Wichtig:

- `Event` ist nicht dasselbe wie `Control`.
- Hue-Farbe gehoert nicht auf einen `Media`-Socket, sondern auf `Control(Color)` oder `Control(Vec3)`.
- Trigger-Mappings duerfen nur auf dafuer freigegebenen `Control In`-Sockets erscheinen.

### 7.3 Socket-Identitaet

Statt Indexen braucht das System stabile Socket-IDs:

- z. B. `trigger.beat_out`, `audiofft.rms_out`, `layer.media_in`, `layer.trigger_in`
- Verbindungen referenzieren IDs, nicht Positionsindizes.
- Dynamische Trigger-Outputs bleiben dadurch robust gegen Reorder/Neuberechnung.

### 7.4 Runtime

- Eine echte aktive Render-/Frame-Pipeline oder eine klar benannte Alternative.
- `render_ops` nur fuer Visual-Compositing.
- Source-/Device-Kommandos ueber einen expliziten Runtime-Bus.
- Tote Parallelkonzepte entfernen oder final integrieren.

## 8. Vorschlag fuer ein animiertes, individuelles Nodelayout

### 8.1 Designrichtung

Nicht mehr eine generische Karte pro Node, sondern eine Familie von Node-Skins:

- Trigger Nodes
  - schmale, pulsierende Kacheln mit Live-Meter und Output-Badges
- Source Nodes
  - breite "Preview Cards" mit Thumbnail/Transport-Status
- FX Nodes
  - kompakte Rack-Module mit Parameter-Chips
- Layer Nodes
  - "Stage Cards" mit Mesh-/Preview-Split
- Output Nodes
  - Konsole/Monitor-Look mit Output-Target und Health-Status
- Hue Nodes
  - Lampen-/Zone-Cluster mit Farbflaeche und Trigger-State
- Mesh Nodes
  - Topology-Card mit kleiner Wireframe-Miniatur

### 8.2 Motion

- Signalfluss als subtile Connector-Particles nur auf aktiven Leitungen
- aktiver Trigger-Puls als kurzer Halo ueber dem Port, nicht ueber der ganzen Node
- Preview-Cards mit weichem Crossfade bei Medienwechsel
- Inspector-zu-Node-Fokusanimation: selektierte Node vergroessert Titelband und Port-Labels
- nur eine ruhige Animationsebene gleichzeitig; keine permanente "Christmas Tree"-Bewegung

### 8.3 Layout-Engine

Der aktuelle Auto-Layout-Ansatz sortiert nur nach Kategorien in groben Spalten.

SOLL:

- lane-basiert nach Datenfluss:
  - `Event` links
  - `Source` links-mitte
  - `FX/Mask/Mesh` mitte
  - `Layer` rechts-mitte
  - `Output/Hue` rechts
- Untergruppen pro verbundenem Subgraph
- Abstaende anhand Node-Hoehe und Port-Anzahl
- separate visuelle Lane fuer `Control`-Leitungen oberhalb der Media-Kette

## 9. Verbesserungsvorschlag fuer Trigger Nodes und Triggerverbindungen

### 9.1 Aktuelles Problem

Heute werden unter "Trigger" mehrere Dinge vermischt:

- Event-Quellen (`Beat`, `Fixed`, `Shortcut`)
- kontinuierliche Signalquellen (`AudioFFT RMS`, `BPM`)
- Parameter-Mapping auf beliebige Input-Sockets
- verstecktes Link-System
- getrennte Cue-Trigger im Control-Crate

Das fuehrt zu:

- semantisch unklaren Leitungen
- falschen Socket-Typen
- irrefuehrendem Inspector-Verhalten
- schlechter Erweiterbarkeit fuer Automation/Timeline

### 9.2 Vorschlag

Neues Modell:

- `Event Nodes`
  - Beat
  - Fixed Timer
  - Random Pulse
  - MIDI Note
  - OSC Event
  - Shortcut
- `Control Nodes`
  - FFT Bands
  - RMS/Peak
  - BPM
  - Smooth
  - Clamp
  - Scale
  - LFO
  - Envelope
  - Random Range
  - Gate-to-Hold
- `Routing Nodes`
  - Split
  - Merge
  - Remap
  - Color Pack/Unpack

Jede Node, die Automation erlaubt, definiert explizit:

- `Control In`-Sockets
- erwarteten Datentyp
- Range
- Default
- moegliche Modulationskurve

### 9.3 Trigger-Connector-Regeln

- `Event -> Event` erlaubt
- `Event -> Control` nur ueber explizite Konverter (`GateToValue`, `PulseToLatch`)
- `Control -> Control` erlaubt bei kompatiblem Datentyp
- `Media -> Media` erlaubt
- alles andere blockieren

### 9.4 Cue-Integration

`cue/triggers.rs` sollte nicht als separates Trigger-Universum neben dem Node-System stehen.

SOLL:

- gemeinsame Trigger-Registry
- gemeinsame Event-Normalform
- Cues koennen dieselben `Event`-Quellen abonnieren wie Node-Graphen

## 10. Priorisierte To-Dos

### P0

- `ModuleManager::get_module_mut()` von Dirty-Markierung entkoppeln oder gesonderte "edit sessions" einfuehren.
- Verbindungsvalidierung beim Drop hart durchsetzen.
- Sockets nur noch zentral ableiten oder nach jedem Struktur-Edit konsequent neu berechnen.
- Node-Erstellung ueber echte Factory-Funktionen mit eindeutigen IDs und Namen fuehren.
- Masken, Blend-Modi und Source-Transform entweder rendern oder im UI ausblenden.
- `LayerType::All` entweder implementieren oder aus dem Canvas entfernen.
- `SourceCommand`/Media-/Device-Runtime vereinheitlichen; tote Pfade entfernen.
- `FramePipeline`-Doku und echte App-Laufzeit wieder angleichen.

### P1

- Inspector fuer `HueNodeType::*` implementieren.
- `Bevy3DModel`-Inspector implementieren.
- fehlende TriggerTargets im Inspector nachziehen.
- Projector-Advanced-Settings (`output_width`, `output_height`, `output_fps`) in den Inspector aufnehmen.
- Quick Create, Kontextmenue und Katalog auf dieselbe Definitionsquelle umstellen.
- NDI/Spout/LiveInput/Shader Nodes nur sichtbar machen, wenn auch Runtime vorhanden ist.

### P2

- per-Node-Skin-System fuer das Canvas.
- lane-basiertes Auto-Layout mit `Media`/`Control`/`Event`.
- gemeinsame Trigger-/Cue-Registry.
- stabiler Socket-ID-basierter Graph statt Index-Verbindungen.

## 11. Empfehlung

Die sinnvollste naechste Etappe ist kein weiterer Feature-Zuwachs, sondern eine Konsolidierungsphase:

1. Schema fuer Nodes, Sockets und Inspector vereinheitlichen.
2. tote bzw. halbverdrahtete Node-Typen temporaer verstecken.
3. Renderpfad auf die im UI sichtbaren Features begrenzen und diese dann sauber zu Ende implementieren.
4. danach erst das neue Trigger-/Control-Bus-Konzept und das neue Node-Design ausrollen.

So wird aus dem heutigen "breiten, aber inkonsistenten" Node-System ein kleines, aber belastbares System, das sich spaeter wieder kontrolliert vergroessern laesst.

## 12. Umsetzungsstand 2026-03-19

Folgende Basis wurde inzwischen im Code umgesetzt:

- `ModuleSocket` traegt jetzt stabile Schema-Metadaten:
  - `id`
  - `direction`
  - `supports_trigger_mapping`
  - `is_primary`
  - `accepts_multiple_connections`
- `ModulePart::schema()` liefert ein konsolidiertes Node-/Socket-/Inspector-Schema.
- `MapFlowModule` validiert neue Verbindungen ueber `validate_connection(...)` und `connect_parts(...)`.
- `repair_graph()` entfernt inkonsistente Connections und ungueltige Trigger-Mappings und normalisiert doppelte IDs.
- `ModuleManager::get_module_mut()` dirty-markiert nicht mehr implizit.
- `ModuleManager::repair_modules(...)` wird im App-Loop vor der Evaluation fuer Self-Healing genutzt.
- Der Canvas erzwingt beim Verbinden jetzt die Core-Validierung statt nur einer losen Farbvorschau.
- Presets erzeugen Sockets nicht mehr ueber eine driftende UI-Kopie, sondern ueber das Core-Modell.
- Die App verwendet jetzt eine explizite `RuntimeRenderQueue` statt eines lose semantischen `Vec<(ModuleId, RenderOp)>`.
- Der eingebettete Bevy-Runner deaktiviert `bevy::winit::WinitPlugin`, damit kein zweiter Event-Loop erzeugt wird.
- Der Windows-Runtime-DLL-Copy-Pfad validiert jetzt FFmpeg-DLLs und bevorzugt den gueltigen Bestand unter `vcpkg_installed/x64-windows/bin`.
- `frame_counter` wird wieder im primaeren Renderpfad erhoeht; dadurch funktioniert `--exit-after-frames` im Automation-Modus wieder.
- Das Bevy-Frame-Readback entmappt den GPU-Buffer jetzt deterministisch innerhalb desselben Frames.
- Die `composite`-Textur wird fuer den Automation-Capture-Pfad jetzt mit `COPY_SRC` angelegt; dadurch verursacht `--screenshot-dir` keine WGPU-Validierungspanik mehr.
- Debug-Smoke-Tests am 2026-03-19:
  - `target/debug/MapFlow.exe --help` -> `EXIT=0`
  - `target/debug/MapFlow.exe --mode automation --exit-after-frames 1` -> `EXIT=0`
  - `target/debug/MapFlow.exe --mode automation --exit-after-frames 1 --screenshot-dir <dir>` -> `EXIT=0`
  - `tmp/automation-check-debug/automation_frame_1.png` wird erfolgreich geschrieben

Noch offen:

- Output-Inspector, Masken, Blend-Modi und Render-Transforms sind noch nicht voll end-to-end geschlossen
- ein finaler Release-Smoke-Test steht noch aus, obwohl die eigentlichen Startblocker im Debug-Build behoben und verifiziert sind

### 12.1 Inspector Previews (Feature Update)
Ein konsistenter Preview-Contract wurde fuer alle Node-Familien umgesetzt:
- **Layer**: Hat weiterhin den erweiterten Preview-Panel (Direct, Linked Output, Fallback).
- **Source, Mask, Modulizer (Effekte)**: Zeigen nun alle standardmaessig einen Live-Texture-Preview im Inspector an.
- **Output**: Zeigt nun den Live-Texture-Preview des finalen Outputs basierend auf der Output ID an.
- **Mesh**: Preview ist im Node-Inspector bewusst nicht verfuegbar, dafuer gibt es einen Hinweis, den darunterliegenden Mesh-Editor zu verwenden.
- **Hue**: Live-Visual-Preview ist fuer Hardware-Outputs bewusst nicht verfuegbar. Es gibt einen UI-Hinweis auf den Spatial-Editor oder die physischen Lampen.
- **Trigger**: Alle Trigger-Typen (nicht nur `Fixed`, sondern auch `Beat`, `AudioFFT`, `Random`, etc.) zeigen nun einen Live-Trigger-Status (`ProgressBar` & Live Pulse Info) basierend auf `last_trigger_values` an.
