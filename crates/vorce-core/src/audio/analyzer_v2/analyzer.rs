//! Audio Analyzer V2 Core
use crossbeam_channel::{bounded, Receiver, Sender};
use num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::debug;

use super::types::{AudioAnalysisV2, AudioAnalyzerV2Config};

/// Audio Analyzer V2 - Working implementation with proper buffering
pub struct AudioAnalyzerV2 {
    /// FFT instance
    pub(crate) fft: Arc<dyn Fft<f32>>,

    /// Configuration
    pub(crate) config: AudioAnalyzerV2Config,

    /// Input sample buffer (ring buffer for FFT)
    pub(crate) input_buffer: Vec<f32>,

    /// Write position in ring buffer
    pub(crate) buffer_write_pos: usize,

    /// Samples since last FFT
    pub(crate) samples_since_fft: usize,

    /// Hop size (samples between FFT frames)
    pub(crate) hop_size: usize,

    /// FFT complex buffer
    pub(crate) fft_buffer: Vec<Complex<f32>>,

    /// FFT scratch buffer
    pub(crate) scratch_buffer: Vec<Complex<f32>>,

    /// Hann window coefficients
    pub(crate) window: Vec<f32>,

    /// FFT magnitude buffer (half of FFT size - only positive frequencies)
    pub(crate) magnitude_buffer: Vec<f32>,

    /// Smoothed magnitude buffer
    pub(crate) smoothed_magnitudes: Vec<f32>,

    /// 9 frequency band energies
    pub(crate) band_energies: [f32; 9],

    /// Smoothed band energies
    pub(crate) smoothed_bands: [f32; 9],

    /// Current RMS volume
    pub(crate) rms_volume: f32,

    /// Smoothed RMS
    pub(crate) smoothed_rms: f32,

    /// Current peak volume (with decay)
    pub(crate) peak_volume: f32,

    /// Beat detection: energy history
    pub(crate) energy_history: VecDeque<f32>,

    /// Current timestamp
    pub(crate) current_time: f64,

    /// Waveform buffer
    pub(crate) waveform_buffer: Vec<f32>,

    /// Analysis sender channel
    pub(crate) analysis_sender: Sender<AudioAnalysisV2>,

    /// Analysis receiver channel
    pub(crate) analysis_receiver: Receiver<AudioAnalysisV2>,

    /// Latest analysis result
    pub(crate) latest_analysis: AudioAnalysisV2,

    /// Debug: sample count
    pub(crate) total_samples: u64,

    /// Debug: FFT count
    pub(crate) fft_count: u64,

    // === BPM Tracking ===
    /// Timestamps of detected beats (for BPM calculation)
    pub(crate) beat_timestamps: VecDeque<f64>,

    /// Current estimated BPM
    pub(crate) estimated_bpm: Option<f32>,

    /// Time since last beat (for beat detection cooldown)
    pub(crate) time_since_last_beat: f64,

    /// Minimum time between beats (prevents double-triggers) - ~200ms = 300 BPM max
    pub(crate) min_beat_interval: f64,
}

