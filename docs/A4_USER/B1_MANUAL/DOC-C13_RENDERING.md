# Rendering & Export

Vorce bietet leistungsstarke Rendering-Optionen, mit denen du deine Live-Visuals und Projektions-Mappings nicht nur in Echtzeit auf externe Displays ausgeben, sondern auch als hochauflösende Videodateien exportieren kannst.

## Export Dialog

Der Export-Dialog ermöglicht dir die genaue Kontrolle über die Rendereinstellungen, das Zielformat und die verwendeten Hardware-Ressourcen.

![Screenshot: Der Render-Dialog mit ausgeklappten GPU-Settings](docs/assets/missing/export-dialog-gpu.png)

### Wichtige Einstellungen

* **Auflösung (Resolution):** Bestimmt die Größe des finalen Videos. Wähle zwischen Standard-Formaten (z.B. 1080p, 4K) oder gib benutzerdefinierte Werte ein.
* **Framerate (FPS):** Die Anzahl der Bilder pro Sekunde. Höhere Werte (z.B. 60 FPS) sorgen für flüssigere Animationen, erfordern aber mehr Rechenleistung.
* **Format:** Wähle den Video-Codec (z.B. H.264, ProRes).
* **GPU-Beschleunigung (Hardware Encoding):** Aktiviere dies, um deine Grafikkarte (NVIDIA NVENC, AMD AMF oder Apple VideoToolbox) für einen deutlich schnelleren Export zu nutzen. Diese Einstellungen findest du unter den erweiterten GPU-Settings.
* **Anti-Aliasing:** Glättet die Kanten von 3D-Geometrie und Mesh-Warpings.

### Render-Vorgang starten

1. Klicke im Hauptmenü auf **Datei > Rendern / Export**.
2. Konfiguriere deine gewünschten Einstellungen im Dialog.
3. Klicke auf den Button **Export Starten**.
4. Ein Fortschrittsbalken zeigt dir den Status des Renders an. Du kannst den Vorgang jederzeit abbrechen.

**Tipp 💡:**
> Wenn du aufwendige Partikel-Simulationen (Bevy Particles) oder komplexe Shader renderst, stelle sicher, dass du genügend VRAM zur Verfügung hast und speichere dein Projekt, bevor du den Export startest!
