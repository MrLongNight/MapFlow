# Machbarkeitsstudie: Multi-PC Architektur

## 1. Executive Summary

Die Erweiterung von MapFlow für den Multi-PC-Betrieb ist technisch **machbar** und positioniert die Software als professionelle Alternative zu teuren Lösungen wie Resolume Arena oder MadMapper.

Diese Studie analysiert **vier Architektur-Optionen** für verschiedene Hardware-Anforderungen:

| Option | Name | Zielgruppe | Hardware-Anforderung |
|--------|------|------------|----------------------|
| **A** | NDI Streaming | Standard-Installationen | Modern (Gigabit LAN) |
| **B** | Distributed Rendering | High-End Multi-GPU | Leistungsstark |
| **C** | Legacy Slave Client | Sehr alte Hardware | Minimal |
| **D** | Raspberry Pi Player | Embedded/Budget | Optional |

**Empfehlung:** Ein **Single-Binary-Ansatz** mit integrierten Modulen für alle Optionen. Die Auswahl erfolgt über Startparameter oder automatische Hardware-Erkennung.

---

## 2. Architektur-Übersicht

### 2.1 High-Level Systemarchitektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MAPFLOW MASTER                                  │
│                          (Haupt-Rendering-PC)                               │
│                                                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────────────┐ │
│  │   Editor   │  │  Renderer  │  │   Media    │  │      Output Router     │ │
│  │    GUI     │  │   Engine   │  │  Pipeline  │  │                        │ │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘  │  ┌──────┐ ┌─────────┐  │ │
│        │               │               │          │  │ NDI  │ │ Control │  │ │
│        └───────────────┴───────────────┴──────────┼─▶│Sender│ │  OSC    │  │ │
│                                                   │  └──┬───┘ └────┬────┘  │ │
│                                                   └─────┼──────────┼───────┘ │
└─────────────────────────────────────────────────────────┼──────────┼─────────┘
                                                          │          │
                           ┌──────────────────────────────┼──────────┼─────────┐
                           │           NETZWERK (Gigabit LAN)                  │
                           └──────────────────────────────┼──────────┼─────────┘
                                    │         │           │          │
       ┌────────────────────────────┼─────────┼───────────┼──────────┼─────────┐
       │                            │         │           │          │         │
       ▼                            ▼         ▼           ▼          ▼         ▼
