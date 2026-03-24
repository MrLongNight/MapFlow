//! Types for Audio Analyzer V2

use std::sync::Arc;

/// Audio analysis results from V2 analyzer
#[derive(Debug, Clone)]
pub struct AudioAnalysisV2 {
    /// Timestamp of this analysis
    pub timestamp: f64,
    /// RMS volume (0.0 - 1.0)
    pub rms_volume: f32,
    /// Peak volume (0.0 - 1.0)
    pub peak_volume: f32,
    /// FFT magnitude spectrum (half of FFT size)
    pub fft_magnitudes: Arc<Vec<f32>>,
    /// 9 frequency band energies
    pub band_energies: [f32; 9],
    /// Beat detected this frame
    pub beat_detected: bool,
    /// Beat strength (0.0 - 1.0)
    pub beat_strength: f32,
    /// Current waveform samples
    pub waveform: Arc<Vec<f32>>,
    /// Estimated tempo in BPM (None if not enough data)
    pub tempo_bpm: Option<f32>,
}

impl Default for AudioAnalysisV2 {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            rms_volume: 0.0,
            peak_volume: 0.0,
            fft_magnitudes: Arc::new(Vec::new()),
            band_energies: [0.0; 9],
            beat_detected: false,
            beat_strength: 0.0,
            waveform: Arc::new(Vec::new()),
            tempo_bpm: None,
        }
    }
}

/// Configuration for AudioAnalyzerV2
#[derive(Debug, Clone)]
pub struct AudioAnalyzerV2Config {
    /// Sample rate from audio backend
    pub sample_rate: u32,
    /// FFT size (power of 2)
    pub fft_size: usize,
    /// Overlap ratio (0.0 - 1.0, typically 0.5)
    pub overlap: f32,
    /// Smoothing factor for outputs
    pub smoothing: f32,
}

impl Default for AudioAnalyzerV2Config {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            fft_size: 2048,
            overlap: 0.5,
            smoothing: 0.7,
        }
    }
}