impl AudioAnalyzerV2 {
    /// Create a new AudioAnalyzerV2 with the given configuration
    pub fn new(config: AudioAnalyzerV2Config) -> Self {
        let fft_size = config.fft_size;
        let sample_rate = config.sample_rate;
        let overlap = config.overlap;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let half_size = fft_size / 2;
        let hop_size = ((1.0 - overlap) * fft_size as f32) as usize;
        let hop_size = hop_size.max(1);

        // Pre-compute Hann window
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                let t = i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * t).cos())
            })
            .collect();

        let (tx, rx) = bounded(16);

        debug!(
            "AudioAnalyzerV2 created: sample_rate={}, fft_size={}, hop_size={}, overlap={}",
            sample_rate, fft_size, hop_size, overlap
        );

        Self {
            fft,
            config,
            input_buffer: vec![0.0; fft_size],
            buffer_write_pos: 0,
            samples_since_fft: 0,
            hop_size,
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            scratch_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            window,
            magnitude_buffer: vec![0.0; half_size],
            smoothed_magnitudes: vec![0.0; half_size],
            band_energies: [0.0; 9],
            smoothed_bands: [0.0; 9],
            rms_volume: 0.0,
            smoothed_rms: 0.0,
            peak_volume: 0.0,
            energy_history: VecDeque::with_capacity(64),
            current_time: 0.0,
            waveform_buffer: Vec::with_capacity(2048),
            analysis_sender: tx,
            analysis_receiver: rx,
            latest_analysis: AudioAnalysisV2::default(),
            total_samples: 0,
            fft_count: 0,
            // BPM Tracking
            beat_timestamps: VecDeque::with_capacity(32),
            estimated_bpm: None,
            time_since_last_beat: 1.0, // Start ready for beat
            min_beat_interval: 0.2,    // 300 BPM max
        }
    }

    /// Process incoming audio samples
    pub fn process_samples(&mut self, samples: &[f32], timestamp: f64) {
        if samples.is_empty() {
            return;
        }

        // Sanitize input samples: replace non-finite values (NaN, Infinity) with 0.0
        // to prevent contamination of analysis metrics.
        let sanitized_samples: Vec<f32> = samples
            .iter()
            .map(|&s| if s.is_finite() { s } else { 0.0 })
            .collect();
        let samples = &sanitized_samples;

        self.current_time = timestamp;
        self.total_samples += samples.len() as u64;

        // Log every ~1 second of audio
        if self.total_samples % 44100 < samples.len() as u64 {
            debug!(
                "AudioV2: Processed {}k samples, {} FFTs, current RMS={:.4}",
                self.total_samples / 1000,
                self.fft_count,
                self.smoothed_rms
            );
        }

        // Store waveform for visualization
        self.waveform_buffer.clear();
        self.waveform_buffer
            .extend_from_slice(&samples[..samples.len().min(2048)]);

        // 1. Calculate RMS from input samples
        self.rms_volume = Self::calculate_rms(samples);
        self.smoothed_rms = self.smoothed_rms * self.config.smoothing
            + self.rms_volume * (1.0 - self.config.smoothing);

        // 2. Update peak (fast attack, slow decay)
        let max_sample = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if max_sample > self.peak_volume {
            self.peak_volume = max_sample; // Fast attack
        } else {
            self.peak_volume *= 0.995; // Slow decay
        }

        // 3. Add samples to ring buffer and perform FFT when ready
        for &sample in samples {
            self.input_buffer[self.buffer_write_pos] = sample;
            self.buffer_write_pos = (self.buffer_write_pos + 1) % self.config.fft_size;
            self.samples_since_fft += 1;

            // Perform FFT every hop_size samples (after first full buffer)
            if self.samples_since_fft >= self.hop_size
                && self.total_samples >= self.config.fft_size as u64
            {
                self.perform_fft();
                self.samples_since_fft = 0;
            }
        }

        // 4. Beat detection with BPM tracking
        let (beat_detected, beat_strength) = self.detect_beat(self.current_time);

        // 5. Create analysis result
        let analysis = AudioAnalysisV2 {
            timestamp: self.current_time,
            rms_volume: self.smoothed_rms,
            peak_volume: self.peak_volume,
            fft_magnitudes: Arc::new(self.smoothed_magnitudes.clone()),
            band_energies: self.smoothed_bands,
            beat_detected,
            beat_strength,
            waveform: Arc::new(self.waveform_buffer.clone()),
            tempo_bpm: self.estimated_bpm,
        };

        // Store and send
        self.latest_analysis = analysis.clone();
        let _ = self.analysis_sender.try_send(analysis);
    }

    /// Calculate RMS from samples
    pub(crate) fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    /// Get the latest analysis result
    pub fn get_latest_analysis(&self) -> AudioAnalysisV2 {
        self.latest_analysis.clone()
    }

    /// Try to receive analysis from channel
    pub fn try_receive(&self) -> Option<AudioAnalysisV2> {
        self.analysis_receiver.try_recv().ok()
    }

    /// Reset the analyzer state
    pub fn reset(&mut self) {
        self.input_buffer.fill(0.0);
        self.buffer_write_pos = 0;
        self.samples_since_fft = 0;
        self.magnitude_buffer.fill(0.0);
        self.smoothed_magnitudes.fill(0.0);
        self.band_energies = [0.0; 9];
        self.smoothed_bands = [0.0; 9];
        self.rms_volume = 0.0;
        self.smoothed_rms = 0.0;
        self.peak_volume = 0.0;
        self.energy_history.clear();
        self.waveform_buffer.clear();
        self.latest_analysis = AudioAnalysisV2::default();
        self.total_samples = 0;
        self.fft_count = 0;

        debug!("AudioAnalyzerV2 reset");
    }

    /// Update configuration (e.g., when sample rate changes)
    pub fn update_config(&mut self, config: AudioAnalyzerV2Config) {
        if config.fft_size != self.config.fft_size {
            // Need to recreate FFT
            let mut planner = FftPlanner::new();
            self.fft = planner.plan_fft_forward(config.fft_size);

            let half_size = config.fft_size / 2;
            self.input_buffer = vec![0.0; config.fft_size];
            self.fft_buffer = vec![Complex::new(0.0, 0.0); config.fft_size];
            self.scratch_buffer = vec![Complex::new(0.0, 0.0); config.fft_size];
            self.magnitude_buffer = vec![0.0; half_size];
            self.smoothed_magnitudes = vec![0.0; half_size];

            // Recompute window
            self.window = (0..config.fft_size)
                .map(|i| {
                    let t = i as f32 / (config.fft_size - 1) as f32;
                    0.5 * (1.0 - (2.0 * std::f32::consts::PI * t).cos())
                })
                .collect();
        }

        self.hop_size = ((1.0 - config.overlap) * config.fft_size as f32) as usize;
        self.hop_size = self.hop_size.max(1);
        self.config = config;

        debug!("AudioAnalyzerV2 config updated, hop_size={}", self.hop_size);
    }

    /// Get current sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }
}
