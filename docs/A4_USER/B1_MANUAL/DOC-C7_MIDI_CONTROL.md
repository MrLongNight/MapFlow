´9# MIDI User Guide - MapFlow

> **Version:** 1.0
> **Stand:** 2025-12-27
> **Status:** MIDI-System vollständig verdrahtet

---

## 📋 Inhaltsverzeichnis

1. [Übersicht](#übersicht)
2. [System-Architektur](#system-architektur)
3. [User Workflow: MIDI Einrichten](#user-workflow-midi-einrichten)
4. [User Workflow: MIDI Learn](#user-workflow-midi-learn)
5. [User Workflow: Controller Overlay](#user-workflow-controller-overlay)
6. [Bekannte Probleme & Einschränkungen](#bekannte-probleme--einschränkungen)
7. [Technische Details](#technische-details)

---

## Übersicht

MapFlow unterstützt MIDI-Eingabe für:
- **Trigger-Nodes** im Module Canvas (Steuerung von Medien/Effekten)
- **Controller Overlay** (visuelle Anzeige des Ecler NUO 4 Mixers)

### Was funktioniert:
✅ MIDI-Ports werden automatisch erkannt
✅ Auto-Connect zum ersten verfügbaren Port
✅ Port-Auswahl in Settings
✅ MIDI Learn für Trigger-Nodes
✅ Controller Overlay zeigt MIDI-Werte in Echtzeit

### Was noch fehlt:
❌ MIDI-zu-Layer/Effect Routing (direkte Parametersteuerung)

---

## System-Architektur

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          MIDI DATENFLUSS                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐                                                       │
│  │ MIDI Device  │  (z.B. Ecler NUO 4)                                   │
│  └──────┬───────┘                                                       │
│         │ USB/MIDI                                                      │
│         ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ midir Crate (Rust)                                               │   │
│  │ Datei: mapmap-control/src/midi/input.rs                          │   │
│  │ Struct: MidiInputHandler                                         │   │
│  │ - new() → Initialisierung                                        │   │
│  │ - list_ports() → Alle verfügbaren Ports                          │   │
│  │ - connect(index) → Verbindet zu Port                             │   │
│  │ - poll_message() → Holt nächste MIDI-Message                     │   │
│  └──────┬───────────────────────────────────────────────────────────┘   │
│         │                                                               │
│         ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │ main.rs Event Loop (Event::AboutToWait)                          │   │
│  │ Zeilen: 451-460                                                  │   │
│  │                                                                  │   │
│  │ while let Some(msg) = handler.poll_message() {                   │   │
│  │     controller_overlay.process_midi(msg);   ──► Overlay UI       │   │
│  │     module_canvas.process_midi_message(msg); ──► MIDI Learn      │   │
│  │ }                                                                │   │
│  └──────┬───────────────────────────────────────────────────────────┘   │
│         │                                                               │
│         ├─────────────────────────┬─────────────────────────────────────│
│         ▼                         ▼                                     │
│  ┌──────────────────┐      ┌──────────────────────────────────────┐     │
│  │ Controller       │      │ Module Canvas (module_canvas.rs)    │     │
│  │ Overlay Panel    │      │                                      │     │
│  │ (controller_     │      │ process_midi_message():              │     │
│  │ overlay_panel.rs)│      │   if midi_learn_part_id is set:      │     │
│  │                  │      │     → Store in learned_midi          │     │
│  │ - Zeigt Knobs    │      │                                      │     │
│  │ - Zeigt Fader    │      │ show():                              │     │
│  │ - Live-Werte     │      │   → Apply learned_midi to Part       │     │
│  └──────────────────┘      └──────────────────────────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## User Workflow: MIDI Einrichten

### Schritt 1: App starten
MapFlow verbindet sich **automatisch** zum ersten verfügbaren MIDI-Port.

Im Log (`logs/mapflow_*.log`) erscheint:
```
INFO  MIDI initialized
INFO  Available MIDI ports: ["Port 1", "Port 2"]
INFO  Auto-connected to MIDI port: Port 1
```

### Schritt 2: Settings öffnen

**Menü:** `File` → `Settings` (oder Toolbar ⚙️)

### Schritt 3: MIDI-Section aufklappen

Im Settings-Fenster gibt es eine **klappbare Section** "🎹 MIDI".

**UI-Elemente:**

| Element | Beschreibung |
|---------|--------------|
| **Status** | 🟢 Connected (grün) oder 🔴 Disconnected (rot) |
| **MIDI Port** | Dropdown mit allen verfügbaren Ports |
| **🔄 Refresh Ports** | Button zum Aktualisieren der Port-Liste |
| **X port(s) available** | Anzahl gefundener Ports |

### Schritt 4: Port wechseln (falls nötig)

1. **MIDI Port Dropdown** klicken
2. Gewünschten Port auswählen
3. MapFlow disconnectet vom alten Port und connectet zum neuen

**Log-Ausgabe:**
```
INFO  Connected to MIDI port: Ecler NUO 4
```

---

## User Workflow: MIDI Learn

### Voraussetzung
- MIDI-Device ist verbunden (Status: 🟢)
- Ein **MIDI Trigger Node** existiert im Module Canvas

### Schritt 1: Module Canvas öffnen

In der **linken Sidebar** → Modul auswählen → Canvas wird angezeigt

### Schritt 2: MIDI Trigger Node erstellen

**Toolbar:** `⚡ Trigger` → `🎹 MIDI`

Ein neuer Node erscheint mit:
- Channel: 1 (Slider 1-16)
- Note: 0 (Slider 0-127)
- Device: Dropdown (falls mehrere)

### Schritt 3: Node auswählen

Klick auf den **MIDI Trigger Node** → erscheint im **Node Control Panel** (rechts)

### Schritt 4: MIDI Learn aktivieren

Im Node Control Panel gibt es einen Button:
- **"🎯 MIDI Learn"** (normal)
- **"⏳ Waiting for MIDI..."** (aktiv)

**Klicken** → Button wechselt zu "Waiting..."

### Schritt 5: MIDI-Control bewegen

Drehe einen **Knob** oder drücke eine **Taste** am MIDI-Controller.

- Die erkannte **Note** oder **CC** wird automatisch eingetragen
- Der Learn-Modus wird beendet
- Channel und Note werden im Node aktualisiert

**Log-Ausgabe:**
```
INFO  MIDI Learn: Part ... assigned to CC 7 on channel 0
INFO  Applied MIDI Learn: Channel=0, CC=7
```

### Schritt 6: Testen

Bewege den gelernten Control → Der Trigger Node sollte reagieren.

---

## User Workflow: Controller Overlay

### Was ist der Controller Overlay?

Ein **visuelles Fenster**, das den MIDI-Controller (z.B. Ecler NUO 4) darstellt mit:
- Knobs die sich drehen
- Fader die sich bewegen
- Buttons die leuchten

### Aktueller Status

Das Controller Overlay zeigt:
- Mixer-Foto als Hintergrund (z.B. Ecler NUO 4)
- Asset-Bilder für Knobs, Fader und Buttons

### Schritt 1: Overlay öffnen

Das Overlay kann über das Menü **View** → **MIDI Controller Overlay** (oder das entsprechende Icon in der Menüleiste) ein- und ausgeblendet werden.

---

## Bekannte Probleme & Einschränkungen

### 🟡 Medium

| Problem | Geplante Lösung |
|---------|--------|
| MIDI Learn nur für Trigger-Nodes | Erweiterung auf alle Parameter in zukünftiger Version |

### 🟢 Low Priority

| Problem | Geplante Lösung |
|---------|--------|
| MIDI-zu-Layer/Effect direkt | ControlTarget-Routing implementieren |
| Multi-Device Support | Device-ID in Mapping speichern |

---

## Technische Details

### Dateien

| Datei | Zweck |
|-------|-------|
| `mapmap-control/src/midi/mod.rs` | MIDI-Modul Root, MidiMessage enum |
| `mapmap-control/src/midi/input.rs` | MidiInputHandler (Connect, Poll) |
| `mapmap-control/src/midi/mapping.rs` | MidiMapping, MidiMappingKey |
| `mapmap-control/src/midi/midi_learn.rs` | MidiLearnManager, MidiLearnState |
| `mapmap-control/src/midi/ecler_nuo4.rs` | 89 vordefinierte Mappings |
| `mapmap-ui/src/controller_overlay_panel.rs` | Overlay UI |
| `mapmap-ui/src/module_canvas.rs` | MIDI Learn für Nodes |
| `mapmap/src/main.rs` Zeile 451-460 | MIDI Message Routing |
| `resources/controllers/ecler_nuo4/elements.json` | Element-Positionen/MIDI-Config |

### Feature Flags

```toml
# In crates/mapmap/Cargo.toml
[features]
default = ["audio", "midi"]  # MIDI ist standardmäßig aktiviert
midi = ["mapmap-control/midi", "mapmap-ui/midi"]
```

### Structs/Enums

```rust
// MidiMessage - Eingegangene MIDI-Nachricht
pub enum MidiMessage {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    PitchBend { channel: u8, value: u16 },
    Clock, Start, Stop, Continue,
}

// MidiMappingKey - Eindeutiger Schlüssel für Mapping
pub enum MidiMappingKey {
    Note(u8, u8),      // channel, note
    Control(u8, u8),   // channel, controller
    PitchBend(u8),     // channel
}

// MidiInputHandler - Hauptklasse für MIDI-Eingabe
pub struct MidiInputHandler {
    connection: Option<MidiInputConnection<()>>,
    message_sender: Sender<MidiMessage>,
    message_receiver: Arc<Mutex<Receiver<MidiMessage>>>,
    mapping: Arc<Mutex<MidiMapping>>,
}
```

---
