# DOC-B4: Planned Features & Specifications

Dieses Dokument enthält technische Konzepte und Spezifikationen für zukünftige Erweiterungen von MapFlow.

## 1. Kamera-basierte Kalibrierung (Auto-Mapping)
Ziel: Automatische Anpassung des Outputs an die Geometrie der Projektionsfläche.
- **Verfahren**: Strukturiertes Licht (Gray-Codes).
- **Stack**: `nokhwa` für Kamera-Zugriff, native Rust Decoding-Engine.
- **Workflow**: Wizard-gestützter Scan der Oberfläche -> Generierung eines Warp-Meshs.

## 2. GPU Fluid Simulation
Ziel: Echtzeit-Flüssigkeits- und Rauchsimulation als visueller Effekt.
- **Algorithmus**: Stable Fluids (Navier-Stokes) via WGPU Compute Shader.
- **Interaktion**: Dynamische Beeinflussung durch Video-Bewegung (Optical Flow) oder Audio-Impulse.

## 3. Professional Video I/O
Ziel: Nahtloser Austausch mit anderen VJ-Tools.
- **Protokolle**:
    - **Spout (Windows)** / **Syphon (macOS)**: Zero-Copy Texture Sharing auf derselben GPU.
    - **NDI**: Netzwerk-Videoübertragung via LAN.
- **Crate**: `mapflow-io`.

## 4. Universal Link System
Ziel: Master/Slave Verknüpfung von Knoten im Module Canvas.
- **Konzept**: Ein Knoten (Master) steuert die Sichtbarkeit oder Aktivität anderer Knoten (Slaves).
- **Inversion**: Unterstützung für invertierte Links (Slave aktiv, wenn Master inaktiv).

## 5. Multi-PC Support
Ziel: Verteilte Projektion über mehrere Rechner im Netzwerk.
- **Master**: Zentrale Steuerung und Orchestrierung.
- **Slaves**: Reine Player-Instanzen für dedizierte Ausgänge.
- **Sync**: OSC-basierte Synchronisation und Timecode-Abgleich.
