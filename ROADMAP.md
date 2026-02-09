# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 2.0
> **Stand:** 2026-02-15 12:00
> **Zielgruppe:** @Projektleitung und Entwickler-Team
> **Projekt-Version:** 0.2.0

---

## 📋 Inhaltsverzeichnis

1. [Fokus & Ziele für Release 1.0](#fokus--ziele-für-release-10)
2. [Feature-Status-Übersicht](#feature-status-übersicht)
3. [Architektur und Crate-Übersicht](#architektur-und-crate-übersicht)
4. [Multi-PC-Architektur (Phase 8)](#multi-pc-architektur-phase-8)
5. [Arbeitspakete für @jules](#arbeitspakete-für-jules)
6. [Task-Gruppen (Adaptiert für Rust)](#task-gruppen-adaptiert-für-rust)
7. [Implementierungsdetails nach Crate](#implementierungsdetails-nach-crate)
8. [Technologie-Stack und Entscheidungen](#technologie-stack-und-entscheidungen)
9. [Build- und Test-Strategie](#build--und-test-strategie)

---

## Fokus & Ziele für Release 1.0

Basierend auf dem aktuellen Status und den Projektzielen für die erste produktive Version (v1.0) sind dies die Prioritäten:

1. **Stabilität (Core):** Fehlerfreie FFT-Analyse, robustes Layer-Compositing und stabiles Window-Management.
2. **Performance:** Minimierung von Latenzen in der Audio-Trigger-Kette und GPU-optimiertes Rendering (WGPU).
3. **Benutzererfahrung:** Intuitive Steuerung via MIDI/OSC und ein konsistentes Cyber-Dark UI-Thema.
4. **Konnektivität:** Volle Unterstützung für NDI (In/Out) und Spout (Windows) für professionelle VJ-Workflows.

---

## Feature-Status-Übersicht

### General Updates

* ✅ **Rebranding: VjMapper -> MapFlow** (COMPLETED 2025-12-22)
  * ✅ Rename Project (2025-12-22)
  * ✅ Update UI Strings & Docs (2025-12-22)
  * ✅ Refactor Crates/Namespaces (2025-12-22)

---

### Phase 1: Core Engine & WGPU Rendering

* ✅ **WGPU Renderer Core** (COMPLETED)
  * ✅ Cross-platform Windowing (Winit)
  * ✅ Texture Pool & Resource Management
  * ✅ Multi-Window Support (Phase 1, Month 3)
* ✅ **Layer Compositing System** (COMPLETED)
  * ✅ 14 Blend Modes (Normal, Add, Multiply, etc.)
  * ✅ Hierarchical Groups (Phase 1, Month 4)
  * ✅ Opacity, Solo, Bypass per Layer
  * ✅ Resize Modes (Fill, Fit, Stretch)
* ✅ **Audio Engine V2** (COMPLETED)
  * ✅ Multi-backend (CPAL: ASIO, WASAPI, CoreAudio)
  * ✅ High-precision FFT Analysis (9 Bands)
  * ✅ BPM & Beat Detection (Phase 1, Month 5)
  * ✅ Thread-safe Data Flow

---

### Phase 2: User Interface & Interaction

* ✅ **Egui Integration** (COMPLETED)
  * ✅ Context Management & Input Handling
  * ✅ Multi-Window UI Support
* ✅ **Cyber Dark Theme** (COMPLETED)
  * ✅ Standardized Color Palette
  * ✅ Modern Panel Headers & Containers
  * ✅ Responsive Layout System
* ✅ **Control Panels** (COMPLETED)
  * ✅ Layer Inspector (Phase 2, Month 2)
  * ✅ Mapping Editor (Phase 2, Month 3)
  * ✅ Audio Analysis Monitor (Phase 2, Month 4)
  * ✅ Shortcuts & MIDI Learn UI

---

### Phase 3: External Connectivity

* ✅ **NDI Integration** (COMPLETED)
  * ✅ NDI Input (Discovery & Receiving)
  * ✅ NDI Output (Phase 3, Month 2)
  * ✅ Hardware Acceleration (YCoCg)
* ✅ **Spout/Syphon Support** (COMPLETED)
  * ✅ Spout 2.0 (Windows)
  * ✅ Syphon (macOS - via feature gates)
* ✅ **WebSocket API** (COMPLETED)
  * ✅ JSON-RPC Command Set
  * ✅ Subprotocol Authentication (Phase 3, Month 4)

---

### Phase 4: Control & Automation

* ✅ **MIDI System** (COMPLETED)
  * ✅ Port Discovery & Selection
  * ✅ 14-bit CC / High-res Support
  * ✅ Global MIDI Learn (Phase 4, Month 2)
* ✅ **OSC Server** (COMPLETED)
  * ✅ Custom Address Mapping
  * ✅ Bi-directional Feedback
* 🟡 **Automation System** (IN PROGRESS)
  * ✅ LFO / Oscillator Modules
  * ⬜ Sequence / Timeline Editor (Phase 4, Month 5)

---

### Phase 5: Effects & Processing

* ✅ **Shader Effects Pipeline** (COMPLETED)
  * ✅ Real-time WGSL Compilation
  * ✅ Global Effect Chain
  * ✅ Layer-specific Effects (Phase 5, Month 2)
* ✅ **Standard Effect Library** (COMPLETED)
  * ✅ Color Correction (LUT, Levels)
  * ✅ Distortion (Glitch, Pixelate)
  * ✅ Geometry (Mirror, Kaleidoscope)
  * ✅ Stylize (Film Grain, Vignette)

---

### Phase 6: Project Management & I/O

* ✅ **Project Persistence** (COMPLETED)
  * ✅ AppState Serialization (JSON)
  * ✅ Auto-save Functionality
  * ✅ Assets Relocation (relative paths)
* ✅ **Undo/Redo System** (COMPLETED)
  * ✅ Global History Manager
  * ✅ Atomic Actions (Phase 6, Month 3)

---

### Phase 7: Stability & Distribution

* ✅ **CI/CD Pipeline** (COMPLETED)
  * ✅ GitHub Actions (Build & Test)
  * ✅ Security Scanning (CodeQL)
  * ✅ Automated Releases
* ✅ **Installer & Packaging** (COMPLETED)
  * ✅ Windows WiX Installer
  * ✅ App Bundle (macOS)
  * ✅ AppImage (Linux)

---

### Phase 8: Multi-PC & Large Scale (Planned)

* ⬜ **MapFlow Sync Protocol**
  * ⬜ Clock Synchronization (PTP)
  * ⬜ Distributed Layer Rendering
* ⬜ **Hardware Info Overlay**
  * ⬜ GPU/CPU Monitoring in UI

---

## Arbeitspakete für @jules

### Grundlagen (implementiert)

* ✅ Controller-Profil (89 MIDI-Mappings in `ecler_nuo4.rs`)
* ✅ Element-Datenstruktur (30 Elemente in `elements.json`)
* ✅ MIDI-Learn Modul (`midi_learn.rs`)
* ✅ Overlay UI Panel Grundgerüst (`controller_overlay_panel.rs`)
* ✅ Hintergrundbild (`resources/controllers/ecler_nuo4/background.jpg`)

### Overlay UI Features

* ⚠️ **Hintergrundbild anzeigen** - Mixer-Foto als Background (841x1024 px) (Asset fehlt)
* ✅ **Skalierbares Panel** - Zoom 30%-100% via Slider
* ⬜ **PNG-Assets für Elemente** - Knobs, Fader, Buttons (vom User bereitgestellt)
* ⬜ **Exakte Platzierung** - Koordinaten aus `elements.json` auf Foto mappen
* ⬜ **Animation** - Knobs rotieren (0-270°), Fader bewegen sich

### Interaktive Features

* ✅ **Rahmen um MIDI-Elemente** mit Farbzuständen:
  * Kein Rahmen / Grau = Inaktiv
  * 🟡 Gelb pulsierend = MIDI Learn aktiv
  * 🟢 Grün = Wert ändert sich
  * ⚪ Weiß = Hover
  * 🔵 Blau = Ausgewählt
  * 🎨 **NEU: Zuweisungs-Modus**: Grün (Frei) / Blau / Lila / Orange (Belegt)
* ✅ **Mouseover-Tooltip** pro Element:
  * Element-Name, MIDI-Typ, Channel, CC/Note, Wert
  * ✅ **Aktuelle Zuweisung** (MapFlow/Streamer.bot/Mixxx) anzeigen

### MIDI Learn Buttons

* ✅ **MapFlow MIDI Learn** - Button im Panel
* ✅ **Streamer.bot MIDI Learn** - Mit Eingabefeld für Funktionsname
* ✅ **Mixxx MIDI Learn** - Mit Eingabefeld für Funktionsname
* ✅ **Toolbar Toggle** - 🎛️ Button zum Ein/Ausblenden des Overlays

### Zuweisungs-Editor

* ✅ **Element-Liste** - Alle 30 MIDI-Elemente tabellarisch
* ✅ **Filter-Ansichten**:
  * Alle Zuweisungen
  * Nur MapFlow-Zuweisungen
  * Nur Streamer.bot-Zuweisungen
  * Nur Mixxx-Zuweisungen
  * Freie Elemente (ohne Zuweisung)
* ✅ **Bearbeiten** - Zuweisung löschen via 🗑 Button
* ✅ **Bearbeiten** - Zuweisung auswählen via Dropdown (Weg 2)
* ✅ **Global MIDI Learn** - Zuweisung per Mouse-Hover über UI-Elemente (Weg 1)
* ✅ **Persistierung** - MidiAssignment in UserConfig (config.json)

* 🟡 **WGPU Rendering Fixes**
  * ⬜ R32Float Validation Error in OscillatorRenderer
  * ⬜ Texture Bind Group Lifetime issue in Compositor
  * ⬜ Shader Hot-Reloading stability (Linux)

---

### Task-Gruppen (Adaptiert für Rust)

Die folgenden Node-Typen haben vollständige UI-Panels:

#### Part-Typen (6 Hauptkategorien)

* ✅ **Trigger** - Schaltet andere Nodes
  * ✅ AudioFFT Panel (Band-Auswahl, Threshold-Slider, 11 Outputs)
  * ✅ Random Panel (Min/Max Interval, Probability)
* ✅ **Input** - Liefert Bilddaten
  * ✅ Video File Panel (File Picker, Loop Toggle)
  * ✅ WebCam Panel (Device Selection)
  * ✅ NDI Input Panel (Source Discovery)
  * ✅ Spout Input Panel (Windows Only)
  * ✅ Solid Color Panel (RGBA Picker)
* ✅ **Generator** - Erzeugt Bilder pro Frame
  * ✅ Oscillator Panel (Sine/Square/Noise, Speed, Amplitude)
  * ✅ Particle System Panel (Count, Lifetime, Physics)
* ✅ **Adjustment** - Verändert Bilddaten (Single Layer)
  * ✅ Color Grade Panel (Brightness, Contrast, Saturation)
  * ✅ Transform Panel (Pos, Scale, Rotation)
  * ✅ Crop Panel (Left, Right, Top, Bottom)
* ✅ **Composition** - Kombiniert Layers (Multi Layer)
  * ✅ Group Panel (Z-Order, Master Opacity)
* ✅ **Output** - Endstation
  * ✅ Window Output Panel (Fullscreen, Monitor Select)
  * ✅ NDI Output Panel (Sender Name)

#### Socket-Typen (für Wire-Kompatibilität)

* ✅ Trigger (Signal-Flow)
* ✅ Media (Bild/Video-Daten)
* ✅ Effect (Effekt-Kette)
