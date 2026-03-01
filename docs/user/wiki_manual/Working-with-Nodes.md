# Arbeiten mit Nodes

Die Stärke von MapFlow liegt in seiner Node-basierten Architektur. Anstatt Layer (Ebenen) einfach übereinanderzustapeln, bauen Sie einen benutzerdefinierten \"Signalfluss\" auf, indem Sie verschiedene Module miteinander verbinden. Dies ermöglicht eine beispiellose Flexibilität beim Routing, der Effektverarbeitung und der Performance-Steuerung.

## Der Module Canvas

Der Module Canvas (der große zentrale Bereich der Benutzeroberfläche) ist Ihr Arbeitsbereich.

> 🖼️ **[PLATZHALTER SCREENSHOT: Der leere Module Canvas, bereit für Nodes.]**

### 1. Nodes hinzufügen

Es gibt mehrere Möglichkeiten, Nodes zum Canvas hinzuzufügen:

*   **Drag & Drop:** Der einfachste Weg, Medien (Videos, Bilder) hinzuzufügen, besteht darin, sie direkt aus dem Media Browser auf den Canvas zu ziehen. Dadurch wird automatisch ein Media-Node erstellt.
*   **Rechtsklick-Menü:** Klicken Sie mit der rechten Maustaste irgendwo auf den leeren Canvas, um das kategorisierte Node-Menü zu öffnen. Durchsuchen Sie Sources, Effects, Layers, Outputs etc. und wählen Sie den benötigten Node aus.
*   **Quick Create (Tab):** Drücken Sie die `Tab`-Taste auf Ihrer Tastatur. Dadurch öffnet sich ein durchsuchbares Popup-Fenster. Fangen Sie an, den Namen des Nodes (z. B. \"Blur\", \"Projector\", \"Oscillator\") einzutippen, und drücken Sie Enter, um ihn hinzuzufügen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Quick Create (Tab) Popup, das Suchergebnisse für \"Blur\" anzeigt.]**

### 2. Node-Struktur

Ein typischer Node in MapFlow besteht aus drei Hauptteilen:

> 🖼️ **[PLATZHALTER SCREENSHOT: Eine Nahaufnahme eines einzelnen Nodes (z. B. eines Blur-Effekts), die die Eingänge, den Hauptkörper und die Ausgänge hervorhebt.]**

1.  **Header/Body (Kopf/Körper):** Zeigt den Namen des Nodes, den Typ und manchmal eine kleine Vorschau oder grundlegende Bedienelemente (wie einen Bypass-Schalter oder einen einzelnen markanten Schieberegler) an. Durch Klicken auf den Körper wird der Node ausgewählt.
2.  **Eingangs-Sockets (Linke Seite):** Hier treten Daten in den Node ein. Ein Node kann einen Eingang haben (z. B. ein einfacher Effekt, der einen Videostream entgegennimmt) oder mehrere Eingänge (z. B. ein Mix-Layer, der zwei Videostreams und ein Steuersignal aufnimmt).
    *   *Hinweis: Einige Nodes, wie Generatoren oder Media Sources, haben keine Videoeingänge, da sie das Signal selbst erzeugen.*
3.  **Ausgangs-Sockets (Rechte Seite):** Hier verlassen Daten den Node. Die meisten Nodes haben mindestens einen Ausgang, um das verarbeitete Signal weiter in der Kette zu übergeben.

### 3. Nodes verbinden (Der Signalfluss)

Damit etwas passiert, müssen Sie Nodes miteinander verbinden.

1.  Fahren Sie mit der Maus über einen **Ausgangs-Socket** (rechte Seite). Der Mauszeiger wird sich ändern.
2.  Klicken und ziehen Sie von dem Socket weg. Sie zeichnen nun ein \"Kabel\" (Wire).
3.  Ziehen Sie das Kabel zu einem **Eingangs-Socket** (linke Seite) an einem anderen Node. Der Socket wird hervorgehoben, wenn Sie in der Nähe sind. Lassen Sie die Maustaste los, um die Verbindung herzustellen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Eine Animation oder Bildsequenz, die zeigt, wie ein Kabel vom Ausgang einer Quelle zum Eingang eines Effekts gezogen wird.]**

