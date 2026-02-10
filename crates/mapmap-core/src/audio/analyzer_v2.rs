//! Audio Analyzer V2 - Simplified FFT-based audio analysis
//!
//! This module provides a working audio analyzer using rustfft directly,
//! with proper sample buffering and FFT processing.

use crossbeam_channel::{bounded, Receiver, Sender};
use num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, trace};

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
    pub fft_magnitudes: Vec<f32>,
    /// 9 frequency band energies
    pub band_energies: [f32; 9],
    /// Beat detected this frame
    pub beat_detected: bool,
    /// Beat strength (0.0 - 1.0)
    pub beat_strength: f32,
    /// Current waveform samples
    pub waveform: Vec<f32>,
    /// Estimated tempo in BPM (None if not enough data)
    pub tempo_bpm: Option<f32>,
}

impl Default for AudioAnalysisV2 {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            rms_volume: 0.0,
            peak_volume: 0.0,
            fft_magnitudes: Vec::new(),
            band_energies: [0.0; 9],
            beat_detected: false,
            beat_strength: 0.0,
            waveform: Vec::new(),
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

/// Audio Analyzer V2 - Working implementation with proper buffering
pub struct AudioAnalyzerV2 {
    /// FFT instance
    fft: Arc<dyn Fft<f32>>,

    /// Configuration
    config: AudioAnalyzerV2Config,

    /// Input sample buffer (ring buffer for FFT)
    input_buffer: Vec<f32>,

    /// Write position in ring buffer
    buffer_write_pos: usize,

    /// Samples since last FFT
    samples_since_fft: usize,

    /// Hop size (samples between FFT frames)
    hop_size: usize,

    /// FFT complex buffer
    fft_buffer: Vec<Complex<f32>>,

    /// FFT scratch buffer
    scratch_buffer: Vec<Complex<f32>>,

    /// Hann window coefficients
    window: Vec<f32>,

    /// FFT magnitude buffer (half of FFT size - only positive frequencies)
    magnitude_buffer: Vec<f32>,

    /// Smoothed magnitude buffer
    smoothed_magnitudes: Vec<f32>,

    /// 9 frequency band energies
    band_energies: [f32; 9],

    /// Smoothed band energies
    smoothed_bands: [f32; 9],

    /// Current RMS volume
    rms_volume: f32,

    /// Smoothed RMS
    smoothed_rms: f32,

    /// Current peak volume (with decay)
    peak_volume: f32,

    /// Beat detection: energy history
    energy_history: VecDeque<f32>,

    /// Current timestamp
    current_time: f64,

    /// Waveform buffer
    waveform_buffer: Vec<f32>,

    /// Analysis sender channel
    analysis_sender: Sender<AudioAnalysisV2>,

    /// Analysis receiver channel
    analysis_receiver: Receiver<AudioAnalysisV2>,

    /// Latest analysis result
    latest_analysis: AudioAnalysisV2,

    /// Debug: sample count
    total_samples: u64,

    /// Debug: FFT count
    fft_count: u64,

    // === BPM Tracking ===
    /// Timestamps of detected beats (for BPM calculation)
    beat_timestamps: VecDeque<f64>,

    /// Current estimated BPM
    estimated_bpm: Option<f32>,

    /// Time since last beat (for beat detection cooldown)
    time_since_last_beat: f64,

    /// Minimum time between beats (prevents double-triggers) - ~200ms = 300 BPM max
    min_beat_interval: f64,
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

        // Sanitize input samples: replace NaN/Inf with 0.0
        // This is critical to prevent contamination of analysis metrics
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
            fft_magnitudes: self.smoothed_magnitudes.clone(),
            band_energies: self.smoothed_bands,
            beat_detected,
            beat_strength,
            waveform: self.waveform_buffer.clone(),
            tempo_bpm: self.estimated_bpm,
        };

