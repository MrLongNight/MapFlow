# Willkommen bei MapFlow

**MapFlow** ist eine zukunftsweisende, modulare **VJ (Video Jockey) Software**, die für hochleistungsfähige visuelle Synthese, Echtzeit-Effekte und professionelles Projection Mapping entwickelt wurde. Mit der Geschwindigkeit und Sicherheit von Rust bietet es Künstlern die Möglichkeit, immersive visuelle Erlebnisse mit beispielloser Flexibilität zu schaffen.

In einer Zeit komplexer visueller Performances schließt MapFlow die Lücke zwischen Benutzerfreundlichkeit und professioneller Leistung. Durch die Nutzung einer **Node-basierten Architektur** wird jeder Parameter zu einem Spielplatz für Automatisierung, Audio-Reaktivität und externe Steuerung.

## Kernkonzepte

Um MapFlow effektiv nutzen zu können, ist es hilfreich, die zugrunde liegenden Prinzipien seines Designs zu verstehen. Diese Anleitung führt Sie in die Kernkonzepte ein, die den Workflow von MapFlow definieren.

---

### Das Node-basierte Paradigma

Im Gegensatz zu herkömmlicher Layer-basierter VJ-Software (bei der Videoclips wie Pfannkuchen übereinandergestapelt werden) ist MapFlow um einen **Node-Graphen** herum aufgebaut.

> 🖼️ **[PLATZHALTER SCREENSHOT: Ein komplexer, aber organisierter Node-Graph, der den Datenfluss von Quellen auf der linken Seite, durch Effekte und Layer in der Mitte, bis zu Ausgaben auf der rechten Seite zeigt.]**

*   **Nodes (Module):** Jede Funktion in MapFlow wird durch einen "Node" (oder ein Modul) repräsentiert. Ein Node ist ein Block, der eine bestimmte Aufgabe erfüllt. Es gibt Nodes zum Abspielen von Videos, zur Erzeugung von Farben, zur Anwendung von Effekten (wie Unschärfe oder Farbkorrektur), zum Compositing von Layern und zur Ausgabe auf Bildschirme.
*   **Sockets & Verbindungen (Kabel):** Nodes haben Eingänge (Sockets auf der linken Seite) und Ausgänge (Sockets auf der rechten Seite). Sie verbinden den Ausgang eines Nodes mit dem Eingang eines anderen, indem Sie "Kabel" (Wires) ziehen.
*   **Der Signalfluss:** Die Kabel repräsentieren den Fluss von Videodaten, Audiodaten oder Steuersignalen. Sie "zeichnen" buchstäblich den Weg, den Ihre Visuals von der Erstellung bis zum finalen Bildschirm nehmen.

Dieser Ansatz ist unglaublich leistungsstark, da er unendliche Routing-Möglichkeiten bietet. Sie sind nicht auf eine feste Pipeline beschränkt.

### Quellen (Sources), Effekte, Layer und Ausgaben (Outputs)

Obwohl der Node-Graph flexibel ist, folgen die meisten Workflows einem gemeinsamen Muster, bei dem Nodes in vier Hauptkategorien unterteilt werden:

1.  **Quellen (Generatoren / Medien):** Dies sind die Startpunkte jedes Signalflusses. Sie erstellen oder importieren Inhalte. Beispiele sind Videodateien, Bildsequenzen, Live-Kamera-Feeds (NDI, Spout) oder generative Shader.
2.  **Effekte (FX):** Diese Nodes modifizieren das eingehende Signal. Beispiele sind Unschärfe, Verzerrung, Verpixelung und Farbkorrektur. Sie können mehrere Effekte aneinanderhängen.
3.  **Layer (Compositing):** Diese Nodes kombinieren mehrere Signale. Sie verwalten Eigenschaften wie Position, Skalierung, Rotation, Deckkraft (Opacity) und Blend-Modi (z.B. Add, Multiply, Screen, Overlay etc.).
4.  **Ausgaben (Outputs):** Diese Nodes senden das finale, zusammengesetzte Bild an physische Displays, Projektoren oder virtuelle Ausgaben (wie NDI oder Spout).

### Projection Mapping

MapFlow ist für komplexe physische Umgebungen konzipiert. Sobald Ihre Visuals erstellt und zusammengesetzt sind, ist die letzte Phase oft die Projektion auf physische Objekte.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das aktive Mapping-Panel, das ein über eine geometrische Form gelegtes Mesh zeigt, welches zur Anpassung verzerrt wird.]**

Dies wird mit speziellen Werkzeugen innerhalb von MapFlow erreicht:
*   **Warping (Mesh Editing):** Die Verformung des Videobildes zur Anpassung an gekrümmte oder unregelmäßige Oberflächen mithilfe eines Gitters (Grid) von Kontrollpunkten.
*   **Keystoning:** Eine spezielle Art der einfachen 4-Punkt-Verzerrung zur Korrektur von perspektivischer Verzerrung, wenn ein Projektor nicht perfekt senkrecht auf eine Leinwand ausgerichtet ist.
*   **Masking (Maskieren):** Das Zeichnen von Formen, um Teile der Videoprojektion auszublenden, um sicherzustellen, dass Licht nur auf die gewünschten Oberflächen fällt.

### Echtzeit-Automatisierung und Steuerung

MapFlow ist für Live-Performances konzipiert. Das bedeutet, dass alles in Echtzeit reagieren muss.

*   **Audio-Reaktivität:** MapFlow analysiert eingehendes Audio (Lautstärke, Frequenzbänder, Beats) und ermöglicht es Ihnen, diese Werte fast jedem Parameter im Node-Graphen zuzuweisen. Sie können einen Layer zum Bass springen lassen oder die Farbe eines Effekts mit den Hi-Hats ändern.
*   **Externe Steuerung (MIDI/OSC):** MapFlow kann durch physische MIDI-Hardware (Keyboards, Fader-Boxen) oder via OSC (Open Sound Control) von Tablets oder anderer Software gesteuert werden. Jeder Drehregler und Schieberegler in der MapFlow-Benutzeroberfläche kann einem externen Controller zugewiesen werden.

---

Nachdem Sie nun die Grundlagen verstanden haben, gehen Sie weiter zur [Übersicht der Benutzeroberfläche](UI-Overview.md), um zu lernen, wie Sie sich in MapFlow zurechtfinden.
