# Audio-Analyse Migration: audio-processor-analysis Integration

> **Version:** 1.0  
> **Stand:** 2025-12-29  
> **Status:** GEPLANT  
> **Priorität:** KRITISCH

---

## 📋 Problem-Analyse

### Aktuelle Implementierung (DEFEKT)

Die bestehende Audio-Analyse in `mapmap-core/src/audio/mod.rs` funktioniert nicht korrekt:

1. **Manuelle FFT-Implementierung** hat unbekannte Fehler
2. **RMS-Berechnung** liefert keine korrekten Werte
3. **Sample-Puffer-Management** ist fehlerhaft
4. **Konfiguration war falsch** (Gain=100x, jetzt 1.0, aber immer noch defekt)
5. **Keine Verifizierung** der Audio-Daten-Pipeline

### Symptome

- Audio-Meter zeigen keine Werte an
- FFT-Spektrum bleibt leer
- Beat-Detection funktioniert nicht
- Frequenzband-Analyse zeigt keine Aktivität

---

## 🎯 Lösungsansatz: audio-processor-analysis Crate

Die [`audio-processor-analysis`](https://docs.rs/audio-processor-analysis/latest/) Crate bietet:

### Verfügbare Module

| Modul | Beschreibung | Ersetzt |
|-------|--------------|---------|
| `FftProcessorImpl` | FFT mit Window-Funktionen und Overlap | `perform_fft()`, `apply_window()` |
| `RunningRMSProcessorImpl` | Real-time safe RMS | `calculate_rms()` |
| `PeakDetectorImpl` | Peak mit Attack/Release | `peak_volume` Berechnung |
| `EnvelopeFollower` | Hüllkurven-Tracking | Beat/Onset Detection Helper |
| `TransientDetectionProcessor` | Transient Detection | `detect_onset()` |
| `WindowFunctionType` | Hann, Hamming, Blackman, etc. | Manuelle Hann-Window |

### Vorteile

1. **Getestet & bewährt** - Produktionsreife Library
2. **Real-time safe** - Keine Allokationen im Audio-Thread
3. **Flexible Konfiguration** - Window-Funktionen, Overlap, Size
4. **Konsistente API** - `AudioProcessor` Trait

---

## 📦 Neue Dependencies

```toml
# crates/mapmap-core/Cargo.toml
[dependencies]
audio-processor-analysis = "2.4"
audio-processor-traits = "4.3"
basedrop = "0.1"  # Für RunningRMS GC Handle
```

---

## 🏗️ Implementierungsplan

### Phase 1: Dependency Integration (30 min)

1. [ ] `audio-processor-analysis` zu `mapmap-core/Cargo.toml` hinzufügen
2. [ ] Feature-Flags prüfen (optional `analysis` Feature?)
3. [ ] Compile-Test durchführen

### Phase 2: Neue AudioAnalyzer-Struktur (2 Stunden)

Datei: `mapmap-core/src/audio/analyzer.rs` (NEU)

```rust
use audio_processor_analysis::{
    fft_processor::{FftProcessorImpl, FftProcessorOptions, FftDirection},
    peak_detector::PeakDetectorImpl,
    window_functions::WindowFunctionType,
};

pub struct AudioAnalyzerV2 {
    // FFT Processor
    fft: FftProcessorImpl<f32>,
    
    // Peak Detector (pro Kanal)
    peak_detector: PeakDetectorImpl<f32>,
    
    // Konfiguration
    sample_rate: u32,
    fft_size: usize,
    
    // Output Buffer
    magnitude_buffer: Vec<f32>,
    band_energies: [f32; 9],  // 9 Bänder
    
    // Metrics
    rms_volume: f32,
    peak_volume: f32,
}

impl AudioAnalyzerV2 {
    pub fn new(sample_rate: u32, fft_size: usize) -> Self {
        let options = FftProcessorOptions {
            size: fft_size,
            direction: FftDirection::Forward,
            overlap_ratio: 0.5,
            window_function: WindowFunctionType::Hann,
        };
        
        Self {
            fft: FftProcessorImpl::new(options),
            peak_detector: PeakDetectorImpl::default(),
            sample_rate,
            fft_size,
            magnitude_buffer: vec![0.0; fft_size / 2],
            band_energies: [0.0; 9],
            rms_volume: 0.0,
            peak_volume: 0.0,
        }
    }
    
    pub fn process(&mut self, samples: &[f32]) {
        // 1. Samples durch FFT verarbeiten
        for sample in samples {
            self.fft.s_process(*sample);
            
            if self.fft.has_changed() {
                self.update_magnitudes();
                self.update_band_energies();
            }
        }
        
        // 2. RMS berechnen
        self.rms_volume = self.calculate_rms(samples);
        
        // 3. Peak aktualisieren
        self.peak_detector.accept_frame(0.9, 0.99, samples);
        self.peak_volume = self.peak_detector.value();
    }
    
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() { return 0.0; }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }
    
    fn update_magnitudes(&mut self) {
        let buffer = self.fft.buffer();
        for (i, complex) in buffer.iter().take(self.magnitude_buffer.len()).enumerate() {
            self.magnitude_buffer[i] = complex.norm();
        }
    }
    
    fn update_band_energies(&mut self) {
        // Frequenzband-Berechnung basierend auf FFT-Bins
        let bin_width = self.sample_rate as f32 / self.fft_size as f32;
        
        // 9 Bänder: SubBass, Bass, LowMid, Mid, HighMid, UpperMid, Presence, Brilliance, Air
        let band_ranges = [
            (20.0, 60.0),     // SubBass
            (60.0, 250.0),    // Bass
            (250.0, 500.0),   // LowMid
            (500.0, 1000.0),  // Mid
            (1000.0, 2000.0), // HighMid
            (2000.0, 4000.0), // UpperMid
            (4000.0, 6000.0), // Presence
            (6000.0, 12000.0),// Brilliance
            (12000.0, 20000.0),// Air
        ];
        
        for (i, (min_freq, max_freq)) in band_ranges.iter().enumerate() {
            let min_bin = (*min_freq / bin_width) as usize;
            let max_bin = ((*max_freq / bin_width) as usize).min(self.magnitude_buffer.len() - 1);
            
            if max_bin > min_bin {
                let sum: f32 = self.magnitude_buffer[min_bin..=max_bin].iter().sum();
                self.band_energies[i] = sum / (max_bin - min_bin + 1) as f32;
            }
        }
    }
    
    // Getter
    pub fn rms(&self) -> f32 { self.rms_volume }
    pub fn peak(&self) -> f32 { self.peak_volume }
    pub fn band_energies(&self) -> &[f32; 9] { &self.band_energies }
    pub fn magnitudes(&self) -> &[f32] { &self.magnitude_buffer }
}
```

### Phase 3: Integration in Audio-Backend (1 Stunde)

1. [ ] `AudioAnalyzerV2` in `main.rs` instanziieren
2. [ ] Sample-Rate vom CPAL-Backend übernehmen
3. [ ] `process()` mit Samples aus `backend.get_samples()` aufrufen
4. [ ] Debug-Logging hinzufügen

```rust
// In main.rs render loop
let samples = backend.get_samples();
if !samples.is_empty() {
    debug!("Processing {} samples, first 5: {:?}", samples.len(), &samples[..5.min(samples.len())]);
    self.audio_analyzer_v2.process(&samples);
    
    debug!("RMS: {:.4}, Peak: {:.4}, Bands: {:?}", 
           self.audio_analyzer_v2.rms(),
           self.audio_analyzer_v2.peak(),
           self.audio_analyzer_v2.band_energies());
}
```

### Phase 4: UI-Anbindung (30 min)

1. [ ] `AudioAnalysis` Struct aktualisieren mit V2-Daten
2. [ ] Meter-Widget mit `rms()` und `peak()` füttern
3. [ ] Spektrum-Visualisierung mit `magnitudes()` füttern
4. [ ] Band-Visualisierung mit `band_energies()` füttern

### Phase 5: Alte Implementierung entfernen (30 min)

1. [ ] `AudioAnalyzer` (alter Code) deprecaten
2. [ ] Compile-Warnungen beheben
3. [ ] Tests aktualisieren
4. [ ] Dokumentation aktualisieren

---

## ✅ Erfolgskriterien

- [ ] RMS-Meter zeigt Werte bei Audio-Input
- [ ] Peak-Meter reagiert auf laute Signale
- [ ] FFT-Spektrum visualisiert Frequenzen
- [ ] 9 Frequenzbänder sind korrekt getrennt
- [ ] Debug-Log zeigt plausible Werte (RMS 0.0-1.0, Bänder 0.0-1.0)

---

## 🔗 Referenzen

- [audio-processor-analysis Docs](https://docs.rs/audio-processor-analysis/latest/)
- [audio-processor-traits Docs](https://docs.rs/audio-processor-traits/latest/)
- [rustfft Docs](https://docs.rs/rustfft/latest/)