┌─────────────┐            ┌─────────────┐  ┌─────────────┐   ┌─────────────────┐
│  Option A   │            │  Option B   │  │  Option C   │   │    Option D     │
│ NDI Player  │            │ Distributed │  │   Legacy    │   │  Raspberry Pi   │
│  (Modern)   │            │   Render    │  │   Client    │   │    (Optional)   │
│             │            │   Client    │  │   (H.264)   │   │                 │
│ ┌─────────┐ │            │ ┌─────────┐ │  │ ┌─────────┐ │   │ ┌─────────────┐ │
│ │NDI Recv │ │            │ │  Own    │ │  │ │  RTSP   │ │   │ │Dicaffeine / │ │
│ │ wgpu    │ │            │ │ wgpu    │ │  │ │ Player  │ │   │ │Custom Player│ │
│ └─────────┘ │            │ │ Render  │ │  │ └─────────┘ │   │ └─────────────┘ │
│  1080p60+   │            │ └─────────┘ │  │  720p30     │   │   720p60        │
└─────────────┘            └─────────────┘  └─────────────┘   └─────────────────┘
```

### 2.2 Single-Binary-Konzept

Anstatt separate Anwendungen zu entwickeln, nutzt MapFlow **eine einzige ausführbare Datei** mit verschiedenen Betriebsmodi:

```rust
// main.rs - Mode Selection
fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Editor => run_editor(),           // Volles GUI + Rendering
        Mode::PlayerNdi => run_player_ndi(),    // Option A: NDI Receiver
        Mode::PlayerDist => run_player_dist(),  // Option B: Distributed
        Mode::PlayerLegacy => run_player_legacy(), // Option C: RTSP/H.264
        Mode::PlayerPi => run_player_pi(),      // Option D: Raspberry Pi
    }
}
```

**Vorteile:**
- Eine Codebasis = einfachere Wartung
- Einheitlicher Installer
- Automatische Feature-Discovery
- Reduzierte Testmatrix

---

## 3. Option A: NDI Video-Streaming (Empfohlen)

### 3.1 Konzept

Der Master-PC berechnet das **gesamte Rendering** (Mapping, Effekte, Compositing) und sendet das fertige Videosignal über das Netzwerk an die Clients.

```
┌─────────────────────┐         NDI Stream          ┌─────────────────────┐
│       MASTER        │ ──────────────────────────▶ │      PLAYER A       │
│                     │         (LAN)               │                     │
│  ┌───────────────┐  │                             │  ┌───────────────┐  │
│  │ wgpu Renderer │  │                             │  │ NDI Receiver  │  │
│  │     ↓         │  │                             │  │     ↓         │  │
│  │ NDI Sender    │  │                             │  │ Fullscreen    │  │
│  └───────────────┘  │                             │  │ Texture       │  │
│                     │                             │  └───────────────┘  │
└─────────────────────┘                             └─────────────────────┘
```

### 3.2 Technologie-Stack

#### 3.2.1 NDI (Network Device Interface)

| Komponente | Details |
|------------|---------|
| **SDK** | NDI 6 SDK (proprietär, kostenlos für Integration) |
| **Rust Bindings** | `grafton-ndi` - Zero-Copy, idiomatic Rust |
| **Latenz** | Full-Bandwidth: <100ms, NDI\|HX: ~145ms |
| **Bandbreite** | ~250 Mbps für 1080p60 (unkomprimiert) |

**Lizenz-Hinweis:**
Das NDI SDK ist proprietär. Die "NDI Runtime" muss vom Benutzer separat installiert werden (ähnlich wie bei OBS/vMix). MapFlow bindet nur die Library zur Laufzeit.

#### 3.2.2 GStreamer Alternative (Open Source)

Falls NDI-Lizenzierung ein Problem darstellt:

| Protokoll | Latenz | Bandbreite | Kompatibilität |
|-----------|--------|------------|----------------|
| **RTP/H.264** | ~50-150ms | ~5-20 Mbps | Universal |
| **SRT** | ~100-500ms | ~5-20 Mbps | Broadcast-Standard |
| **RTSP** | ~200-500ms | ~5-20 Mbps | Legacy-freundlich |

### 3.3 Implementierung

#### 3.3.1 Master-Seite (NDI Sender)

```rust
// crates/mapmap-ndi/src/sender.rs
pub struct NdiSender {
    ndi_instance: NdiInstance,
    video_sender: NdiVideoSender,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
}

impl NdiSender {
    pub fn new(name: &str) -> Result<Self, NdiError> {
        let ndi_instance = NdiInstance::new()?;
        let video_sender = ndi_instance.create_video_sender(name)?;

        Ok(Self {
            ndi_instance,
            video_sender,
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::new())),
        })
    }

    /// Zero-Copy Texture-to-NDI Transfer
    pub fn send_frame(&self, texture: &wgpu::Texture, device: &wgpu::Device) {
        // GPU Readback via staging buffer
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            size: texture.width() * texture.height() * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            ..Default::default()
        });

        // Copy texture to staging buffer
        encoder.copy_texture_to_buffer(texture, &staging);

        // Map and send to NDI (async)
        staging.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            if result.is_ok() {
                let data = staging.slice(..).get_mapped_range();
                self.video_sender.send_video_async(&data, callback);
            }
        });
    }
}
```

#### 3.3.2 Player-Seite (NDI Receiver)

```rust
// crates/mapmap-ndi/src/receiver.rs
pub struct NdiPlayerApp {
    ndi_receiver: NdiReceiver,
    texture_handle: Option<wgpu::Texture>,
    fullscreen_renderer: FullscreenRenderer,
}

