# Externe Steuerung (MIDI & OSC)

MapFlow ist als zentraler Hub für Live-Visual-Performances konzipiert, was bedeutet, dass es nahtlos mit dem Rest Ihres Hardware- und Software-Setups kommunizieren muss. Dies wird durch eine robuste Unterstützung für MIDI (Musical Instrument Digital Interface) und OSC (Open Sound Control) erreicht.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Controller Overlay, das eine virtuelle Darstellung eines MIDI-Controllers mit markierten, gemappten Parametern zeigt.]**

---

## 1. MIDI-Integration

MIDI ermöglicht es Ihnen, physische Hardware-Controller (Keyboards, Fader-Boxen, Drum-Pads) zur Steuerung von MapFlow zu verwenden, was Ihnen eine taktile, praktische Kontrolle über Ihre Visuals anstelle einer reinen Maussteuerung bietet.\n\n### Ein MIDI-Gerät einrichten

1.  Schließen Sie Ihren MIDI-Controller über USB oder ein MIDI-Interface an Ihren Computer an.
2.  Öffnen Sie das **Settings**-Menü (Einstellungen) in MapFlow (oft zu finden unter File > Settings oder in der Dashboard-Toolbar).
3.  Navigieren Sie zum Tab **MIDI/Control**.
4.  MapFlow erkennt angeschlossene MIDI-Geräte automatisch. Wählen Sie Ihren Controller aus der Liste der **Input Devices** (Eingabegeräte) aus und aktivieren Sie ihn.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Settings-Fenster geöffnet beim MIDI-Tab, das eine erkannte MIDI-Tastatur als aktives Input Device anzeigt.]**

### MIDI Learn (Mapping von Steuerungen)

Der einfachste Weg, physische Knöpfe und Fader MapFlow-Parametern zuzuweisen, ist die Verwendung von **MIDI Learn**.

1.  Suchen Sie den **MIDI Learn**-Button (oft ein kleines Icon in der Toolbar oder im Dashboard). Klicken Sie ihn an, um in den Learn-Modus (Lernmodus) zu wechseln. Die Benutzeroberfläche wird normalerweise Parameter, die gemappt werden können, einfärben oder hervorheben.
2.  Klicken Sie auf den Parameter, den Sie in der MapFlow-Benutzeroberfläche steuern möchten (z. B. den Opacity-Schieberegler eines Layer-Nodes im Inspector). Der Parameter wartet nun auf ein eingehendes MIDI-Signal.
3.  Bewegen Sie den Drehregler, Fader oder drücken Sie die Taste an Ihrem physischen MIDI-Controller, die Sie diesem Parameter zuweisen möchten.
4.  MapFlow verknüpft die beiden sofort. Der Parameter in der Benutzeroberfläche reagiert nun auf Ihre physischen Bewegungen.
5.  Klicken Sie erneut auf den **MIDI Learn**-Button, um den Lernmodus zu verlassen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Die Benutzeroberfläche im MIDI Learn-Modus, die einen markierten Parameter (wie Layer Opacity) zeigt, der auf Eingabe wartet, und dann eine Bestätigung, dass er auf 'CC 7' gemappt wurde.]**

### Fortgeschrittenes MIDI-Mapping

*   **Zwei-Wege-Kommunikation:** MapFlow kann MIDI-Feedback *zurück* an Ihren Controller senden. Dies ist nützlich für motorisierte Fader oder Controller mit LED-Ringen und stellt sicher, dass die Hardware immer den Zustand der Software widerspiegelt.
*   **Mappings bearbeiten:** Sie können MIDI-Zuweisungen (Kanal, CC-Nummer, Notennummer) manuell im Settings > MIDI-Tab bearbeiten, wenn Sie präzise Kontrolle benötigen oder komplexe Mappings ohne Verwendung des Learn-Modus erstellen möchten.

---

## 2. OSC (Open Sound Control)

OSC ist ein moderneres, netzwerkbasiertes Protokoll für die Kommunikation zwischen Computern, Synthesizern und anderen Multimedia-Geräten. Es wird häufig für die Fernsteuerung von Tablets (mit Apps wie TouchOSC oder Lemur) oder für die tiefe Integration mit anderer Software (wie Ableton Live oder Max/MSP) verwendet.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Settings-Fenster geöffnet beim OSC-Tab, das die Incoming Port (z.B. 8000) und Outgoing Port/IP Einstellungen zeigt.]**

### OSC konfigurieren

1.  Öffnen Sie das **Settings**-Menü und navigieren Sie zum Tab **OSC**.
2.  **Incoming Port (Eingehender Port):** Dies ist der Netzwerk-Port, auf dem MapFlow auf Befehle lauscht (z. B. 8000). Stellen Sie sicher, dass die sendende Anwendung (Ihr Tablet oder andere Software) an diesen Port auf der IP-Adresse von MapFlow sendet.
3.  **Outgoing Port/IP (Ausgehender Port/IP):** Wenn MapFlow Daten *nach außen* senden muss (z. B. das Senden des aktuellen BPM an eine andere App), konfigurieren Sie hier die Ziel-IP-Adresse und den Port.

### Verwendung von OSC-Befehlen

Im Gegensatz zu MIDI, das numerische Kanäle und Control Changes (CCs) verwendet, verwendet OSC beschreibende, hierarchische Adresspfade (wie eine Website-URL).

*   **Adressstruktur:** Ein typischer OSC-Befehl an MapFlow könnte wie `/layer/1/opacity` aussehen. Das Senden eines Float-Wertes (z. B. `0.5`) an diese Adresse würde die Deckkraft des ersten Layers auf 50% setzen.
*   **Dokumentation:** MapFlow wird in der Entwicklerdokumentation eine detaillierte Liste aller verfügbaren OSC-Adressen enthalten, die es Ihnen ermöglicht, benutzerdefinierte Steuerungsoberflächen zu erstellen oder komplexe Sequenzen aus externen Werkzeugen zu automatisieren.

> 🚧 **[WIP: Eine umfassende Liste der gängigsten OSC-Adresspfade für die Kernparameter von MapFlow (Nodes, Layer, Dashboard-Steuerungen) wird hier in einem zukünftigen Update bereitgestellt.]**

---

Durch die Nutzung von MIDI und OSC können Sie MapFlow von einer eigenständigen Anwendung in eine vollständig integrierte Komponente eines komplexen Live-Performance-Rigs verwandeln.
