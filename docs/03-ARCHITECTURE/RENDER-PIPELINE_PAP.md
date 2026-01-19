MapFlow Render Pipeline - Prozessablaufplan (PAP)
Dieses Dokument beschreibt den visuellen Datenfluss durch die MapFlow Render Pipeline, gruppiert nach Node-Kategorien.

1. Hauptübersicht: Gesamter Render-Prozess
Trigger Signal (0.0-1.0)
Trigger Signal
Trigger Signal
Media/Texture
Media/Texture
Processed Media
Composited Layer
Composited Layer
Warped Geometry
📺 OUTPUT NODES
Projector Window
NDI Output
Spout Output
Hue Entertainment
🔷 MESH NODES
Quad/Keystone
Grid Warp
Bezier Surface
Polygon
📚 LAYER NODES
Single Layer
Layer Group
All Layers Master
✨ MODULIZER NODES
Effects
Blend Modes
Audio Reactive
Mask Application
📹 SOURCE NODES
Media File
Live Input/Camera
NDI Input
Shader Generator
Image Sequence
🎯 TRIGGER NODES
Audio FFT
Beat Detection
MIDI Input
OSC Input
Keyboard Shortcut
Timer/Random
2. Socket-Typen (Verbindungsarten)
Socket-Typ	Symbol	Beschreibung	Typische Quelle → Ziel
Trigger	🔵	Logisches Signal (0.0-1.0)	Trigger → Source/Modulizer/Layer
Media	🟢	Video/Textur-Stream	Source → Modulizer → Layer
Effect	🟣	Effekt-Konfiguration	Modulizer → Modulizer
Layer	🟠	Kompositions-Layer	Layer → Output
Link	⚪	Master/Slave Verbindung	Node ↔ Node
3. Detailansicht: Trigger-Kategorie
⏱️ Zeit-basierte Trigger
🎹 Externe Trigger
🎵 Audio Triggers
SubBass, Bass,\nMid, High...
Beat Pulse
Volume Level
Note Velocity
OSC Value
Key Press
Periodic Pulse
Random Pulse
Audio FFT\n(9 Frequenzbänder)
Beat Detection\n(Beat Out, BPM)
Volume\n(RMS, Peak)
MIDI Note/CC\n(Device, Channel, Note)
OSC Message\n(/address)
Keyboard\n(Hotkey + Modifiers)
Fixed Timer\n(Interval + Offset)
Random\n(Interval, Probability)
Trigger\nEmpfänger
Audio FFT Output-Konfiguration
Band	Frequenzbereich	Typische Verwendung
SubBass	20-60 Hz	Tiefe Bässe, Kick drum
Bass	60-250 Hz	Basslines
LowMid	250-500 Hz	Untere Mitten
Mid	500-2000 Hz	Vocals, Instrumente
HighMid	2-4 kHz	Presence
UpperMid	4-6 kHz	Obere Presence
Presence	6-10 kHz	Brilliance
Brilliance	10-16 kHz	Air
Air	16-20 kHz	Ultrahohe Frequenzen
4. Detailansicht: Output-Kategorie
💡 Lighting Outputs
🌐 Network Outputs
🖥️ Display Outputs
Layer Input\n(Composited Frame)
Projector Window\n• Fullscreen/Windowed\n• Target Screen\n• Hide Cursor
Preview Window\n• UI Panel Preview\n• Extra Window
NDI Output\n• Broadcast Name\n• 1080p/60fps
Spout (Windows)\n• Shared GPU Texture\n• Zero-copy
Hue Entertainment\n• DTLS Streaming\n• Per-Lamp Control
wgpu Surface
NDI SDK
Spout SDK
Hue Bridge DTLS
5. Separater PAP: Hue Entertainment Flow
▶️ Laufzeit
🔧 Einrichtung (Einmalig)
Ambient
Spatial
Trigger
Bridge IP eingeben
API Link-Button drücken
Username + ClientKey erhalten
Entertainment Groups abrufen
Gruppe auswählen
Frame empfangen\n(von Layer Input)
Mapping Mode anwenden
Mapping Mode?
Ambient:\nDurchschnittsfarbe berechnen
Spatial:\nPro-Lampe Sampling
Trigger:\nBrightness Pulse
DTLS Paket erstellen\n(XY + RGB pro Lampe)
An Bridge senden\n(UDP Port 2100)
Hue-spezifische Konfiguration
Parameter	Beschreibung
bridge_ip	IP-Adresse der Hue Bridge
username	API Whitelist Username
client_key	DTLS Encryption Key
entertainment_area	Ausgewählte Entertainment Zone
lamp_positions	(X, Y) Position pro Lampe (0.0-1.0)
mapping_mode	Ambient / Spatial / Trigger
6. Separater PAP: Audio Analysis Flow
📤 Ausgabe
📊 Analyse
🎤 Audio Eingang
PCM Samples
Sample Buffer
CPAL Audio Device
Ringbuffer\n(1024 samples)
FFT Berechnung\n(RustFFT)
Frequenzband-Energie\n(9 Bänder)
RMS Volume
Peak Volume
Beat Detection\n(Onset + Threshold)
BPM Tracking\n(Tempo Estimation)
AudioAnalysisV2\nStruct
UI Dashboard
Trigger Nodes\n(Audio FFT)
Audio-Konfiguration
Parameter	Beschreibung	Standard
sample_rate	Audio Sample Rate	44100 Hz
buffer_size	FFT Window Size	1024 samples
beat_threshold	Beat Detection Schwelle	0.5
bpm_range	BPM Tracking Bereich	60-200 BPM
7. Separater PAP: Media Playback Flow
🎮 Playback Control
🗂️ Frame Buffer
📼 FFmpeg Decode
Datei öffnen\n(AVFormatContext)
Video Stream finden
Decoder initialisieren\n(AVCodecContext)
Frame lesen\n(av_read_frame)
Frame dekodieren\n(avcodec_decode)
Pixel-Format konvertieren\n(swscale → RGBA)
CPU Frame Buffer
Texture Upload\n(wgpu Queue)
GPU Texture\n(TexturePool)
Play / Pause / Stop
Seek (Position)
Loop Mode
Speed Control
Texture Output\nzu Modulizer/Layer
8. Zusammenfassung: Typischer Datenfluss
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   TRIGGER   │───▶│   SOURCE    │───▶│  MODULIZER  │───▶│    LAYER    │───▶│   OUTPUT    │
│             │    │             │    │             │    │             │    │             │
│ • Audio FFT │    │ • Media     │    │ • Effects   │    │ • Composite │    │ • Projector │
│ • MIDI      │    │ • Camera    │    │ • Blend     │    │ • Opacity   │    │ • NDI       │
│ • OSC       │    │ • NDI       │    │ • Mask      │    │ • Groups    │    │ • Hue       │
│ • Keyboard  │    │ • Shader    │    │             │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                  │                  │                  │
       └──────────────────┴──────────────────┴──────────────────┘
                          Trigger Signal (0.0-1.0)
NOTE

Die Diagramme zeigen den logischen Datenfluss. Die tatsächliche Implementierung nutzt einen 
ModuleEvaluator
, der den Node-Graphen traversiert und 
RenderOp
-Strukturen generiert.