impl NdiPlayerApp {
    pub fn run(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Receive NDI frame
        if let Some(video_frame) = self.ndi_receiver.receive_video(timeout) {
            // Upload to GPU texture
            queue.write_texture(
                self.texture_handle.as_ref().unwrap(),
                video_frame.data(),
                wgpu::ImageDataLayout { ... },
                texture_size,
            );
        }

        // Render fullscreen quad
        self.fullscreen_renderer.render(&self.texture_handle);
    }
}
```

### 3.4 Hardware-Anforderungen

| Rolle | CPU | RAM | GPU | Netzwerk |
|-------|-----|-----|-----|----------|
| **Master** | 8+ Cores | 16+ GB | RTX 3060+ | Gigabit |
| **Player** | 4+ Cores | 8+ GB | Intel HD 4000+ | Gigabit |

### 3.5 Performance-Ziele

| Auflösung | Framerate | Latenz (Ziel) | Bandbreite |
|-----------|-----------|---------------|------------|
| 1920x1080 | 60 fps | <50ms | ~250 Mbps |
| 3840x2160 | 30 fps | <80ms | ~500 Mbps |
| 3840x2160 | 60 fps | <100ms | ~1000 Mbps |

---

## 4. Option B: Distributed Rendering

### 4.1 Konzept

Der Master sendet nur **Steuerbefehle und Szenen-Updates** über das Netzwerk. Jeder Client rendert seine Ausgabe **selbstständig**.

```
┌─────────────────────┐                           ┌─────────────────────┐
│       MASTER        │    OSC/Control Commands   │      CLIENT A       │
│                     │ ─────────────────────────▶│                     │
│  ┌───────────────┐  │                           │  ┌───────────────┐  │
│  │ Scene Manager │  │    {"layer": 1,           │  │ Local wgpu    │  │
│  │ Control Hub   │  │     "media": "vid.mp4",   │  │ Renderer      │  │
│  │               │  │     "opacity": 0.8}       │  │               │  │
│  └───────────────┘  │                           │  │ Lokale Assets │  │
│                     │                           │  └───────────────┘  │
└─────────────────────┘                           └─────────────────────┘
                                                           │
                                                           ▼
                                                       [Projektor]
```

### 4.2 Vorteile

- **Minimale Netzwerkbandbreite** (~100 Kbps für Steuerdaten)
- **Unabhängige Auflösungen** pro Client
- **Höhere Gesamtauflösung** möglich (z.B. 16K verteilt)
- **Redundanz** bei Netzwerkausfällen (Clients rendern weiter)

### 4.3 Nachteile

- **Komplexe Synchronisation** (Genlock/Frame-Sync erforderlich)
- **Asset-Distribution** (Videos müssen auf alle Clients kopiert werden)
- **Höhere Hardware-Kosten** (jeder Client braucht GPU)
- **Erhöhte Latenz-Variabilität**

### 4.4 Synchronisations-Strategie

#### 4.4.1 Time-Code basiert (Empfohlen)

```rust
// crates/mapmap-sync/src/timecode.rs
pub struct TimecodeSync {
    master_clock: Arc<AtomicU64>,  // Nanosekunden seit Start
    ntp_offset: i64,               // NTP-Korrektur
    frame_duration: Duration,
}

impl TimecodeSync {
    /// Master: Broadcast aktuellen Timecode
    pub fn broadcast_timecode(&self, osc_sender: &OscSender) {
        let tc = self.master_clock.load(Ordering::SeqCst);
        osc_sender.send("/mapflow/timecode", tc);
    }

