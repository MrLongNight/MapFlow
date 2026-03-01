# Schnellstart-Anleitung

Willkommen bei MapFlow! Diese Schnellstart-Anleitung hilft Ihnen dabei, innerhalb weniger Minuten Ihre erste visuelle Ausgabe zu erstellen. Wir behandeln die absoluten Grundlagen: Importieren von Medien, Erstellen eines einfachen Node-Graphen und das Anzeigen des Ergebnisses.

## Voraussetzungen

Bevor Sie beginnen, stellen Sie sicher, dass:
*   MapFlow installiert ist und ausgeführt wird.
*   Sie eine Videodatei (z. B. MP4) oder ein Bild zur Verwendung bereit haben.

---

## 1. Die Benutzeroberfläche auf einen Blick

Wenn Sie MapFlow starten, sehen Sie mehrere Hauptbereiche:

> 🖼️ **[PLATZHALTER SCREENSHOT: Vollständige Übersicht der MapFlow-Benutzeroberfläche, Hervorhebung des Media Browsers auf der linken Seite, des zentralen Module Canvas und der Output/Inspector-Panels auf der rechten Seite.]**

*   **Media Browser (Links):** Hier finden Sie Ihre Dateien.
*   **Module Canvas (Mitte):** Das Herzstück von MapFlow, wo Sie Ihren visuellen Node-Graphen aufbauen.\n*   **Inspector / Panels (Rechts):** Hier passen Sie die Eigenschaften dessen an, was Sie ausgewählt haben.
*   **Dashboard (Mitte Oben):** Wiedergabesteuerungen und globale Leistungsstatistiken.

---

## 2. Medien importieren

Der erste Schritt besteht darin, eine visuelle Quelle in MapFlow einzubinden.

1.  Suchen Sie das **Media Browser**-Panel auf der linken Seite des Bildschirms.
2.  Navigieren Sie zu dem Ordner, der Ihr Video oder Bild enthält.
3.  Klicken Sie auf die Datei und ziehen Sie sie aus dem Media Browser direkt auf den leeren **Module Canvas** in der Mitte.

> 🖼️ **[PLATZHALTER SCREENSHOT: Die Aktion, eine Videodatei aus dem Media Browser auf den leeren Module Canvas zu ziehen.]**

Ein neuer Node, der Ihre Mediendatei repräsentiert, wird auf dem Canvas erscheinen.

---

## 3. Eine einfache Ausgabe erstellen

MapFlow ist eine Node-basierte Software, was bedeutet, dass Sie verschiedene \"Blöcke\" (Nodes) miteinander verbinden, um einen Datenfluss zu erstellen. Um Ihre Medien zu sehen, benötigen Sie einen Output (Ausgabe).

1.  Klicken Sie mit der rechten Maustaste irgendwo auf eine leere Stelle des **Module Canvas**.
2.  Wählen Sie aus dem erscheinenden Menü **Output > Projector** (oder drücken Sie einfach `Tab`, um das Schnellmenü \"Quick Create\" zu öffnen, und suchen Sie nach \"Output\").

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Rechtsklick-Menü zur Node-Erstellung oder das Quick Create-Popup mit der Auswahl \"Output\".]**

Sie haben nun zwei Nodes: Einen Media-Node (Ihre Quelle) und einen Output-Node.

---

## 4. Die Nodes verbinden

Lassen Sie uns nun das Videosignal von der Quelle zur Ausgabe senden.

1.  Suchen Sie den kleinen Kreis (Socket) auf der rechten Seite Ihres Media-Nodes. Das ist der Ausgangs-Socket (Output).
2.  Klicken Sie auf diesen Socket und ziehen Sie davon weg. Sie werden sehen, dass ein Kabel Ihrer Maus folgt.
3.  Ziehen Sie das Kabel zu dem kleinen Kreis (Socket) auf der linken Seite des Output-Nodes (dem Eingangs-Socket / Input).

> 🖼️ **[PLATZHALTER SCREENSHOT: Ein Kabel, das vom Ausgangs-Socket eines Media-Nodes zum Eingangs-Socket eines Output-Nodes gezogen wird.]**

Sobald die Verbindung hergestellt ist, beginnt MapFlow sofort mit der Verarbeitung des visuellen Signals.

---

## 5. Ihre Visuals betrachten

Um die Ausgabe zu sehen, müssen Sie ein Ausgabefenster öffnen.

1.  Klicken Sie auf den **Output Node**, den Sie gerade erstellt haben, um ihn auszuwählen.
2.  Schauen Sie auf das **Inspector**-Panel (normalerweise auf der rechten Seite).
3.  Suchen Sie die Einstellung **Display Mode** (Anzeigemodus) und ändern Sie sie auf **Windowed** (Fenstermodus) (oder Fullscreen / Vollbild, falls bevorzugt).

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Inspector-Panel, das die Eigenschaften eines Output-Nodes anzeigt, wobei das Dropdown-Menü 'Display Mode' auf Windowed eingestellt ist.]**

Ein neues Fenster öffnet sich und zeigt Ihr Video an!

> 🚧 **[WIP: Erklärung des Output-Panels vs. Output-Node. Das Routing von Ausgabefenstern könnte sich vor dem offiziellen Release noch leicht ändern. Derzeit steuert der Output-Node direkt die Fenstererstellung.]**

---

## 6. Einfache Transformation (Optional)

Möchten Sie Ihr Video bewegen oder skalieren? Dann benötigen Sie einen Layer-Node (Ebenen-Node).

1.  Trennen Sie das Kabel zwischen dem Media-Node und dem Output-Node (Klicken Sie mit der rechten Maustaste auf das Kabel, um es zu löschen, oder klicken Sie auf den Ziel-Socket).
2.  Fügen Sie einen **Layer**-Node hinzu (Rechtsklick > Layer > Standard Layer).
3.  Verbinden Sie den Media-Node mit dem Eingang des Layers.
4.  Verbinden Sie den Ausgang des Layers mit dem Output-Node.
5.  Wählen Sie den **Layer Node** aus und verwenden Sie den **Inspector**, um seine Position, Skalierung (Scale) oder Rotation zu ändern.\n\n> 🖼️ **[PLATZHALTER SCREENSHOT: Ein einfacher Graphen mit 3 Nodes: Media -> Layer -> Output. Der Inspector sollte die Transform-Eigenschaften des Layers anzeigen.]**

---

## Wie geht es weiter?

Herzlichen Glückwunsch! Sie haben Ihre erste MapFlow-Komposition erstellt.

Von hier aus können Sie:
*   [Mehr über die Benutzeroberfläche lernen](UI-Overview.md)
*   [Effekte zu Ihrem Signalfluss hinzufügen](Working-with-Nodes.md)
*   [Ihre Visuals auf Musik reagieren lassen](Audio-Reactivity.md)
*   [Mit dem Projection Mapping Ihrer Ausgabe beginnen](Projection-Mapping.md)