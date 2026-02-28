# Projection Mapping in MapFlow

Projection Mapping ist der Prozess, Videoinhalte präzise auf physische, dreidimensionale Objekte auszurichten und sie über einen flachen Bildschirm hinausgehen zu lassen. MapFlow bietet eine robuste Suite von Werkzeugen – Warping, Keystoning, Masking und Edge Blending – um komplexe architektonische Mappings und Multi-Projektor-Setups zu handhaben.

> 🖼️ **[PLATZHALTER SCREENSHOT: Eine weite Aufnahme, die die Benutzeroberfläche des Mapping-Panels zeigt, mit einer komplexen gekrümmten Oberfläche, die gemappt wird, zusammen mit den Eigenschaften im Inspector.]**

Um auf diese Werkzeuge zuzugreifen, stellen Sie sicher, dass das **Mapping Panel** sichtbar ist (über das View-Menü oder die Toolbar).

---

## Der Mapping-Prozess

Das Mapping in MapFlow findet typischerweise in der **Layer**- oder **Output**-Node-Phase statt. Sie manipulieren die *Geometrie* des finalen Bildes, bevor es die Software verlässt.

1.  **Wählen Sie Ihr Ziel aus:** Wählen Sie im Module Canvas oder in der Timeline den Layer- oder Output-Node aus, den Sie mappen möchten.
2.  **Öffnen Sie das Mapping Panel:** Das Panel wird eine Vorschau der Ausgabe dieses Nodes anzeigen.
3.  **Wählen Sie Ihr Werkzeug:** Wählen Sie in der Symbolleiste des Mapping-Panels das entsprechende Mapping-Werkzeug (Mesh, Keystone, Mask).

---

## 1. Mesh Warping (Gitter-Verzerrung)

Mesh Warping wird für komplexe, nicht rechteckige Oberflächen wie Kugeln, Zylinder oder unregelmäßige Architektur verwendet. Es verformt das Bild mithilfe eines Gitters von Kontrollpunkten.

> 🖼️ **[PLATZHALTER SCREENSHOT: Der Mesh Editor aktiv, der ein 4x4 Bezier-Gitter über einer Videovorschau zeigt, wobei einige Punkte verschoben sind, um das Bild zu krümmen.]**

### Arbeiten mit Meshes

*   **Arten von Meshes:** MapFlow unterstützt verschiedene Mesh-Primitive (Grid, Circle, Cylinder, Sphere, Polygon). Sie können den Typ im Inspector auswählen, wenn das Mesh-Werkzeug aktiv ist.
*   **Gitterauflösung (Resolution):** Für ein Standard-Gitter-Warp (Bezier Surface) können Sie die Anzahl der Kontrollpunkte definieren (z.B. 4x4, 8x8). Eine höhere Auflösung ermöglicht feinere Details, kann aber schwieriger zu handhaben sein.
*   **Punkte verschieben:** Klicken und ziehen Sie einzelne Kontrollpunkte (Vertices) auf dem Gitter im Mapping Panel, um das Bild zu verzerren. Sie können auch Shift-Klick verwenden, um mehrere Punkte auszuwählen.
*   **Bezier-Griffe (Handles):** Gekrümmte Mesh-Typen (wie Bezier-Surfaces) haben Griffe an ihren Kontrollpunkten. Das Ziehen dieser Griffe passt die Krümmung des Bildes zwischen den Punkten an, anstatt es nur linear zu strecken.

> 🚧 **[WIP: Detaillierte Erklärung der spezifischen Mesh-Typen (Polygon vs. Bezier) und wie man sie konvertiert oder bearbeitet. Die aktuelle Implementierung im `MeshEditor` ermöglicht das Konvertieren von Bezier-Surfaces in bearbeitbare Gitter.]**

---

## 2. Keystoning (Eckpunkte anpassen)

Keystoning ist eine einfachere Form der Verzerrung, die speziell zur Korrektur perspektivischer Verzerrungen verwendet wird. Dies passiert, wenn ein Projektor nicht perfekt senkrecht auf eine flache Leinwand projiziert (z.B. von unten oder von oben).

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Keystone-Werkzeug aktiv, das zeigt, wie die 4 Eckpunkte gezogen werden, um ein Bild auf einer angewinkelten Leinwand rechtwinklig zu machen.]**