    /// Client: Berechne Frame-Nummer aus Timecode
    pub fn current_frame(&self) -> u64 {
        let adjusted = self.master_clock.load(Ordering::SeqCst) + self.ntp_offset as u64;
        adjusted / self.frame_duration.as_nanos() as u64
    }
}
```

#### 4.4.2 Hardware Genlock (Professionell)

Für frame-perfekte Synchronisation:

| Hardware | Kosten | Präzision |
|----------|--------|-----------|
| Blackmagic DeckLink | ~200-800€ | <1 Frame |
| AJA Corvid | ~500-1500€ | <1 Frame |
| SDI-Wordclock | ~100€ | ~1 Frame |

### 4.5 Asset-Distribution

```yaml
# mapflow-project.yaml
assets:
  distribution: hybrid  # local, network, hybrid

  network_sources:
    - type: nfs
      path: "//nas/mapflow/assets"

    - type: s3
      bucket: "mapflow-assets"
      region: "eu-central-1"

  local_cache:
    path: "/tmp/mapflow-cache"
    max_size: "50GB"

  sync_strategy:
    on_project_load: prefetch_all
    on_media_change: stream_on_demand
```

### 4.6 Hardware-Anforderungen

| Rolle | CPU | RAM | GPU | Netzwerk |
|-------|-----|-----|-----|----------|
| **Master** | 4+ Cores | 8+ GB | Keine/Beliebig | 100 Mbps |
| **Client** | 8+ Cores | 16+ GB | RTX 3060+ | Gigabit + Storage |

---

## 5. Option C: Legacy Slave Client (Sehr alte Hardware)

### 5.1 Motivation

Viele VJ-Setups nutzen **ältere Hardware** als Ausgabe-Clients:

- Alte Laptops (2010-2015 Ära)
- Mini-PCs ohne dedizierte GPU
- Thin Clients
- Embedded Industrial PCs

### 5.2 Konzept

Der Legacy-Client nutzt **Hardware-dekodiertes H.264** anstatt NDI, um die CPU-Last zu minimieren.

```
┌─────────────────────┐                           ┌─────────────────────┐
│       MASTER        │                           │   LEGACY CLIENT     │
│                     │                           │                     │
│  ┌───────────────┐  │     H.264 / RTSP          │  ┌───────────────┐  │
│  │ wgpu Renderer │  │ ─────────────────────────▶│  │ HW Decoder    │  │
│  │     ↓         │  │     (5-15 Mbps)           │  │ (VA-API/DXVA) │  │
│  │ H.264 Encoder │  │                           │  │     ↓         │  │
│  │ (Software/HW) │  │                           │  │ Fullscreen    │  │
│  └───────────────┘  │                           │  │ (keine wgpu)  │  │
│                     │                           │  └───────────────┘  │
└─────────────────────┘                           └─────────────────────┘
```

### 5.3 Technologie-Stack

#### 5.3.1 Streaming-Protokoll

| Protokoll | Latenz | CPU-Last | Empfehlung |
|-----------|--------|----------|------------|
| **RTSP/H.264** | 200-500ms | Sehr gering | ✅ Empfohlen |
| **HLS** | 2-10s | Minimal | Nur für Backup |
| **RTMP** | 1-3s | Gering | Flash-Legacy |

#### 5.3.2 Encoder (Master-Seite)

```rust
// crates/mapmap-legacy/src/encoder.rs
pub struct LegacyStreamEncoder {
    encoder: x264::Encoder,      // Software-Encoder (fallback)
    hw_encoder: Option<NvEnc>,   // Hardware-Encoder (wenn verfügbar)
    rtsp_server: RtspServer,
}

impl LegacyStreamEncoder {
    pub fn encode_and_stream(&mut self, frame: &RawFrame) {
        // Wähle besten Encoder
        let encoded = if let Some(hw) = &mut self.hw_encoder {
            hw.encode(frame)  // NvEnc/QSV/VCE
        } else {
            self.encoder.encode(frame)  // x264 Software
        };

        // Stream über RTSP
        self.rtsp_server.push_frame(encoded);
    }
}
```

#### 5.3.3 Player (Client-Seite)

Für maximale Kompatibilität wird der Legacy-Player als **separates Modul** bereitgestellt:

```rust
// crates/mapmap-legacy/src/player.rs
pub struct LegacyPlayer {
    ffmpeg_decoder: FfmpegDecoder,  // Hardware-beschleunigt
    display: SdlDisplay,            // SDL2 für breite Kompatibilität
}