*   **Verbindungen löschen:** Um ein Kabel zu entfernen, klicken Sie mit der rechten Maustaste irgendwo auf das Kabel selbst. Alternativ klicken Sie auf den Ziel-Socket und ziehen das Kabel in den leeren Raum weg.
*   **Mehrfache Verbindungen:** Sie können einen Ausgang mit *mehreren* Eingängen verbinden. Zum Beispiel können Sie eine Videoquelle gleichzeitig an drei verschiedene Effektketten senden.

### 4. Node-Eigenschaften (Der Inspector)

Wenn Sie einen Node durch Klicken auf seinen Körper auswählen, erscheinen seine detaillierten Einstellungen im **Inspector-Panel** (normalerweise auf der rechten Seite des Bildschirms).

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Inspector-Panel, das die Eigenschaften eines ausgewählten \"Color Correction\"-Nodes anzeigt, mit Schiebereglern für Farbton (Hue), Sättigung (Saturation), Helligkeit (Brightness) und Kontrast (Contrast).]**

Hier passen Sie Parameter an, ändern Blend-Modi, legen Transform-Werte (Position/Skalierung) fest oder konfigurieren die Audio-Reaktivität für diesen spezifischen Node.

### 5. Den Graphen verwalten

Wenn Ihre Kompositionen wachsen, kann der Canvas unübersichtlich werden. Verwenden Sie diese Werkzeuge, um Ordnung zu halten:

*   **Auswahl (Selection):** Klicken Sie, um einen einzelnen Node auszuwählen. Shift-Klick, um mehrere Nodes auszuwählen. Sie können auch ein Auswahlfeld um mehrere Nodes ziehen (Click & Drag).
*   **Bewegen (Moving):** Klicken und ziehen Sie den Header/Körper eines Nodes, um ihn auf dem Canvas zu verschieben.
*   **Löschen (Deletion):** Wählen Sie einen oder mehrere Nodes aus und drücken Sie die `Entf`- oder `Backspace`-Taste. (Dadurch werden auch alle mit ihnen verbundenen Kabel gelöscht).
*   **Bypass (Stummschalten):** Viele Effekt-Nodes haben einen Bypass-Schalter im Header oder im Inspector. Dies deaktiviert den Effekt vorübergehend, ohne die Verbindungen zu unterbrechen, was nützlich für schnelle Vergleiche während eines Live-Sets ist.

---

## Beispiel-Workflows

Hier sind einige gängige Node-Graph-Muster, um Ihnen den Einstieg zu erleichtern:

### Die einfache Kette
`Media Source -> Effect (z. B. Blur) -> Output (Projector)`
Das einfachste Setup. Das Video wird abgespielt, unscharf gemacht und auf den Bildschirm ausgegeben.

> 🖼️ **[PLATZHALTER SCREENSHOT: Eine einfache Kette mit 3 Nodes: Media -> Blur -> Output.]**

### Layer zusammenfügen (Compositing)
`Media Source 1 -> Input A (Mix Layer)`
`Media Source 2 -> Input B (Mix Layer)`
`(Mix Layer Output) -> Output (Projector)`
Zwei Videos werden mit einem Layer-Node kombiniert (z. B. Add, Multiply oder Crossfade).

> 🖼️ **[PLATZHALTER SCREENSHOT: Ein Y-förmiger Graph, der zeigt, wie zwei Quellen in einen Layer-Node fließen, welcher dann an einen Output gesendet wird.]**

### Routing einer Quelle an mehrere Ausgaben
`Generative Source -> (Split Wire) -> Mapping Layer 1 (Output 1)`
`(Split Wire) -> Mapping Layer 2 (Output 2)`
Ein einzelnes generatives Visual wird an zwei verschiedene Mapping-Layer gesendet, die dann auf zwei verschiedenen physischen Projektoren ausgegeben werden.

> 🖼️ **[PLATZHALTER SCREENSHOT: Eine einzelne Quelle mit einem Kabel, das sich in zwei verschiedene Layer/Output-Ketten aufteilt.]**

Für eine detaillierte Liste aller verfügbaren Nodes und deren Funktion, siehe das [Node Referenz-Handbuch](Node-Reference.md).
