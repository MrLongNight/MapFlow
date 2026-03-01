# Übersicht der Benutzeroberfläche

Die Benutzeroberfläche (UI) von MapFlow ist modular und flexibel gestaltet. Sie können Panels so anordnen, wie es Ihrem Workflow am besten entspricht, egal ob Sie auf einem einzelnen Bildschirm arbeiten oder ein komplexes Multi-Projektor-Setup verwalten.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das vollständige MapFlow-Standard-Layout, mit Nummern, die auf jedes Hauptpanel hinweisen.]**

Das Interface von MapFlow basiert auf einem Docking-System. Sie können Panel-Tabs ziehen, um sie neu anzuordnen, oder das **View**-Menü (Ansicht) verwenden, um bestimmte Panels ein- oder auszublenden.

Das Standardlayout umfasst typischerweise:
- **Oben Mitte:** Dashboard
- **Linke Seitenleiste:** Media Browser
- **Mitte:** Module Canvas
- **Rechte Seitenleiste:** Inspector
- **Unten:** Timeline
- **Standardmäßig ausgeblendet (Erreichbar über das View-Menü oder spezielle Werkzeuge):** Mapping Panel, Output Panel, Audio Panel

Lassen Sie uns jedes dieser Panels im Detail betrachten.

---

## 1. Dashboard (Top Center)

Das Dashboard ist Ihre Hauptsteuerzentrale während einer Performance. Es bietet übergeordnete Kontrolle und Überwachung.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Dashboard-Panel, Hervorhebung der Wiedergabesteuerungen, Leistungsstatistiken und globalen Schieberegler.]**

*   **Wiedergabesteuerungen (Playback Controls):** Play, Pause, Stop und Geschwindigkeitskontrolle für den globalen Transport (Timeline).
*   **Leistungsstatistiken (Performance Stats):** Echtzeit-Überwachung wichtiger Metriken: FPS (Frames pro Sekunde), Frame Time, CPU-Auslastung und GPU-Auslastung. Behalten Sie diese während komplexer Setups im Auge.
*   **Master Controls:** Globale Schieberegler für Deckkraft (Opacity / Fade to Black) und Geschwindigkeit, die die gesamte Komposition beeinflussen.
*   **Toolbar (Werkzeugleiste):** Schneller Zugriff auf häufig verwendete Werkzeuge und Modi, wie das Umschalten zwischen \"Edit\" (Aufbau des Graphen), \"Map\" (Verzerrung von Ausgaben) und \"Perform\" (Sperren der Benutzeroberfläche für den Live-Einsatz).

---

## 2. Module Canvas (Center)

Der Module Canvas ist das Herzstück des Node-basierten Workflows von MapFlow. Hier verbinden Sie verschiedene Module, um Ihren visuellen Signalfluss zu erstellen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Der Module Canvas, der einige verbundene Nodes mit Datenfluss durch Kabel zeigt.]**

*   **Nodes (Module):** Repräsentieren Funktionseinheiten (Media Player, Effekte, Layer, Ausgaben).
*   **Verbindungen (Wires / Kabel):** Kabel, die Ausgänge (rechte Sockets) eines Nodes mit Eingängen (linke Sockets) eines anderen verbinden.
*   **Interaktion:**
    *   **Rechtsklick:** Öffnet das Menü \"Node hinzufügen\" (Add Node).
    *   **Kabel ziehen:** Erstellt eine Verbindung, indem Sie von einem Ausgang zu einem Eingang klicken und ziehen.
    *   **Node anklicken:** Wählt ihn aus, um Eigenschaften im Inspector anzuzeigen.
    *   **Tab (Quick Create):** Öffnet ein durchsuchbares Popup-Fenster, um schnell Nodes hinzuzufügen.
*   **Navigation:** Scrollen, um hinein-/herauszuzoomen; klicken und ziehen auf den Hintergrund des Canvas, um sich zu bewegen (Pan).

---

## 3. Media Browser (Left Sidebar)

Der Media Browser ermöglicht es Ihnen, Inhalte von Ihrem lokalen Dateisystem zu durchsuchen, in der Vorschau anzuzeigen und zu importieren.

> 🖼️ **[PLATZHALTER SCREENSHOT: Der Media Browser, der eine Ordnerhierarchie und Video-Thumbnails zeigt.]**

*   **Navigation:** Durchsuchen von Ordnern und Laufwerken auf Ihrem Computer.
*   **Vorschau (Preview):** Bewegen Sie den Mauszeiger über unterstützte Dateien, um eine Vorschau anzuzeigen (falls verfügbar).
*   **Importieren:** Ziehen Sie Dateien direkt aus dem Browser auf den Module Canvas, um sofort einen Media-Node zu erstellen.
*   **Unterstützte Formate:** MapFlow unterstützt eine Vielzahl von Video- (H.264, H.265, VP8/9, HAP), Bild- (PNG, JPG, BMP, GIF) und Bildsequenzformaten.

---

## 4. Inspector (Right Sidebar)

Der Inspector ist ein kontextsensitives Panel, das die Eigenschaften des aktuell ausgewählten Objekts anzeigt. Hier nehmen Sie feine Anpassungen vor.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Inspector-Panel, das die Eigenschaften eines ausgewählten Layer-Nodes (Transform, Opacity, Blend Mode) anzeigt.]**