impl LegacyPlayer {
    pub fn new(rtsp_url: &str) -> Result<Self> {
        let decoder = FfmpegDecoder::new(rtsp_url)?
            .with_hw_acceleration(true)  // VA-API, DXVA, VideoToolbox
            .with_low_latency(true);

        let display = SdlDisplay::new()?
            .fullscreen(true);

        Ok(Self { decoder, display })
    }

    pub fn run(&mut self) {
        loop {
            if let Some(frame) = self.decoder.next_frame() {
                self.display.present(&frame);
            }
        }
    }
}
```

### 5.4 Mindest-Hardware-Anforderungen

| Komponente | Minimum | Empfohlen |
|------------|---------|-----------|
| **CPU** | Dual-Core 1.6 GHz | Quad-Core 2.0 GHz |
| **RAM** | 2 GB | 4 GB |
| **GPU** | Intel HD 2000 (Sandy Bridge) | Intel HD 4000+ |
| **OS** | Windows 7 SP1 / Ubuntu 18.04 | Windows 10 / Ubuntu 22.04 |
| **Netzwerk** | 100 Mbps | Gigabit |

### 5.5 Unterstützte Hardware-Decoder

| Platform | API | Unterstützte Codecs |
|----------|-----|---------------------|
| **Windows** | DXVA2 / D3D11VA | H.264, H.265 |
| **Linux** | VA-API | H.264, H.265, VP9 |
| **macOS** | VideoToolbox | H.264, H.265, ProRes |
| **Intel** | QSV | H.264, H.265, AV1 |
| **AMD** | AMF | H.264, H.265 |
| **NVIDIA** | NVDEC | H.264, H.265, AV1 |

### 5.6 Performance-Ziele

| Auflösung | Framerate | Bitrate | CPU-Last (typisch) |
|-----------|-----------|---------|-------------------|
| 1280x720 | 30 fps | 5 Mbps | 5-15% |
| 1920x1080 | 30 fps | 10 Mbps | 10-25% |
| 1920x1080 | 60 fps | 15 Mbps | 15-35% |

---

## 6. Option D: Raspberry Pi Player (Optional)

### 6.1 Motivation

Der Raspberry Pi ist eine **kostengünstige** Lösung für kleinere Installationen:

- Preis: ~50-100€ pro Einheit
- Geringer Stromverbrauch (~5-15W)
- Kompakte Bauform
- Zuverlässig für Dauerbetrieb

### 6.2 Unterstützte Modelle

| Modell | Status | Max. Auflösung | Empfehlung |
|--------|--------|----------------|------------|
| **Raspberry Pi 5** | ✅ Empfohlen | 1080p60 / 4K30 | Best Performance |
| **Raspberry Pi 4** | ✅ Unterstützt | 720p60 / 1080p30 | Budget-Option |
| **Raspberry Pi 3B+** | ⚠️ Eingeschränkt | 720p30 | Nur für Notfälle |
| **CM4** | ✅ Empfohlen | 1080p60 / 4K30 | Industrial Use |

### 6.3 Software-Optionen

#### 6.3.1 Option D1: Dicaffeine (Empfohlen für NDI)

[Dicaffeine](https://dicaffeine.com/) ist ein professionneller NDI-Player für Raspberry Pi:

```bash
# Installation auf Raspberry Pi OS (64-bit)
wget https://dicaffeine.com/releases/dicaffeine_latest_arm64.deb
sudo dpkg -i dicaffeine_latest_arm64.deb

# Start als NDI-Receiver
dicaffeine --source "MAPFLOW-MASTER" --fullscreen
```

**Performance (Raspberry Pi 4):**
- 720p60: ✅ Stabil
- 1080p30: ✅ Stabil  
- 1080p60: ⚠️ Möglich mit Drops

**Performance (Raspberry Pi 5):**
- 720p60: ✅ Stabil
- 1080p30: ✅ Stabil
- 1080p60: ✅ Stabil (erwartet)
- 4K30: ⚠️ Experimentell

#### 6.3.2 Option D2: Custom MapFlow Player (Portierung)

Für vollständige Integration kann MapFlow für ARM64 kompiliert werden:

```rust
// Compile Target: aarch64-unknown-linux-gnu
// crates/mapmap-pi/src/player.rs

