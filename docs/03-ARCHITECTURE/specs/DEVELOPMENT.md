# HueFlow Development Guide

## 🎨 Effect Development

### Effect Architecture

```
AudioSpectrum ──┐
                ├──→ LightEffect::update() ──→ HashMap<channel_id, RGB>
LightNode[] ────┘
```

### Step-by-Step: New Effect

1. Create file in `hue_flow_core/src/effects/`
2. Implement `LightEffect` trait
3. Add to `mod.rs` exports
4. Test with CLI: `cargo run -- run --effect your_effect`

---

## 🔊 Audio Analysis

### FFT Band Mapping (for MapFlow integration)

```rust
// Recommended frequency bands
const BASS_RANGE: Range<f32> = 20.0..200.0;
const LOW_MID_RANGE: Range<f32> = 200.0..500.0;
const MID_RANGE: Range<f32> = 500.0..2000.0;
const HIGH_MID_RANGE: Range<f32> = 2000.0..6000.0;
const HIGHS_RANGE: Range<f32> = 6000.0..20000.0;

// 9-band equalizer style
const BANDS_9: [Range<f32>; 9] = [
    31.0..62.0,     // Sub-bass
    62.0..125.0,    // Bass
    125.0..250.0,   // Low-mids
    250.0..500.0,   // Mids
    500.0..1000.0,  // Upper-mids
    1000.0..2000.0, // Presence
    2000.0..4000.0, // Brilliance
    4000.0..8000.0, // Air
    8000.0..16000.0, // Ultra-highs
];
```

### Smoothing & Attack/Decay

```rust
pub struct SmoothedValue {
    current: f32,
    attack: f32,  // 0.0-1.0, higher = faster rise
    decay: f32,   // 0.0-1.0, higher = faster fall
}

impl SmoothedValue {
    pub fn update(&mut self, target: f32) {
        if target > self.current {
            self.current += (target - self.current) * self.attack;
        } else {
            self.current += (target - self.current) * self.decay;
        }
    }
}

// Typical values:
// Fast/punchy: attack=0.8, decay=0.3
// Smooth/ambient: attack=0.1, decay=0.05
// BPM sync: attack=0.9, decay=0.1
```

---

## 🌈 Color Utilities

### HSV to RGB

```rust
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = match (h / 60.0) as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
```

### Color Temperature

```rust
// Kelvin to RGB approximation
pub fn kelvin_to_rgb(kelvin: u32) -> (u8, u8, u8) {
    let temp = kelvin as f32 / 100.0;
    
    let r = if temp <= 66.0 { 255.0 } 
            else { 329.698727446 * (temp - 60.0).powf(-0.1332047592) };
    
    let g = if temp <= 66.0 { 99.4708025861 * temp.ln() - 161.1195681661 }
            else { 288.1221695283 * (temp - 60.0).powf(-0.0755148492) };
    
    let b = if temp >= 66.0 { 255.0 }
            else if temp <= 19.0 { 0.0 }
            else { 138.5177312231 * (temp - 10.0).ln() - 305.0447927307 };
    
    (r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8)
}
```

### Gamma Correction

```rust
// Hue bulbs have non-linear brightness perception
pub fn gamma_correct(value: u8, gamma: f32) -> u8 {
    let normalized = value as f32 / 255.0;
    let corrected = normalized.powf(gamma);
    (corrected * 255.0) as u8
}

// Recommended gamma: 2.2 for natural perception
```

---

## 🎬 Effect Ideas

### Music Reactive

| Effect | Description | Parameters |
|--------|-------------|------------|
| **Bass Pulse** | All lights pulse on bass hits | threshold, attack, decay, color |
| **Multi-Band** | Different colors per frequency band | band_colors[], spatial_mapping |
| **Beat Flash** | Flash on detected beats | bpm, flash_duration, color |
| **VU Meter** | Brightness = volume level | min_brightness, max_brightness |
| **Waveform** | Lights follow audio waveform shape | speed, amplitude |

### Ambient

| Effect | Description | Parameters |
|--------|-------------|------------|
| **Color Cycle** | Slow hue rotation | speed, saturation |
| **Breathe** | Gentle pulse like breathing | period_ms, min/max_brightness |
| **Candle** | Flickering warm light | intensity, color_temp |
| **Gradient** | Static spatial gradient | start_color, end_color, axis |

### Gaming/Video

