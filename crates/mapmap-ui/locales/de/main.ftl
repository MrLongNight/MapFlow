#
# MapFlow - Deutsche Übersetzungen
#

#
# Allgemein / App-weit
#
app-title = MapFlow
project-filter-name = MapFlow Projekt
new-project-name = Neues Projekt
paint-new-color-name = Neue Farbe
new-output-name = Neue Ausgabe

#
# Hauptmenü & Werkzeugleiste (egui)
#
menu-file = Datei
menu-file-new-project = Neues Projekt...
menu-file-open-project = Projekt öffnen...
menu-file-open-recent = Zuletzt geöffnet
menu-file-save-project = Projekt speichern
menu-file-save-as = Speichern unter...
menu-file-export = Exportieren...
menu-file-settings = Einstellungen...
menu-file-exit = Beenden

menu-edit = Bearbeiten
menu-edit-undo = Rückgängig
menu-edit-redo = Wiederherstellen
menu-edit-cut = Ausschneiden
menu-edit-copy = Kopieren
menu-edit-paste = Einfügen
menu-edit-delete = Löschen
menu-edit-select-all = Alles auswählen

menu-view = Ansicht
view-egui-panels = Egui-Bedienfelder
view-legacy-panels = Legacy-Bedienfelder
view-reset-layout = Layout zurücksetzen
btn-fullscreen = Vollbild umschalten

menu-help = Hilfe
menu-help-docs = Dokumentation
menu-help-about = Über
menu-help-license = Lizenz
menu-language = Sprache
language-en = Englisch
language-de = Deutsch

toolbar-save = Speichern
toolbar-undo = Rückgängig
toolbar-redo = Wiederherstellen

#
# Ansichtsmenü-Checkboxen
#
check-show-osc = OSC anzeigen
check-show-controls = Wiedergabe anzeigen
check-show-layers = Ebenen anzeigen
check-show-paints = Farben anzeigen
check-show-mappings = Mappings anzeigen
check-show-transforms = Transformationen anzeigen
check-show-master = Master anzeigen
check-show-oscillator = Oszillator anzeigen
check-show-audio = Audio anzeigen
check-show-cues = Cues anzeigen
check-show-stats = Leistung anzeigen
panel-dashboard = Dashboard
panel-media-browser = Medienbrowser
panel-asset-manager = Asset-Manager
panel-mesh-editor = Mesh-Editor
panel-node-editor = Node-Editor
panel-timeline = Zeitleiste
panel-effect-chain = Effekt-Kette

#
# Dashboard
#
panel-dashboard-title = Dashboard
dashboard-layout-grid = Raster
dashboard-layout-freeform = Freiform
dashboard-columns = Spalten
dashboard-add-widget = ➕ Widget hinzufügen
dashboard-add-widget-tooltip = Ein neues Widget zum Dashboard hinzufügen
dashboard-state = Status
dashboard-speed = Geschwindigkeit
dashboard-loop = Schleife
dashboard-audio-analysis = Audio-Analyse
dashboard-device = Gerät
dashboard-no-device = Kein Gerät ausgewählt
dashboard-volume = Lautstärke
dashboard-rms = RMS
dashboard-peak = Spitzenwert
dashboard-spectrum = Frequenzspektrum
dashboard-no-audio-data = Keine Audio-Analysedaten verfügbar.
dashboard-remove-widget = Widget entfernen
dashboard-trigger = Auslösen
btn-play-icon = ▶
btn-pause-icon = ⏸
btn-stop-icon = ⏹

#
# Legacy Wiedergabe-Steuerung
#
panel-playback = Wiedergabe-Steuerung
header-video-playback = Video-Wiedergabe
btn-play = Abspielen
btn-pause = Pause
btn-stop = Stopp
label-speed = Geschwindigkeit
label-mode = Modus
mode-loop = Schleife
mode-play-once = Einmal abspielen

#
# Leistungs-Panel
#
panel-performance = Leistung
label-fps = FPS
label-frame-time = Frame-Zeit

