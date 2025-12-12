# Machbarkeitsstudie: Multi-PC Architektur

## 1. Zusammenfassung
Die Erweiterung von VjMapper für den Multi-PC-Betrieb ist technisch **machbar** und würde die Software auf ein professionelles Level heben.

Basierend auf den Anforderungen (Option A: Streaming, Unterstützung alter Hardware/Raspberry Pi, professioneller Installer) empfehle ich einen **Single-Binary-Ansatz** mit integriertem **NDI-Streaming**. Dies ermöglicht es, dieselbe ausführbare Datei sowohl als "Master" als auch als "Player" zu nutzen, gesteuert durch Startparameter.

## 2. Architektur-Analyse

Wir haben zwei Hauptstrategien evaluiert:

### Option A: Video-Streaming (Empfohlen)
Der Master-PC berechnet das gesamte Bild (Rendering, Mapping, Effekte) und sendet das fertige Videosignal über das Netzwerk an die Clients.
*   **Technologie:** NDI (Network Device Interface) oder GStreamer (RTP/RTSP).
*   **Vorteil:** Clients (Projektor-PCs) müssen nur Video dekodieren. Sehr geringe GPU-Anforderungen am Client. Perfekt für Raspberry Pi 4/5 und ältere Laptops.
*   **Nachteil:** Hohe Netzwerkbandbreite (Gigabit LAN empfohlen).

### Option B: Distributed Rendering
Der Master sendet nur Steuerbefehle (OSC/Netzwerk-Events). Die Clients berechnen die Grafik selbst.
*   **Vorteil:** Geringe Netzwerkbandbreite.
*   **Nachteil:** Clients benötigen leistungsfähige GPUs (wie der Master). Synchronisation (Frame-Sync) ist extrem komplex. Assets (Videos) müssen auf alle Clients kopiert werden.

**Entscheidung:** Für die Anforderung "ältere Hardware/Raspi" und "sofortige Machbarkeit" ist **Option A (Streaming)** der klare Gewinner.

## 3. Technische Umsetzung: Der "Single-Binary"-Ansatz

Anstatt zwei separate Apps zu entwickeln, refaktorieren wir `VjMapper` so, dass es in zwei Modi starten kann. Dies erfüllt den Wunsch nach einem professionellen Installer und einfacher Wartung.

### Konzept
Die `main.rs` wird zur Weiche:
1.  **Master-Mode (Standard):** Lädt `AppEditor` (mit GUI, Rendering-Engine, NDI-Sender).
2.  **Player-Mode (`--player`):** Lädt `AppPlayer` (ohne GUI, Vollbild, NDI-Receiver).

### Technologie-Stack
*   **Streaming:** Wir nutzen **NDI** (via Rust Bindings wie `ndi` oder `grafton-ndi`). NDI ist der Industriestandard für latenzfreies Video im LAN.
    *   *Lizenz-Hinweis:* Das NDI SDK ist proprietär. Wir können es nicht direkt bundeln, aber den User auffordern, die "NDI Runtime" zu installieren (ähnlich wie bei OBS).
    *   *Alternative:* GStreamer (Open Source), falls NDI-Lizenzierung ein Problem darstellt.
*   **Grafik:** `wgpu` wird weiterhin genutzt. Im Player-Mode rendert `wgpu` lediglich eine Fullscreen-Textur (den empfangenen Stream).

## 4. Hardware-Einschätzung

### Raspberry Pi (4 & 5)
*   **Machbarkeit:** **Hoch**.
*   **Voraussetzung:** Der Pi muss Hardware-Videodecodierung nutzen. NDI bietet SDKs für ARM Linux (Raspberry Pi OS).
*   **Performance:** Ein Pi 4 schafft problemlos 1080p60 NDI-Streams. 4K könnte am Limit sein, ist aber mit Pi 5 realistisch.
*   **Betriebssystem:** Raspberry Pi OS (64-bit).

### Ältere Laptops / Mini-PCs
*   **Machbarkeit:** **Sehr Hoch**.
*   **Voraussetzung:** Gigabit-Ethernet. Selbst integrierte Grafikkarten (Intel HD Graphics) schaffen Video-Decodierung mühelos.

## 5. Installer & Deployment

Wie gewünscht, nutzen wir die bestehende Modulstruktur. Der Installer (WiX für Windows, Deb für Linux) bleibt eine einzige Datei.

*   **Windows Installer (`.msi`):**
    *   Der User wählt im Installer: "Full Installation" oder "Player Only" (oder beides).
    *   Technisch wird immer dieselbe `VjMapper.exe` installiert.
    *   Die Auswahl steuert lediglich, welche **Verknüpfungen** (Shortcuts) erstellt werden:
        1.  `VjMapper` (Startet normal)
        2.  `VjMapper Player` (Startet mit `VjMapper.exe --player`)
*   **Linux (`.deb`):**
    *   Ähnliches Prinzip via `.desktop` Files.

Dies wirkt für den Endanwender wie zwei getrennte Apps, ist aber entwicklerseitig hocheffizient (nur eine Codebasis).

## 6. Aufwandsschätzung

Um dieses Feature als **MVP (Minimum Viable Product)** umzusetzen:

1.  **Refactoring `main.rs` & Architektur:** 2-3 Tage.
    *   Aufsplitten der monolithischen `App` in `Editor` und `Player` Module.
2.  **NDI Integration (Prototyp):** 5-7 Tage.
    *   Senden der `wgpu` Textur an NDI (Master).
    *   Empfangen von NDI und Anzeige in `wgpu` (Player).
3.  **Installer Anpassung (WiX):** 1-2 Tage.
    *   Konfiguration der Shortcuts und Features.
4.  **Testing & Optimierung:** 3-5 Tage.
    *   Latenz-Optimierung, Stabilitätstests.

**Gesamt:** Ca. **2-3 Wochen** Entwicklungszeit für einen robusten Prototypen.

## 7. Fazit & Nächste Schritte

Das Vorhaben ist **sinnvoll und machbar**. Es positioniert VjMapper als ernstzunehmende Alternative zu teuren Profi-Lösungen (wie Resolume Arena oder MadMapper).

**Empfohlener erster Schritt:**
Erstellung eines Proof-of-Concept (POC), der lediglich ein Videobild von einer `VjMapper`-Instanz zu einer anderen über LAN sendet, um die Latenz und Performance von Rust+NDI zu verifizieren.