| Effect | Description | Parameters |
|--------|-------------|------------|
| **Explosion** | Bright flash with falloff | radius, duration, color |
| **Hit Indicator** | Flash from direction | direction, intensity, duration |
| **Health Bar** | Color based on value | value%, low_color, high_color |
| **Screen Capture** | Sample colors from screen | sample_regions, update_rate |

### Dynamic

| Effect | Description | Parameters |
|--------|-------------|------------|
| **Chaser** | Running light | speed, direction, tail_length |
| **Sparkle** | Random glitter | density, brightness, duration |
| **Wave** | Sine wave through lights | wavelength, speed, amplitude |
| **Strobe** | Flashing (use carefully!) | frequency_hz, duty_cycle |

---

## 🔧 Advanced Configuration

### Per-Channel Settings

```rust
pub struct ChannelConfig {
    pub enabled: bool,
    pub brightness_scale: f32,    // 0.0-2.0
    pub color_correction: (f32, f32, f32), // RGB multipliers
    pub gamma: f32,
    pub delay_ms: u32,            // For wave effects
    pub min_brightness: u8,       // Never go below
    pub max_brightness: u8,       // Never exceed
}
```

### Effect Mixing

```rust
pub struct EffectMixer {
    layers: Vec<(Box<dyn LightEffect>, f32)>, // effect, opacity
}

impl EffectMixer {
    pub fn render(&mut self, audio: &AudioSpectrum, nodes: &[LightNode]) -> HashMap<u8, (u8, u8, u8)> {
        let mut result: HashMap<u8, (f32, f32, f32)> = HashMap::new();
        
        for (effect, opacity) in &mut self.layers {
            let colors = effect.update(audio, nodes);
            for (id, (r, g, b)) in colors {
                let entry = result.entry(id).or_insert((0.0, 0.0, 0.0));
                entry.0 = entry.0 * (1.0 - opacity) + r as f32 * opacity;
                entry.1 = entry.1 * (1.0 - opacity) + g as f32 * opacity;
                entry.2 = entry.2 * (1.0 - opacity) + b as f32 * opacity;
            }
        }
        
        result.into_iter()
            .map(|(id, (r, g, b))| (id, (r as u8, g as u8, b as u8)))
            .collect()
    }
}
```

---

## 🧪 Testing

### Simulate Without Hardware

```rust
// Mock streamer for testing
pub struct MockStreamer {
    pub frames: Vec<Vec<u8>>,
}

impl MockStreamer {
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.frames.push(buf.to_vec());
        Ok(())
    }
    
    pub fn last_colors(&self) -> HashMap<u8, (u8, u8, u8)> {
        // Parse last frame...
    }
}
```

### Visual Debugging

```rust
// Print channel colors as colored blocks
fn debug_print_frame(colors: &HashMap<u8, (u8, u8, u8)>) {
    for i in 0..10 {
        if let Some((r, g, b)) = colors.get(&(i as u8)) {
            // ANSI color codes
            print!("\x1b[48;2;{};{};{}m  \x1b[0m", r, g, b);
        } else {
            print!("  ");
        }
    }
    println!();
}
```

---

## 📚 Further Reading