#
# Ebenen-Panel
#
panel-layers = Ebenen
label-total-layers = Ebenen insgesamt: { $count }
check-bypass = Umgehen (B)
check-solo = Solo (S)
label-master-opacity = Deckkraft (V)
btn-duplicate = Duplizieren
btn-remove = Entfernen
btn-add-layer = Ebene hinzufügen
btn-eject-all = Alle auswerfen (X)

#
# Farben-Panel (Paints)
#
panel-paints = Farben & Medien
label-total-paints = Farben insgesamt: { $count }
check-playing = Spielt ab
paints-color = Farbe
paint-label-name-type = { $name } ({ $type })
btn-add-paint = Farbe hinzufügen
btn-remove-paint = Entfernen

#
# Mappings-Panel
#
panel-mappings = Mappings
label-total-mappings = Mappings insgesamt: { $count }
check-lock = Sperren
label-depth = Tiefe
label-mesh = Mesh: { $type } ({ $count } Vertices)
btn-add-quad = Quad hinzufügen
btn-remove-this = Entfernen

#
# Transformations-Panel
#
panel-transforms = Transformation
header-transform-sys = Transformations-System
label-editing = Bearbeitet
transform-position = Position
drag-val-x = X:
drag-val-y = Y:
transform-rotation = Rotation (Grad)
slider-z = Z
btn-reset-rotation = Rotation zurücksetzen
transform-scale = Skalierung
drag-val-w = B:
drag-val-h = H:
btn-reset-scale = Skalierung zurücksetzen (1:1)
label-anchor = Ankerpunkt (0-1)
btn-center-anchor = Anker zentrieren (0.5, 0.5)
transform-presets = Größen-Voreinstellungen
transform-fill = Füllen (Cover)
btn-resize-fit = Einpassen (Contain)
btn-resize-stretch = Strecken (Distort)
btn-resize-original = Original (1:1)
transform-no-layer = Keine Ebene ausgewählt.
transform-select-tip = Klicke auf eine Ebene im Ebenen-Panel, um sie auszuwählen.

#
# Master-Steuerungs-Panel
#
panel-master = Master-Steuerung
header-master = Master-Steuerung
label-composition = Komposition
label-master-speed = Master-Geschwindigkeit (S)
label-size = Größe
label-frame-rate = Bildrate
label-effective-multipliers = Effektive Multiplikatoren:
text-mult-opacity = Ebenen-Deckkraft × Master-Deckkraft
text-mult-speed = Wiedergabegeschw. × Master-Geschw.

#
# Ausgabe-Panel
#
panel-outputs = Ausgaben
header-outputs = Ausgaben
header-selected-output = Ausgewählte Ausgabe
label-canvas = Leinwand
btn-projector-array = 2x2 Projektor-Array
btn-add-output = Ausgabe hinzufügen
label-name = Name
label-resolution = Auflösung
label-canvas-region = Leinwand-Bereich
label-x = X
label-y = Y
label-width = Breite
label-height = Höhe
label-none = Keine
output-tip = Tipp
tip-panels-auto-open = Kantenüberblendungs- und Farbkalibrierungs-Bedienfelder öffnen sich automatisch, wenn eine Ausgabe ausgewählt wird.
btn-remove-output = Ausgabe entfernen
msg-multi-window-active = Multi-Fenster-Rendering ist AKTIV.
msg-output-windows-tip = Ausgabefenster werden automatisch erstellt und synchronisiert.

#
# Kantenüberblendungs-Panel
#
panel-edge-blend = Kantenüberblendung
label-output = Ausgabe: { $name }
check-left = Linke Kante
check-right = Rechte Kante
check-top = Obere Kante
check-bottom = Untere Kante
label-offset = Versatz
label-gamma = Blend-Gamma
btn-reset-defaults = Auf Standard zurücksetzen