pub struct PiPlayer {
    // Nutze wgpu mit Vulkan (Pi 5) oder OpenGL ES (Pi 4)
    backend: WgpuBackend,
    ndi_receiver: NdiReceiver,
}

impl PiPlayer {
    pub fn new() -> Result<Self> {
        // Automatische Backend-Erkennung
        let backend = WgpuBackend::new_with_preference(&[
            wgpu::Backends::VULKAN,  // Pi 5
            wgpu::Backends::GL,      // Pi 4 Fallback
        ])?;

        Ok(Self { backend, ndi_receiver })
    }
}
```

**Cross-Compilation:**

```bash
# Voraussetzungen
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu

# Build
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
cargo build --target aarch64-unknown-linux-gnu --release -p mapmap-pi
```

#### 6.3.3 Option D3: RTSP/H.264 mit VLC (Fallback)

Für maximale Kompatibilität ohne Custom-Software:

```bash
# Auf dem Raspberry Pi
cvlc rtsp://master-ip:8554/stream \
    --fullscreen \
    --no-osd \
    --avcodec-hw=any \
    --network-caching=100
```

### 6.4 Hardware-Setup

#### 6.4.1 Bill of Materials (Pi 5)

| Komponente | Empfehlung | Preis (ca.) |
|------------|------------|-------------|
| Raspberry Pi 5 (8GB) | Official | ~90€ |
| Active Cooler | Official | ~10€ |
| Power Supply (27W) | Official | ~15€ |
| microSD (32GB A2) | SanDisk Extreme | ~15€ |
| HDMI-Kabel | 4K-fähig | ~10€ |
| Case | Aluminium passiv | ~20€ |
| **Gesamt** | | **~160€** |

#### 6.4.2 Software-Konfiguration

```bash
# /boot/config.txt Optimierungen
# GPU Memory erhöhen
gpu_mem=256

# HDMI Force Hotplug (für Projektoren)
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=82  # 1080p60

# Overscan deaktivieren
disable_overscan=1

# Audio über HDMI
hdmi_drive=2
```

```bash
# /etc/systemd/system/mapflow-player.service
[Unit]
Description=MapFlow Pi Player
After=network.target graphical.target

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStart=/usr/local/bin/mapflow --player-pi --source MAPFLOW-MASTER
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### 6.5 Netzwerk-Anforderungen

| Stream-Typ | Bandwidth | Empfohlenes Setup |
|------------|-----------|-------------------|
| NDI (720p60) | ~100 Mbps | Gigabit Ethernet |
| NDI (1080p60) | ~250 Mbps | Gigabit Ethernet |
| RTSP/H.264 | ~5-15 Mbps | 100 Mbps ausreichend |

**Wichtig:** WLAN ist **nicht empfohlen** für professionelle Installationen aufgrund von Latenz-Schwankungen.

---

## 7. Installer & Deployment

### 7.1 Windows Installer (WiX)

Der Installer bietet dem Benutzer verschiedene Installationsprofile:

```xml
<!-- installer/wix/main.wxs -->
<Feature Id="Complete" Title="MapFlow Complete" Level="1">
    <Feature Id="Core" Title="Core Components" Level="1">
        <ComponentRef Id="MapFlowExecutable" />
        <ComponentRef Id="RuntimeDependencies" />
    </Feature>

    <Feature Id="Editor" Title="MapFlow Editor" Level="1">
        <ComponentRef Id="EditorShortcut" />
    </Feature>

    <Feature Id="PlayerNdi" Title="MapFlow Player (NDI)" Level="1">
        <ComponentRef Id="PlayerNdiShortcut" />
    </Feature>

    <Feature Id="PlayerLegacy" Title="MapFlow Player (Legacy)" Level="2">
        <ComponentRef Id="PlayerLegacyShortcut" />
    </Feature>
</Feature>

<!-- Shortcuts -->
<Shortcut Id="EditorShortcut" Name="MapFlow"
          Target="[INSTALLDIR]MapFlow.exe" />

<Shortcut Id="PlayerNdiShortcut" Name="MapFlow Player (NDI)"
          Target="[INSTALLDIR]MapFlow.exe" Arguments="--player-ndi" />

<Shortcut Id="PlayerLegacyShortcut" Name="MapFlow Player (Legacy)"
          Target="[INSTALLDIR]MapFlow.exe" Arguments="--player-legacy" />
```

