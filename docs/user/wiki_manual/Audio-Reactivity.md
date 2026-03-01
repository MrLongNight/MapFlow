# Audio-Reaktivität

Mit dem Audio-Analyzer von MapFlow können Sie nahezu jeden Parameter in Ihrem Node-Graphen durch Echtzeit-Audioanalysen steuern. Sie können einen Layer zum Bass springen lassen, die Farbe eines Effekts mit den Hi-Hats ändern oder völlig neue Visuals basierend auf einem Beat auslösen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Audio-Panel zeigt den FFT-Spektrum-Analysator, der auf Musik reagiert, wobei die Frequenzbänder Bass, Mid und High hervorgehoben sind.]**

---

## 1. Audio-Eingang konfigurieren

Bevor MapFlow auf Ton reagieren kann, müssen Sie ihm mitteilen, wo es zuhören soll.

1.  Öffnen Sie das **Audio Panel** (erreichbar über das View-Menü oder die Toolbar).
2.  Wählen Sie im Abschnitt **Input Configuration** (Eingangskonfiguration) Ihr Audio-Interface aus dem Dropdown-Menü (z. B. Ihr eingebautes Mikrofon, eine USB-Soundkarte oder ein virtuelles Audio-Loopback-Kabel wie VB-Audio Cable).
3.  Wählen Sie den spezifischen **Eingangskanal** (Input Channel), den Sie überwachen möchten.
4.  Passen Sie den **Gain**-Schieberegler an. Das eingehende Audiosignal (sichtbar in der Wellenform/Spektrum-Anzeige) sollte stark und dynamisch sein, aber nicht ständig ganz oben anschlagen (Clipping/Übersteuern).

> 🖼️ **[PLATZHALTER SCREENSHOT: Der Abschnitt "Input Configuration" des Audio-Panels, der ein ausgewähltes Audiogerät und den Gain-Regler zeigt, der angepasst wird.]**

---

## 2. Der Audio-Analyzer (FFT)

MapFlow verwendet die schnelle Fourier-Transformation (FFT), um das eingehende Audiosignal in verschiedene Frequenzbänder zu unterteilen. Dies ermöglicht es Ihnen, spezifische Teile der Musik zu isolieren.

Das Audio-Panel zeigt eine visuelle Echtzeitdarstellung dieser Frequenzen:

*   **Niedrige Frequenzen (Bass):** Die linke Seite des Spektrums (z. B. Kick-Drums, Basslines).
*   **Mittlere Frequenzen (Mid):** Die Mitte des Spektrums (z. B. Gesang, Gitarren, Synths).
*   **Hohe Frequenzen (High/Treble):** Die rechte Seite des Spektrums (z. B. Hi-Hats, Becken).

MapFlow verfolgt **9 spezifische Frequenzbänder**, sowie die allgemeine **RMS-Lautstärke** (durchschnittliche Lautheit) und die **Peak-Lautstärke** (momentan lauteste Stellen).

---

## 3. Beat-Erkennung (Beat Detection)

Neben der Frequenzanalyse verfügt MapFlow über eine automatische Beat-Erkennung.

*   Der Analyzer versucht, starke, rhythmische Impulse (typischerweise in den tieferen Frequenzen) zu identifizieren, um das BPM (Beats per Minute) der Musik zu bestimmen.
*   Dies ermöglicht es Ihnen, Ereignisse auszulösen oder Effekte exakt auf den Beat zu synchronisieren, anstatt nur auf Lautstärkeschwankungen zu reagieren.

> 🚧 **[WIP: Detaillierte Erklärung, wie man die Empfindlichkeit der Beat-Erkennung feinabstimmt oder das Tempo manuell "tappt" (Tap Tempo), wenn die automatische Erkennung bei komplexen Rhythmen Schwierigkeiten hat.]**

---

## 4. Parameter modulieren (Audio Triggers)

Die wahre Kraft der Audio-Reaktivität liegt darin, diese Audioanalyse-Werte mit visuellen Parametern in Ihrem Node-Graphen zu verknüpfen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Ein Rechtsklick auf einen Parameter-Schieberegler (wie 'Scale' auf einem Layer-Node) und die Auswahl von 'Assign Audio Trigger' aus dem Kontextmenü.]**

1.  Wählen Sie einen Node (z. B. einen Layer-Node) im Module Canvas aus.
2.  Suchen Sie im **Inspector** den Parameter, den Sie animieren möchten (z. B. Skalierung, Rotation oder die Intensität eines Effekts).
3.  Klicken Sie mit der rechten Maustaste auf den Schieberegler oder das Wertefeld des Parameters.
4.  Wählen Sie **Assign Audio Trigger** (Audio-Trigger zuweisen) aus dem Kontextmenü.
5.  Ein kleines Unter-Panel oder ein visueller Indikator erscheint neben dem Parameter.
6.  Wählen Sie, welcher Teil des Audiosignals diesen Parameter steuern soll:
    *   **Source (Quelle):** Wählen Sie ein spezifisches Frequenzband (z. B. Band 1 für tiefen Bass), RMS-Lautstärke oder Peak-Lautstärke.
    *   **Mapping Range (Wertebereich):** Definieren Sie, wie der Audiowert (der normalerweise von 0.0 bis 1.0 geht) auf den Wert des Parameters abgebildet wird. Zum Beispiel könnten Sie einen Bass-Schlag (0.0 -> 1.0) so mappen, dass ein Layer von seiner Originalgröße (1.0) auf die doppelte Größe (2.0) skaliert wird.
    *   **Smoothing/Damping (Glättung):** Fügen Sie ein wenig Glättung hinzu, damit der visuelle Parameter nicht zu sprunghaft reagiert, was eine flüssigere Reaktion auf die Musik erzeugt.

> 🖼️ **[PLATZHALTER SCREENSHOT: Die erweiterten Audio-Trigger-Einstellungen für einen Parameter, die das ausgewählte Quellband, die Mapping-Range-Schieberegler und die Smoothing-Kontrollen zeigen.]**

Indem Sie kreativ verschiedene Frequenzbänder auf verschiedene Effekte und Layer-Eigenschaften mappen, können Sie komplexe, tief synchronisierte visuelle Performances erstellen.