#
# Farbkalibrierungs-Panel
#
panel-color-cal = Farbkalibrierung
label-brightness = Helligkeit
label-contrast = Kontrast
label-saturation = Sättigung
label-gamma-channels = Gamma (pro Kanal)
label-gamma-red = Rot-Gamma
label-gamma-green = Grün-Gamma
label-gamma-blue = Blau-Gamma
label-color-temp = Farbtemperatur

#
# Oszillator-Panel
#
panel-oscillator = Oszillator-Verzerrung
check-enable = Effekt aktivieren
header-quick-presets = Schnell-Voreinstellungen
btn-subtle = Dezent
btn-dramatic = Dramatisch
btn-rings = Ringe
btn-reset = Zurücksetzen
header-distortion = Verzerrungsparameter
label-amount = Stärke
label-dist-scale = Skalierung
label-dist-speed = Geschwindigkeit
header-visual-overlay = Visuelle Überlagerung
label-overlay-opacity = Deckkraft der Überlagerung
label-color-mode = Farbmodus
color-mode-off = Aus
color-mode-rainbow = Regenbogen
color-mode-black-white = Schwarz & Weiß
color-mode-complementary = Komplementär
header-simulation = Simulationsparameter
sim-res-low = Niedrig (128x128)
sim-res-medium = Mittel (256x256)
sim-res-high = Hoch (512x512)
label-kernel-radius = Kernel-Radius
label-noise-amount = Rauschanteil
label-freq-min = Frequenz Min (Hz)
label-freq-max = Frequenz Max (Hz)
label-coordinate-mode = Koordinatenmodus
coord-mode-cartesian = Kartesisch
coord-mode-log-polar = Log-Polar
label-phase-init = Phaseninitialisierung
phase-init-random = Zufällig
phase-init-uniform = Gleichmäßig
phase-init-plane-h = Ebene H
phase-init-plane-v = Ebene V
phase-init-diagonal = Diagonal
header-coupling = Kopplungsringe (Erweitert)
header-coupling-ring = Ring { $id }
label-diff-coupling = Kopplung
btn-reset-ring = Ring zurücksetzen
btn-clear-ring = Ring leeren
tooltip-intensity = Intensität des Verzerrungseffekts
tooltip-dist-scale = Räumliche Skalierung der Verzerrung
tooltip-dist-speed = Animationsgeschwindigkeit
tooltip-sim-res = Höhere Auflösung = mehr Details, aber langsamer
tooltip-kernel-radius = Interaktionsdistanz der Kopplung
tooltip-noise-amount = Zufällige Variation der Oszillation
tooltip-coord-mode = Log-Polar erzeugt radiale/spiralförmige Muster
tooltip-phase-init = Initiales Phasenmuster für Oszillatoren
tooltip-ring-dist = Abstand vom Zentrum (0-1)
tooltip-ring-width = Ringbreite (0-1)
tooltip-ring-coupling = Negativ = Anti-Sync, Positiv = Sync

#
# Audio-Panel
#
audio-panel-title = Audio-Analyse
audio-panel-device = Eingabegerät
audio-panel-no-device = Kein Gerät
audio-panel-no-data = Warte auf Audiodaten...
audio-panel-rms = RMS-Lautstärke
audio-panel-beat = Beat
audio-panel-bands = Frequenzbänder

#
# OSC-Panel
#
panel-osc-title = OSC-Steuerung
header-osc-server = OSC-Server
label-status = Status
status-running = Läuft
status-stopped = Gestoppt
label-port = Port
btn-start-server = Server starten
header-feedback-clients = Feedback-Clients
label-add-client = Client hinzufügen
btn-add = Hinzufügen
header-address-mappings = Adress-Zuweisungen
text-osc-edit-tip = (Zur Konfiguration osc_mappings.json bearbeiten)

#
# Cue-Panel
#
panel-cue = Cues

#
# Effekt-Ketten-Panel
#

