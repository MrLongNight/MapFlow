# Node Referenz

Dieses Dokument bietet eine umfassende Liste aller im Module Canvas von MapFlow verfügbaren Nodes, kategorisiert nach ihrer Funktion.

> 🚧 **[WIP: Dieses Referenzhandbuch befindet sich derzeit im Aufbau. Weitere Nodes und detaillierte Parameterbeschreibungen werden hinzugefügt, sobald MapFlow sich seinem offiziellen Release nähert.]**

---

## 1. Quellen (Sources / Generators & Media)

Source-Nodes sind der Ausgangspunkt für jeden visuellen Signalfluss. Sie erstellen oder importieren Inhalte in den Node-Graphen.

*   **Media Player:** Spielt Videodateien (MP4, MOV, MKV, HAP) und Bildsequenzen ab. Unterstützt Looping (Schleifenwiedergabe) und Geschwindigkeitssteuerung.
*   **Image:** Lädt eine statische Bilddatei (PNG, JPG).
*   **Solid Color (Einfarbig):** Erzeugt einen soliden, gleichmäßigen Farbblock. Nützlich für Hintergründe oder Masken.
*   **Gradient (Farbverlauf):** Erstellt einen fließenden Übergang zwischen zwei oder mehr Farben (Linear, Radial).
*   **Text:** Rendert benutzerdefinierten Text mit anpassbarer Schriftart, Größe und Ausrichtung.
*   **Live Input (Camera/Capture):** Erfasst Videos von Webcams, Capture-Karten oder virtuellen Eingängen wie NDI/Spout.
*   **Shader (Generative):** Führt benutzerdefinierte GLSL-Shader aus, um algorithmische Visuals (Fraktale, Rauschen, Muster) in Echtzeit zu generieren.

---

## 2. Effekte (FX)

Effekt-Nodes modifizieren das eingehende visuelle Signal. Sie können aneinandergereiht werden, um komplexe Looks zu erstellen.

*   **Blur (Unschärfe):** Wendet eine standardmäßige Gaußsche Unschärfe an, um das Bild weicher zu machen.
*   **Color Correction (Farbkorrektur):** Passt Farbton (Hue), Sättigung (Saturation), Helligkeit (Brightness) und Kontrast (Contrast) an.
*   **Levels (Tonwertkorrektur):** Feinabstimmung des Schwarzpunkts, Weißpunkts und der Mitteltöne (Gamma) für präzise Kontrastkontrolle.
*   **Invert:** Invertiert die Farben des Bildes (Negativ-Effekt).
*   **Pixelate:** Reduziert die Auflösung des Bildes und erzeugt so eine blockige, Retro-Ästhetik.\n*   **Distortion (Warp/Displace):** Verzerrt das Bild basierend auf einem Rauschmuster oder einem anderen Videoeingang (Displacement Mapping).
*   **Edge Detect (Kantenerkennung):** Hebt die Kanten innerhalb des Bildes hervor und erzeugt einen stilisierten, umrissenen Look.
*   **LUT (Look-Up Table):** Wendet komplexes, kinoreifes Color Grading unter Verwendung von branchenüblichen `.cube`-Dateien an.

---

## 3. Layer (Compositing)

Layer-Nodes werden verwendet, um mehrere visuelle Signale zu kombinieren, ihre Position im Raum zu verwalten und zu steuern, wie sie miteinander verschmelzen.

*   **Standard Layer:** Der grundlegende Compositing-Node. Nimmt einen Eingang entgegen, ermöglicht das Transformieren (Position, Skalierung, Rotation), Festlegen der Deckkraft (Opacity) und Auswählen eines Blend-Modus (Add, Multiply, Screen, Overlay etc.), bevor das Ergebnis ausgegeben wird.
*   **Mix (Crossfade):** Nimmt zwei Eingänge (A und B) entgegen und bietet einen Schieberegler für einen stufenlosen Übergang zwischen ihnen.
*   **Mask Layer:** Nimmt einen visuellen Eingang und einen Maskeneingang (z. B. eine Schwarz-Weiß-Form) entgegen. Die Maske bestimmt, welche Teile des visuellen Eingangs sichtbar sind.
*   **Group (Gruppe):** Ein spezieller Node, der als Container für andere Nodes fungiert und es Ihnen ermöglicht, komplexe Abschnitte Ihres Graphen in einem einzigen, übersichtlichen Block zu organisieren.

---

## 4. Ausgaben (Outputs)

Output-Nodes sind das Endziel im Node-Graphen und senden die zusammengesetzten Visuals aus MapFlow heraus.

*   **Projector (Window):** Sendet das visuelle Signal an ein physisches Display oder einen Projektor, der an Ihren Computer angeschlossen ist.
*   **Virtual Output (NDI):** Überträgt das visuelle Signal über Ihr lokales Netzwerk unter Verwendung des NDI-Protokolls, sodass andere Computer oder Software es empfangen können.
*   **Virtual Output (Spout/Syphon):** Teilt das visuelle Signal direkt mit anderen Anwendungen, die auf demselben Computer ausgeführt werden (Spout für Windows, Syphon für macOS), ohne Verzögerung (Zero Latency).
*   **Record:** Erfasst die endgültige visuelle Ausgabe und speichert sie als Videodatei auf Ihrer Festplatte.\n\n---

## 5. Steuerung & Logik (Control & Logic)

Diese Nodes verarbeiten kein Video, sondern generieren oder modifizieren Steuersignale (Zahlen), die zur Automatisierung anderer Parameter verwendet werden.

*   **Oscillator (LFO):** Generiert sich wiederholende mathematische Wellenformen (Sinus, Rechteck, Dreieck, Sägezahn), die mit Parametern verknüpft werden können (z. B. um einen Layer kontinuierlich pulsieren zu lassen).
*   **Audio Trigger:** Nimmt eingehende Audiodaten (vom Audio Panel) auf und gibt ein Steuersignal basierend auf Lautstärke oder spezifischen Frequenzbändern aus.
*   **Math (Add/Multiply/Scale):** Führt mathematische Operationen auf Steuersignalen aus, bevor sie ihr Ziel erreichen.
*   **Timeline Track:** Ein spezialisierter Node, der Keyframe-Daten aus dem Timeline-Panel liest und die entsprechenden Werte ausgibt.

> 🖼️ **[PLATZHALTER SCREENSHOT: Ein zusammengesetztes Bild, das einige Beispiele für Nodes aus verschiedenen Kategorien zeigt und ihre unterschiedlichen Icons sowie Eingänge/Ausgänge hervorhebt.]**