        // Store and send
        self.latest_analysis = analysis.clone();
        let _ = self.analysis_sender.try_send(analysis);
    }

    /// Perform FFT on current buffer
    fn perform_fft(&mut self) {
        self.fft_count += 1;

        // Copy ring buffer to FFT buffer with proper unwrapping
        // The write position is where we'll write NEXT, so data starts there
        for i in 0..self.config.fft_size {
            let src_idx = (self.buffer_write_pos + i) % self.config.fft_size;
            let windowed = self.input_buffer[src_idx] * self.window[i];
            self.fft_buffer[i] = Complex::new(windowed, 0.0);
        }

        // Perform FFT in-place
        self.fft
            .process_with_scratch(&mut self.fft_buffer, &mut self.scratch_buffer);

        // Extract magnitudes (only first half - positive frequencies)
        let half_size = self.magnitude_buffer.len();
        let norm_factor = 1.0 / (self.config.fft_size as f32).sqrt();

        for i in 0..half_size {
            let magnitude = self.fft_buffer[i].norm() * norm_factor;

            // Smooth magnitudes
            self.magnitude_buffer[i] = magnitude;
            self.smoothed_magnitudes[i] = self.smoothed_magnitudes[i] * self.config.smoothing
                + magnitude * (1.0 - self.config.smoothing);
        }

        // Update band energies
        self.update_band_energies();

        // Trace log every 100 FFTs
        if self.fft_count % 100 == 0 {
            trace!(
                "FFT #{}: bands={:?}",
                self.fft_count,
                &self.smoothed_bands[..3]
            );
        }
    }

    /// Calculate RMS from samples
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    /// Calculate energy in each of 9 frequency bands
    fn update_band_energies(&mut self) {
        let bin_width = self.config.sample_rate as f32 / self.config.fft_size as f32;

        // 9 frequency bands covering 20Hz - 20kHz
        let band_ranges: [(f32, f32); 9] = [
            (20.0, 60.0),       // SubBass
            (60.0, 250.0),      // Bass
            (250.0, 500.0),     // LowMid
            (500.0, 1000.0),    // Mid
            (1000.0, 2000.0),   // HighMid
            (2000.0, 4000.0),   // UpperMid
            (4000.0, 6000.0),   // Presence
            (6000.0, 12000.0),  // Brilliance
            (12000.0, 20000.0), // Air
        ];

        for (i, (min_freq, max_freq)) in band_ranges.iter().enumerate() {
            let min_bin = (*min_freq / bin_width) as usize;
            let max_bin = ((*max_freq / bin_width) as usize)
                .min(self.smoothed_magnitudes.len().saturating_sub(1));

            if max_bin > min_bin && min_bin < self.smoothed_magnitudes.len() {
                let sum: f32 = self.smoothed_magnitudes[min_bin..=max_bin].iter().sum();
                let count = (max_bin - min_bin + 1) as f32;
                self.band_energies[i] = sum / count;

                // Smooth bands
                self.smoothed_bands[i] = self.smoothed_bands[i] * self.config.smoothing
                    + self.band_energies[i] * (1.0 - self.config.smoothing);
            }
        }
    }

    /// Simple beat detection based on energy spike
    fn detect_beat(&mut self, timestamp: f64) -> (bool, f32) {
        // Use bass band (60-250Hz) for beat detection
        let bass_energy = self.smoothed_bands[1];

        // Add current energy to history
        self.energy_history.push_back(bass_energy);

        // Keep last 32 values
        if self.energy_history.len() > 32 {
            self.energy_history.pop_front();
        }

        // Update time since last beat
        if !self.beat_timestamps.is_empty() {
            self.time_since_last_beat = timestamp - self.beat_timestamps.back().unwrap_or(&0.0);
        }

        // Need at least 16 samples for detection
        if self.energy_history.len() < 16 {
            return (false, 0.0);
        }

        // Calculate average energy
        let avg_energy: f32 =
            self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;

        // Calculate beat strength (how much above average)
        let beat_strength = if avg_energy > 0.0 {
            (bass_energy / avg_energy - 1.0).clamp(0.0, 2.0) / 2.0
        } else {
            0.0
        };

        // Beat if current energy is 1.5x average, above minimum threshold, and cooldown passed
        let is_beat = bass_energy > avg_energy * 1.5
            && bass_energy > 0.01
            && self.time_since_last_beat >= self.min_beat_interval;

        if is_beat {
            // Record beat timestamp
            self.beat_timestamps.push_back(timestamp);

            // Keep only last 16 beats for BPM calculation
            if self.beat_timestamps.len() > 16 {
                self.beat_timestamps.pop_front();
            }

            // Calculate BPM from beat intervals
            self.estimated_bpm = self.calculate_bpm();

            // Reset cooldown timer
            self.time_since_last_beat = 0.0;
        }

        (is_beat, beat_strength)
    }

    /// Calculate BPM from recent beat timestamps
    fn calculate_bpm(&self) -> Option<f32> {
        if self.beat_timestamps.len() < 4 {
            return None; // Need at least 4 beats
        }

        // Calculate intervals between consecutive beats
        let mut intervals: Vec<f64> = self
            .beat_timestamps
            .iter()
            .zip(self.beat_timestamps.iter().skip(1))
            .map(|(a, b)| b - a)
            .collect();

        if intervals.is_empty() {
            return None;
        }

        // Sort intervals to filtering outliers
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Remove outliers (keep middle 50% approx) if we have enough samples
        let valid_intervals = if intervals.len() >= 4 {
            let start = intervals.len() / 4;
            let end = intervals.len() - start;
            &intervals[start..end]
        } else {
            &intervals[..]
        };

        if valid_intervals.is_empty() {
            return None;
        }

        // Calculate average interval of the clean set
        let avg_interval: f64 = valid_intervals.iter().sum::<f64>() / valid_intervals.len() as f64;

        if avg_interval <= 0.001 {
            return None;
        }

        // Convert to BPM: BPM = 60 / interval_in_seconds
        let bpm = (60.0 / avg_interval) as f32;

        // Clamp to reasonable DJ range (60-200 BPM)
        let bpm = if (60.0..=200.0).contains(&bpm) {
            bpm
        } else if (200.0..=400.0).contains(&bpm) {
            // Might be detecting half-beats, divide by 2
            bpm / 2.0
        } else if (30.0..60.0).contains(&bpm) {
            // Might be detecting double-beats, multiply by 2
            bpm * 2.0
        } else {
            return None;
        };

        // Round to 1 decimal place to reduce visual jitter
        Some((bpm * 10.0).round() / 10.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_analyzer() {
        let config = AudioAnalyzerV2Config::default();
        let analyzer = AudioAnalyzerV2::new(config);
        assert_eq!(analyzer.sample_rate(), 44100);
    }

    #[test]
    fn test_rms_calculation() {
        // Sine wave at 0.5 amplitude should give RMS of ~0.35
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();

        let rms = AudioAnalyzerV2::calculate_rms(&samples);
        assert!(rms > 0.3 && rms < 0.4, "RMS was {}", rms);
    }

    #[test]
    fn test_process_samples() {
        let config = AudioAnalyzerV2Config {
            fft_size: 1024,
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Generate test samples (440Hz sine wave)
        let sample_rate = 44100.0;
        let freq = 440.0;
        let samples: Vec<f32> = (0..4096)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin() * 0.5)
            .collect();

        analyzer.process_samples(&samples, 0.0);

        let analysis = analyzer.get_latest_analysis();
        assert!(
            analysis.rms_volume > 0.0,
            "RMS should be > 0, was {}",
            analysis.rms_volume
        );
        assert!(
            analysis.peak_volume > 0.0,
            "Peak should be > 0, was {}",
            analysis.peak_volume
        );

        // Check that FFT was performed (magnitudes should have values)
        let mag_sum: f32 = analysis.fft_magnitudes.iter().sum();
        assert!(
            mag_sum > 0.0,
            "FFT magnitudes should have values, sum={}",
            mag_sum
        );
    }

    #[test]
    fn test_frequency_bands() {
        let config = AudioAnalyzerV2Config {
            fft_size: 2048,
            sample_rate: 44100,
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Generate 100Hz sine (should appear in bass band)
        let sample_rate = 44100.0;
        let freq = 100.0;
        let samples: Vec<f32> = (0..8192)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin() * 0.5)
            .collect();

        analyzer.process_samples(&samples, 0.0);

        let analysis = analyzer.get_latest_analysis();

        // Bass band (index 1, 60-250Hz) should have the most energy
        let bass = analysis.band_energies[1];
        let other_bands: f32 = analysis.band_energies[3..].iter().sum::<f32>() / 6.0;

        // Bass should be significantly higher than average of higher bands
        assert!(
            bass > other_bands * 2.0 || bass > 0.01,
            "Bass band should dominate for 100Hz signal: bass={}, others_avg={}",
            bass,
            other_bands
        );
    }

    #[test]
    fn test_reset() {
        let config = AudioAnalyzerV2Config::default();
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Process some samples
        let samples: Vec<f32> = (0..4096).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        analyzer.process_samples(&samples, 0.0);

        // Verify we have data
        assert!(analyzer.get_latest_analysis().rms_volume > 0.0);

        // Reset
        analyzer.reset();

        let analysis = analyzer.get_latest_analysis();
        assert_eq!(analysis.rms_volume, 0.0);
        assert_eq!(analysis.peak_volume, 0.0);
    }

    #[test]
    fn test_beat_detection_simulation() {
        let config = AudioAnalyzerV2Config {
            sample_rate: 44100,
            fft_size: 1024, // Smaller FFT for faster test
            smoothing: 0.0, // Disable smoothing for instant reaction
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Generate silence
        let silence = vec![0.0f32; 1024];

        // Generate a strong bass kick (60Hz)
        let sample_rate = 44100.0;
        let kick_freq = 60.0;
        let kick: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * kick_freq * i as f32 / sample_rate).sin())
            .collect();

        // 1. Fill history with silence to establish low average
        for i in 0..20 {
            analyzer.process_samples(&silence, i as f64 * 0.02);
        }

        assert!(!analyzer.get_latest_analysis().beat_detected);

        // 2. Inject Kick
        analyzer.process_samples(&kick, 1.0);

        // 3. Check for beat
        let analysis = analyzer.get_latest_analysis();
        assert!(
            analysis.beat_detected,
            "Beat should be detected after silence -> kick. Bass energy: {}",
            analysis.band_energies[1]
        );
        assert!(analysis.beat_strength > 0.0);
    }

    #[test]
    fn test_bpm_estimation_simulation() {
        let config = AudioAnalyzerV2Config {
            sample_rate: 44100,
            fft_size: 1024,
            smoothing: 0.0,
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config);

        let sample_rate = 44100.0;
        let kick_freq = 60.0;
        let bpm = 120.0;
        let beat_interval_samples = (sample_rate * 60.0 / bpm) as usize; // 22050 samples
        let kick_duration = 2000; // Short kick

        // Simulate 10 seconds of audio (should be enough for ~20 beats)
        // We process in chunks of 512 samples
        let total_samples = beat_interval_samples * 20;
        let chunk_size = 512;
        let mut current_sample_idx = 0;

        let mut bpm_found = false;

        while current_sample_idx < total_samples {
            let mut chunk = Vec::with_capacity(chunk_size);
            for i in 0..chunk_size {
                let absolute_idx = current_sample_idx + i;
                let position_in_beat = absolute_idx % beat_interval_samples;

                let sample = if position_in_beat < kick_duration {
                    // Kick
                    (2.0 * std::f32::consts::PI * kick_freq * position_in_beat as f32 / sample_rate)
                        .sin()
                } else {
                    0.0
                };
                chunk.push(sample);
            }

            let timestamp = current_sample_idx as f64 / sample_rate as f64;
            analyzer.process_samples(&chunk, timestamp);

            if let Some(detected_bpm) = analyzer.get_latest_analysis().tempo_bpm {
                // Allow some tolerance because simulation isn't perfect
                if (detected_bpm - bpm).abs() < 2.0 {
                    bpm_found = true;
                    break;
                }
            }

            current_sample_idx += chunk_size;
        }

        assert!(
            bpm_found,
            "Failed to detect 120 BPM. Last detected: {:?}",
            analyzer.get_latest_analysis().tempo_bpm
        );
    }

    #[test]
    fn test_update_config_resizes_buffers() {
        let mut config = AudioAnalyzerV2Config {
            fft_size: 1024,
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config.clone());

        // Check initial sizes
        assert_eq!(analyzer.input_buffer.len(), 1024);
        assert_eq!(analyzer.magnitude_buffer.len(), 512);

        // Update config
        config.fft_size = 2048;
        analyzer.update_config(config);

        // Check new sizes
        assert_eq!(analyzer.input_buffer.len(), 2048);
        assert_eq!(analyzer.magnitude_buffer.len(), 1024);
        assert_eq!(analyzer.fft_buffer.len(), 2048);
        assert_eq!(analyzer.scratch_buffer.len(), 2048);
    }

    #[test]
    fn test_resilience_to_bad_input() {
        let config = AudioAnalyzerV2Config::default();
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Feed NaN and Infinity
        let bad_samples = vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0];

        // Should not panic, and results should be clean (0.0)
        analyzer.process_samples(&bad_samples, 0.0);

        let analysis = analyzer.get_latest_analysis();

        // Assert that ALL analysis outputs are finite and valid
        assert!(analysis.rms_volume.is_finite());
        assert!(analysis.peak_volume.is_finite());

        // Check that bad inputs were effectively treated as silence (0.0)
        assert_eq!(analysis.rms_volume, 0.0);
        assert_eq!(analysis.peak_volume, 0.0);

        // Check FFT magnitudes are clean
        for mag in analysis.fft_magnitudes {
            assert!(mag.is_finite());
        }

        // Check band energies are clean
        for band in analysis.band_energies {
            assert!(band.is_finite());
        }
    }

    #[test]
    fn test_smoothing_behavior() {
        let config = AudioAnalyzerV2Config {
            sample_rate: 44100,
            fft_size: 1024,
            smoothing: 0.9, // High smoothing
            ..Default::default()
        };
        let mut analyzer = AudioAnalyzerV2::new(config);

        // Feed silence first
        let silence = vec![0.0f32; 1024];
        analyzer.process_samples(&silence, 0.0);
        assert_eq!(analyzer.get_latest_analysis().rms_volume, 0.0);

        // Feed Loud Signal (Amplitude 1.0)
        let loud: Vec<f32> = vec![1.0; 1024]; // DC offset for max RMS calculation simplification

        // Step 1
        analyzer.process_samples(&loud, 1.0);
        let rms1 = analyzer.get_latest_analysis().rms_volume;

        // With smoothing 0.9: New = Old * 0.9 + Input * 0.1
        // Input RMS is 1.0. Old is 0.0.
        // rms1 should be approx 0.1
        assert!(rms1 > 0.05 && rms1 < 0.2, "RMS1 was {}", rms1);

        // Step 2
        analyzer.process_samples(&loud, 2.0);
        let rms2 = analyzer.get_latest_analysis().rms_volume;

        // rms2 should be approx 0.1 * 0.9 + 1.0 * 0.1 = 0.09 + 0.1 = 0.19
        assert!(rms2 > rms1, "RMS should increase over time with smoothing");
        assert!(rms2 < 1.0, "RMS should not reach target instantly");
    }
}