blend-mode-normal = Normal
blend-mode-add = Addieren
blend-mode-subtract = Subtrahieren
blend-mode-multiply = Multiplizieren
blend-mode-screen = Negativ Multiplizieren
blend-mode-overlay = Überlagern
blend-mode-soft-light = Weiches Licht
blend-mode-hard-light = Hartes Licht
blend-mode-lighten = Aufhellen
blend-mode-darken = Abdunkeln
blend-mode-color-dodge = Farbig Abwedeln
blend-mode-color-burn = Farbig Nachbelichten
blend-mode-difference = Differenz
blend-mode-exclusion = Ausschluss

effect-name-color-adjust = Farbanpassung
effect-name-blur = Weichzeichner
effect-name-chromatic-aberration = Chromatische Aberration
effect-name-edge-detect = Kantenerkennung
effect-name-glow = Leuchten
effect-name-kaleidoscope = Kaleidoskop
effect-name-invert = Invertieren
effect-name-pixelate = Verpixeln
effect-name-vignette = Vignette
effect-name-film-grain = Filmkörnung
effect-name-custom = Benutzerdefiniert
effect-add = Hinzufügen
effect-presets = Voreinstellungen
effect-clear = Leeren
effect-select-type = Effekttyp auswählen
effect-no-effects = Keine Effekte in der Kette
effect-start-tip = Fügen Sie einen Effekt hinzu, um zu beginnen
effect-intensity = Intensität
param-brightness = Helligkeit
param-contrast = Kontrast
param-saturation = Sättigung
param-radius = Radius
param-threshold = Schwellenwert
param-segments = Segmente
param-rotation = Rotation
param-pixel-size = Pixelgröße
param-softness = Weichheit
no-parameters = Keine Parameter
effect-save = Speichern
effect-presets-browser = Voreinstellungs-Browser
effect-search = Suchen...

#
# Generische Labels & Buttons
#
label-none = Keine
label-name = Name
label-mode = Modus
label-speed = Geschwindigkeit
label-width = Breite
label-height = Höhe
label-master-opacity = Deckkraft
btn-remove = Entfernen
btn-reset-defaults = Auf Standard zurücksetzen

#
# Logging-Meldungen
#
error-list-audio-devices = Auflisten der Audiogeräte fehlgeschlagen: { $error }
error-init-audio-backend = Initialisierung des Audio-Backends fehlgeschlagen: { $error }
error-start-audio-stream = Starten des Audio-Streams fehlgeschlagen: { $error }
error-mcp-server = MCP-Serverfehler: { $error }
error-handle-event = Fehler bei der Ereignisbehandlung: { $error }
error-render = Renderfehler bei Ausgabe { $output_id }: { $error }
error-autosave = Automatisches Speichern fehlgeschlagen: { $error }
info-autosave = Automatisches Speichern erfolgreich
error-save-project = Speichern des Projekts fehlgeschlagen: { $error }
info-save-project = Projekt gespeichert unter { $path }
error-load-project = Laden des Projekts fehlgeschlagen: { $error }
info-load-project = Projekt geladen von { $path }
info-language-switch = Sprache umgeschaltet auf: { $lang_code }
mcp-info-save = MCP: Speichere Projekt unter { $path }
mcp-error-save = MCP: Speichern des Projekts fehlgeschlagen: { $error }
mcp-info-load = MCP: Lade Projekt von { $path }
mcp-info-add-layer = MCP: Füge Ebene '{ $name }' hinzu
mcp-info-remove-layer = MCP: Entferne Ebene { $id }
mcp-info-trigger-cue = MCP: Löse Cue { $id } aus
mcp-info-next-cue = MCP: Nächster Cue
mcp-info-prev-cue = MCP: Vorheriger Cue
mcp-info-unimplemented = MCP: Nicht implementierte Aktion empfangen: { $action }
label-render-encoder = Render-Encoder
label-egui-render-pass = Egui-Render-Pass
label-output-render-pass = Ausgabe-Render-Pass
label-imgui-render-pass = ImGui-Render-Pass
info-app-start = Starte MapFlow...