*   **4-Punkt-Kontrolle:** Keystoning verwendet nur die vier Ecken des Bildes.
*   **Korrektur:** Durch Ziehen der Ecken in der Software, um sie an die physischen Ecken der Leinwand anzupassen, "entzerren" Sie das Bild und lassen es für das Publikum rechteckig erscheinen.
*   **Wann zu verwenden:** Verwenden Sie Keystoning für flache, rechteckige Leinwände. Für alles Gekrümmte oder Unregelmäßige verwenden Sie Mesh Warping.

---

## 3. Masking (Maskieren)

Masking ermöglicht es Ihnen, bestimmte Bereiche Ihrer Projektion zu verbergen. Dies ist entscheidend für das Projection Mapping, da es verhindert, dass Licht auf Bereiche fällt, in denen Sie *kein* Video haben möchten (wie ein Türrahmen, ein Fenster oder über den Rand eines Gebäudes hinaus).

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Masking-Werkzeug aktiv, das zeigt, wie eine benutzerdefinierte Polygonform gezeichnet wird, um einen Teil der Videovorschau auszublenden.]**

*   **Masken erstellen:** Sie können benutzerdefinierte Formen (Polygone oder freihändige Bezier-Kurven) direkt im Mapping Panel zeichnen.
*   **Maske invertieren (Invert):** Masken können so eingestellt werden, dass sie die von Ihnen gezeichnete Form "ausschneiden" (das Video erscheint nur *außerhalb* der Form) oder nur die von Ihnen gezeichnete Form "behalten" (das Video erscheint nur *innerhalb* der Form).
*   **Weiche Kanten (Feathering):** Sie können die Kanten einer Maske oft weicher machen, um ein sanftes Ausblenden zu erzeugen, was nützlich ist, um Projektionen in komplexe Hintergründe zu überblenden.

---

## 4. Edge Blending (Multi-Projektor-Setups)

Wenn Sie ein Bild benötigen, das größer ist, als ein einzelner Projektor erzeugen kann, verwenden Sie mehrere Projektoren nebeneinander. Edge Blending verbindet nahtlos die überlappenden Kanten dieser Projektionen, sodass sie wie ein kontinuierliches Bild aussehen.

> 🖼️ **[PLATZHALTER SCREENSHOT: Das Output Panel, das die Edge Blending Einstellungen (Überlappungsbreite, Gamma-Korrektur) für zwei nebeneinander stehende Displays zeigt.]**

Edge Blending wird im **Output Panel** konfiguriert, da es sich auf das finale Ausgangssignal bezieht, das an die Hardware gesendet wird.

*   **Überlappungszonen (Overlap Zones):** Sie müssen die Projektionen benachbarter Projektoren physisch überlappen lassen (normalerweise 10-20% der Bildbreite).
*   **Software Blending:** MapFlow blendet das Bild auf dem einen Projektor schrittweise aus, während es auf dem anderen Projektor über die Überlappungszone hinweg eingeblendet wird.
*   **Gamma-Korrektur:** Da zwei überlappende Projektoren einen helleren "Hotspot" erzeugen, wendet MapFlow Gamma-Korrekturkurven (die Sie anpassen können) auf die Mischzone an, um eine glatte, gleichmäßige Helligkeit über das gesamte Setup sicherzustellen.
*   **Schwarzpegelanpassung (Black Level Matching):** Projektoren projizieren kein "echtes Schwarz" (sie projizieren dunkelgraues Licht). In einer Überlappungszone verdoppelt sich dieses dunkelgraue Licht und erzeugt ein sichtbares "schwarzes Band", selbst wenn das Video schwarz ist. MapFlow bietet Werkzeuge, um den Schwarzpegel der nicht überlappenden Bereiche künstlich anzuheben, um ihn an die Überlappung anzupassen und die Naht zu verbergen.

> 🚧 **[WIP: Detaillierte Erklärung zur Konfiguration von Output-Nodes und physischen Displays für Edge Blending. Das genaue UI dafür wird noch finalisiert.]**

---

Indem Sie diese Werkzeuge – Warping, Keystoning, Masking und Blending – beherrschen, können Sie fast jeden physischen Raum in eine dynamische Leinwand verwandeln.