### 7.2 Linux Packages

#### 7.2.1 Debian/Ubuntu (.deb)

```bash
# Paket-Struktur
mapflow_1.0.0_amd64.deb
├── DEBIAN/
│   ├── control
│   ├── postinst
│   └── prerm
├── usr/
│   ├── bin/mapflow
│   └── share/
│       ├── applications/
│       │   ├── mapflow.desktop
│       │   ├── mapflow-player-ndi.desktop
│       │   └── mapflow-player-legacy.desktop
│       └── icons/...
```

#### 7.2.2 Raspberry Pi Image (Optional)

Für einfachste Installation kann ein vorkonfiguriertes Image bereitgestellt werden:

```
mapflow-pi-player-v1.0.0.img.xz
├── Raspberry Pi OS Lite (64-bit)
├── MapFlow Player vorinstalliert
├── Auto-Start konfiguriert
└── Read-Only Filesystem (optional)
```

### 7.3 Deployment-Übersicht

| Zielplattform | Paketformat | Installation |
|---------------|-------------|--------------|
| Windows 10/11 | MSI | Doppelklick |
| Ubuntu 22.04+ | DEB | `dpkg -i` |
| Fedora 38+ | RPM | `dnf install` |
| Arch Linux | AUR | `yay -S mapflow` |
| Raspberry Pi | DEB / Image | `dpkg -i` oder Flash |
| macOS | DMG | Drag & Drop |

---

## 8. Aufwandsschätzung

### 8.1 Option A: NDI Streaming (Empfohlen)

| Phase | Aufgabe | Dauer |
|-------|---------|-------|
| 1 | Architektur-Refactoring (main.rs Split) | 2-3 Tage |
| 2 | NDI Integration (grafton-ndi) | 5-7 Tage |
| 3 | NDI Sender (wgpu Texture → NDI) | 3-4 Tage |
| 4 | NDI Receiver (Player-App) | 3-4 Tage |
| 5 | Installer-Anpassung (WiX/DEB) | 1-2 Tage |
| 6 | Testing & Latenz-Optimierung | 3-5 Tage |
| | **Gesamt Option A** | **~3 Wochen** |

### 8.2 Option B: Distributed Rendering

| Phase | Aufgabe | Dauer |
|-------|---------|-------|
| 1 | OSC Control Protocol Design | 2-3 Tage |
| 2 | Timecode-Synchronisation | 5-7 Tage |
| 3 | Asset-Distribution System | 4-5 Tage |
| 4 | Remote Render Client | 5-7 Tage |
| 5 | Genlock-Integration (Optional) | 3-5 Tage |
| 6 | Multi-Client Testing | 5-7 Tage |
| | **Gesamt Option B** | **~5-6 Wochen** |

### 8.3 Option C: Legacy Slave Client

| Phase | Aufgabe | Dauer |
|-------|---------|-------|
| 1 | H.264 Encoder Integration (x264/NvEnc) | 3-4 Tage |
| 2 | RTSP Server Implementation | 2-3 Tage |
| 3 | Legacy Player (SDL2 + FFmpeg HW) | 3-4 Tage |
| 4 | Hardware-Decoder Testing | 2-3 Tage |
| | **Gesamt Option C** | **~2 Wochen** |

### 8.4 Option D: Raspberry Pi Player