*   **Kontextsensitiv:** Der Inhalt ändert sich je nachdem, was im Canvas, in der Timeline oder im Mapping Panel ausgewählt ist.
*   **Node-Eigenschaften:** Wenn ein Node (z.B. ein Blur-Effekt) ausgewählt ist, erscheinen seine spezifischen Parameter (z.B. Blur Radius) hier.
*   **Layer-Eigenschaften:** Wenn ein Layer-Node ausgewählt ist, sehen Sie Transform-Steuerungen (Position, Scale, Rotation), Blend-Modi (Add, Multiply, Screen) und Deckkraft (Opacity).
*   **Mapping-Eigenschaften:** Beim Bearbeiten eines Meshes im Mapping Panel zeigt der Inspector Vertex-Koordinaten und Warping-Werkzeuge an.
*   **Parameter-Automatisierung:** Viele Parameter im Inspector können mit der rechten Maustaste angeklickt werden, um Audio- oder MIDI-Modulationen zuzuweisen.

---

## 5. Timeline (Bottom)

Die Timeline wird zum Sequenzieren, Automatisieren und Auslösen von Ereignissen im Laufe der Zeit verwendet.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Timeline-Panel, das ein Multi-Track-Layout mit Blöcken zeigt, die die Node-Wiedergabe repräsentieren.]**

*   **Tracks (Spuren):** Jeder Layer oder Parameter kann eine eigene horizontale Spur haben.\n*   **Blöcke/Clips:** Visuelle Darstellungen, wann ein Node oder Effekt aktiv ist.
*   **Keyframes:** Setzen Sie Werte an bestimmten Zeitpunkten, um Parameter zu animieren.
*   **Transport:** Der Abspielkopf (Playhead) scrubbt durch die Zeit. Sie können die Dashboard-Steuerungen oder die Leertaste verwenden, um die Wiedergabe zu steuern.
*   **Modi:** Wechseln Sie zwischen verschiedenen Wiedergabemodi (z.B. Loop, One-Shot, Ping-Pong).

---

## 6. Mapping Panel (View Menu)

Das Mapping Panel ist eine dedizierte Umgebung für Projection-Mapping-Aufgaben, erreichbar, wenn Sie eine Ausgabe verzerren oder maskieren müssen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Mapping Panel aktiv, das ein Mesh-Gitter über einer Ausgabenvorschau zeigt.]**

*   **Mesh Editor:** Auswählen und Manipulieren von Kontrollpunkten auf einem Warping-Mesh.
*   **Keystone:** Einfache 4-Punkt-Perspektivenkorrektur.
*   **Grid Warp:** Erweitertes Multi-Punkt-Bezier-Warping für gekrümmte oder unregelmäßige Oberflächen.
*   **Masking:** Zeichnen von benutzerdefinierten Masken mit Bezier-Kurven, um unerwünschte Teile der Projektion auszublenden.

---

## 7. Output Panel (View Menu)

Das Output Panel verwaltet das endgültige Ziel Ihrer Visuals.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Output Panel, das mehrere physische Displays und Routing-Optionen zeigt.]**

*   **Displays:** Weisen Sie MapFlow-Output-Nodes physischen Monitoren oder Projektoren zu, die an Ihren Computer angeschlossen ist.
*   **Virtuelle Ausgaben:** Konfigurieren Sie Ausgaben an NDI-Streams oder Spout/Syphon, um Videos an andere Anwendungen zu senden.
*   **Auflösung/Bildwiederholrate:** Stellen Sie benutzerdefinierte Auflösungen und Bildwiederholraten für jede Ausgabe ein.
*   **Edge Blending:** Konfigurieren Sie Überlappungszonen für nahtlose Multi-Projektor-Anordnungen.
*   **Testmuster:** Generieren Sie Gitter und Farbbalken zur Projektorausrichtung.

---

## 8. Audio Panel (View Menu)

Im Audio Panel konfigurieren Sie die Toneingabe und die Audio-Reaktivitätseinstellungen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Audio Panel, das die Eingabeauswahl, Gain-Regler und den FFT-Spektrum-Analysator zeigt.]**

*   **Eingabekonfiguration:** Wählen Sie Ihr Audio-Interface, den Eingangskanal und die Abtastrate aus.
*   **Gain-Regler:** Passen Sie den eingehenden Audiopegel an, um starke Signale ohne Clipping (Übersteuern) sicherzustellen.
*   **Spektrum-Analysator:** Eine visuelle Echtzeitdarstellung (FFT) der Audiofrequenzen.
*   **Audio-Triggers:** Konfigurieren Sie Schwellenwerte für Frequenzbänder (Bass, Mid, High), um Parameter in Ihrem Node-Graphen zu steuern.

---

## 9. Controller Overlay (Hidden/View Menu)

Eine visuelle Referenz für Ihre angeschlossenen MIDI- oder OSC-Controller.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Controller Overlay, das eine virtuelle Darstellung eines MIDI-Controllers mit markierten, gemappten Parametern zeigt.]**

*   **Visualisierung:** Zeigt den Status von Drehreglern, Fadern und Tasten auf Ihrer physischen Hardware.
*   **MIDI Learn Modus:** Eine visuelle Schnittstelle, um On-Screen-UI-Elemente schnell physischen Hardware-Steuerungen zuzuweisen.
*   **Feedback:** Markiert aktive Steuerungen und zeigt aktuelle Werte.