- [Hue EDK C++ Source](https://github.com/nicmcd/huesdk) - Reference implementation
- [Color Science](https://www.colour-science.org/) - CIE color space math
- [Audio Analysis](https://docs.rs/rustfft/latest/rustfft/) - FFT in Rust
- [Perceptual Brightness](https://alienryderflex.com/hsp.html) - HSP color model

---

## 📍 Spatial Coordinates & Live Preview

### Coordinate System
Hue Entertainment uses a normalized 3D coordinate system relative to the user's viewing position (e.g., TV or Monitor).

| Axis | Range | Meaning |
|------|-------|---------|
| **X** | -1.0 to 1.0 | **Left** (-1.0) to **Right** (1.0) |
| **Y** | -1.0 to 1.0 | **Back/Behind User** (-1.0) to **Front/Screen** (1.0) |
| **Z** | -1.0 to 1.0 | **Floor** (-1.0) to **Ceiling** (1.0) |

> **Note:** Position (0, 0, 0) is the center of the entertainment area (roughly where the user sits).

### 📺 Live Preview Implementation (Isometric 3D)

To visualize the lights in a 2D UI (like egui), we can project the 3D coordinates onto a 2D plane. An isometric projection is often best for overview.

```rust
pub struct RoomPreview {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl RoomPreview {
    /// Projects 3D Hue coordinates (x,y,z) to 2D UI coordinates (u,v).
    /// Returns (u, v) centered in the rect.
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f32, f32) {
        // Isometric projection formulas
        // u = (x - y) * cos(30°)
        // v = (x + y) * sin(30°) - z
        
        const COS_30: f32 = 0.866;
        const SIN_30: f32 = 0.5;
        
        let u = (x as f32 - y as f32) * COS_30;
        let v = (x as f32 + y as f32) * SIN_30 - (z as f32);
        
        // Scale and center
        (
            self.width / 2.0 + u * self.scale,
            self.height / 2.0 + v * self.scale
        )
    }
    
    /// Draws the room floor grid for reference
    pub fn draw_grid(&self, ui: &mut egui::Ui) {
        // Draw floor boundary (-1,-1) to (1,1)
        let corners = [
            (-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)
        ];
        
        let points: Vec<egui::Pos2> = corners.iter()
            .map(|(x, y)| {
                let (u, v) = self.project(*x, *y, -1.0); // Z = floor (-1.0)
                egui::pos2(u, v)
            })
            .collect();
            
        // Use ui.painter().add(egui::Shape::convex_polygon(...));
    }
}
```

### 🌍 Spatial Effect Examples

Using the `x, y, z` coordinates from `LightNode`, we can create immersive effects.

#### 1. Radial Explosion (Distance-based)
Triggered on a beat, expands from the center of the room.

```rust
pub struct RadialExplosion {
    radius: f32, // 0.0 to 2.0 (covers full room diagonal)
}

impl LightEffect for RadialExplosion {
    fn update(&mut self, _audio: &AudioSpectrum, nodes: &[LightNode]) -> HashMap<u8, (u8, u8, u8)> {
        self.radius += 0.1; // Expand
        
        let mut result = HashMap::new();
        for node in nodes {
            // Calculate distance from center (0,0,0) ignoring Z height likely
            let dist = (node.x.powi(2) + node.y.powi(2)).sqrt() as f32;
            
            // Gaussian bell curve peak at current radius
            let intensity = (-20.0 * (dist - self.radius).powi(2)).exp();
            
            if intensity > 0.01 {
                let r = (255.0 * intensity) as u8;
                result.insert(node.channel_id, (r, 0, 0)); // Red explosion
            }
        }
        result
    }
}
```

#### 2. Linear Wave (Directional)
A wave of color moving from Left to Right (or Front to Back).

```rust
pub struct LinearWave {
    phase: f32, // 0.0 to 2*PI
    direction: (f64, f64), // (1,0) = Left->Right, (0,1) = Back->Front
}

impl LightEffect for LinearWave {
    fn update(&mut self, _audio: &AudioSpectrum, nodes: &[LightNode]) -> HashMap<u8, (u8, u8, u8)> {
        self.phase += 0.2;
        
        let mut result = HashMap::new();
        for node in nodes {
            // Project position onto direction vector
            let metric = node.x * self.direction.0 + node.y * self.direction.1;
            
            // Sine wave based on position
            let val = ((metric * 2.0 + self.phase as f64).sin() + 1.0) / 2.0;
            
            let b = (255.0 * val) as u8;
            result.insert(node.channel_id, (0, 0, b)); // Blue wave
        }
        result
    }
}
```

#### 3. Height Map (Z-Axis)
Different colors for Floor, Eye-level, and Ceiling lights.

```rust
pub struct HeightMapEffect;

impl LightEffect for HeightMapEffect {
    fn update(&mut self, audio: &AudioSpectrum, nodes: &[LightNode]) -> HashMap<u8, (u8, u8, u8)> {
        let mut result = HashMap::new();
        for node in nodes {
            let color = if node.z < -0.5 {
                (50, 0, 0) // Floor: Dim Red
            } else if node.z > 0.5 {
                (200, 200, 255) // Ceiling: Bright Sky Blue
            } else {
                ((audio.bass * 255.0) as u8, 0, 0) // Eye-level: Reacts to Bass
            };
            result.insert(node.channel_id, color);
        }
        result
    }
}
```

#### 4. Sector/Zone Trigger
Only light up specific zones (e.g., "Left Rear") based on game events or triggers.

```rust
pub enum Zone {
    FrontLeft, FrontRight, RearLeft, RearRight
}

pub fn is_in_zone(node: &LightNode, zone: Zone) -> bool {
    match zone {
        Zone::FrontLeft => node.x < 0.0 && node.y > 0.0,
        Zone::FrontRight => node.x >= 0.0 && node.y > 0.0,
        Zone::RearLeft => node.x < 0.0 && node.y <= 0.0,
        Zone::RearRight => node.x >= 0.0 && node.y <= 0.0,
    }
}
```