| Phase | Aufgabe | Dauer |
|-------|---------|-------|
| 1 | ARM64 Cross-Compilation Setup | 1-2 Tage |
| 2 | Dicaffeine Integration/Testing | 1-2 Tage |
| 3 | Custom Pi Player (Optional) | 3-5 Tage |
| 4 | Pi OS Image Creation | 1-2 Tage |
| 5 | Documentation | 1 Tag |
| | **Gesamt Option D** | **~1-2 Wochen** |

### 8.5 Gesamtübersicht

| Variante | Aufwand | Priorität |
|----------|---------|-----------|
| Nur Option A | 3 Wochen | ⭐⭐⭐ Hoch |
| Option A + C | 5 Wochen | ⭐⭐⭐ Hoch |
| Option A + C + D | 6-7 Wochen | ⭐⭐ Mittel |
| Alle Optionen (A+B+C+D) | 10-12 Wochen | ⭐ Perspektivisch |

---

## 9. Empfohlener Implementierungsplan

### Phase 1: MVP (Option A - NDI)
**Zeitraum:** Wochen 1-3

1. Refactoring der Hauptanwendung für Multi-Mode-Unterstützung
2. Integration von `grafton-ndi` für Sender/Receiver
3. Implementierung des NDI-Players als separater Modus
4. Basis-Installer-Anpassung

**Deliverable:** Funktionierender NDI-Stream von Master zu Player über LAN.

### Phase 2: Legacy Support (Option C)
**Zeitraum:** Wochen 4-5

1. H.264 Encoding Pipeline
2. RTSP Server
3. Legacy Player mit Hardware-Decoding
4. Kompatibilitätstest mit alter Hardware

**Deliverable:** RTSP-basierter Player für Hardware ab 2010.

### Phase 3: Raspberry Pi (Option D)
**Zeitraum:** Wochen 6-7

1. ARM64 Build-Pipeline
2. Dicaffeine-Dokumentation
3. Optional: Nativer Pi-Player
4. Pi OS Image für einfache Installation

**Deliverable:** Funktionierender Raspberry Pi 4/5 als Ausgabegerät.

### Phase 4: Distributed Rendering (Option B)
**Zeitraum:** Wochen 8-12 (Future)

1. OSC Control Protocol
2. Timecode-Synchronisation
3. Asset-Distribution
4. Multi-GPU Cluster Support

**Deliverable:** Verteiltes Rendering über mehrere leistungsstarke PCs.

---

## 10. Risiken & Mitigationen

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|-------------------|--------|------------|
| NDI SDK Lizenzänderung | Gering | Hoch | GStreamer als Fallback bereithalten |
| Latenz >100ms | Mittel | Mittel | Hardware-Encoding, Netzwerk-Tuning |
| Raspberry Pi Instabilität | Mittel | Gering | Dicaffeine als Fallback, Watchdog |
| Komplexität Distributed Sync | Hoch | Hoch | Option B als letztes implementieren |
| Cross-Platform Bugs | Mittel | Mittel | CI/CD mit Multi-Platform Testing |

---

## 11. Fazit

Die Multi-PC-Architektur für MapFlow ist **technisch machbar** und wirtschaftlich sinnvoll. Der empfohlene Ansatz ist:

1. **Single-Binary** mit verschiedenen Modi (professioneller, einfache Wartung)
2. **NDI als primäres Protokoll** (Industriestandard, geringe Latenz)
3. **Legacy-Support über H.264/RTSP** (maximale Kompatibilität)
4. **Raspberry Pi als optionale Budget-Lösung**
5. **Distributed Rendering für High-End** (langfristige Perspektive)

Mit diesem Ansatz positioniert sich MapFlow als **ernstzunehmende Alternative** zu kommerziellen Lösungen wie:
- Resolume Arena (~€799)
- MadMapper (~€449)
- TouchDesigner (~$2000/Jahr)

**Empfohlener erster Schritt:**
Erstellung eines Proof-of-Concept für Option A, der einen NDI-Stream von MapFlow zu einem zweiten PC überträgt und dort fullscreen darstellt. Geschätzte Zeit: 5-7 Tage.
