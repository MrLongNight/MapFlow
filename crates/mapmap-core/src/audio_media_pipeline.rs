//! Audio-Media Pipeline Integration
//!
//! This module connects the audio backend to the media pipeline,
//! allowing audio analysis to drive visual effects in real-time.
//!
//! ## Features
//! - Real-time audio analysis with FFT and beat detection
//! - Configurable latency compensation for audio-video sync
//! - Audio-reactive parameter mapping
//! - Ring buffer for historical analysis data

use crate::audio::{AudioAnalysis, AudioAnalyzer, AudioConfig};
use crate::audio_reactive::AudioReactiveController;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for the audio-media pipeline
#[derive(Debug, Clone)]
pub struct AudioPipelineConfig {
    /// Sample rate in Hz (default: 44100)
    pub sample_rate: u32,
    /// Latency compensation in milliseconds (default: 0)
    pub latency_ms: f32,
    /// Number of analysis frames to buffer for smoothing (default: 8)
    pub analysis_buffer_size: usize,
    /// Enable automatic latency detection (default: true)
    pub auto_latency_detection: bool,
}

impl Default for AudioPipelineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            latency_ms: 0.0,
            analysis_buffer_size: 8,
            auto_latency_detection: true,
        }
    }
}

/// Statistics for the audio pipeline
#[derive(Debug, Clone, Default)]
pub struct AudioPipelineStats {
    /// Total samples processed
    pub samples_processed: u64,
    /// Total analysis frames generated
    pub frames_analyzed: u64,
    /// Current buffer fill level (0.0 - 1.0)
    pub buffer_fill: f32,
    /// Estimated audio latency in ms
    pub estimated_latency_ms: f32,
    /// Dropped samples due to buffer overflow
    pub dropped_samples: u64,
}

/// Timestamped audio analysis for latency compensation
#[derive(Clone)]
pub struct TimestampedAnalysis {
    /// The audio analysis data
    pub analysis: AudioAnalysis,
    /// Timestamp when this analysis was generated
    pub timestamp: Instant,
    /// Audio position in seconds
    pub audio_position: f64,
}

/// Audio pipeline that integrates with media processing
pub struct AudioMediaPipeline {
    /// Audio analyzer for FFT and beat detection
    #[allow(dead_code)]
    analyzer: Arc<RwLock<AudioAnalyzer>>,

    /// Audio-reactive controller for parameter mapping
    reactive_controller: Arc<RwLock<AudioReactiveController>>,

    /// Channel to send audio samples to analyzer
    sample_sender: Sender<Vec<f32>>,

    /// Channel to receive analyzed data
    analysis_receiver: Receiver<AudioAnalysis>,

    /// Pipeline configuration
    config: AudioPipelineConfig,

    /// Ring buffer of recent analyses for latency compensation
    analysis_buffer: VecDeque<TimestampedAnalysis>,

    /// Current audio position in seconds
    audio_position: f64,

    /// Statistics
    samples_processed: Arc<AtomicU64>,
    frames_analyzed: Arc<AtomicU64>,
    dropped_samples: Arc<AtomicU64>,

    /// Running state
    is_running: Arc<AtomicBool>,

    /// Last analysis timestamp for rate calculation
    last_analysis_time: Option<Instant>,
}

impl AudioMediaPipeline {
    /// Create a new audio-media pipeline with default config
    pub fn new(audio_config: AudioConfig) -> Self {
        Self::with_config(audio_config, AudioPipelineConfig::default())
    }

    /// Create a new audio-media pipeline with custom config
    pub fn with_config(audio_config: AudioConfig, pipeline_config: AudioPipelineConfig) -> Self {
        let analyzer = Arc::new(RwLock::new(AudioAnalyzer::new(audio_config)));
        let reactive_controller = Arc::new(RwLock::new(AudioReactiveController::new()));

        let (sample_tx, sample_rx) = crossbeam_channel::bounded::<Vec<f32>>(64);
        let analysis_rx = analyzer.read().analysis_receiver();

        let samples_processed = Arc::new(AtomicU64::new(0));
        let frames_analyzed = Arc::new(AtomicU64::new(0));
        let dropped_samples = Arc::new(AtomicU64::new(0));
        let is_running = Arc::new(AtomicBool::new(true));

        // Spawn audio processing thread
        let analyzer_clone = analyzer.clone();
        let samples_clone = samples_processed.clone();
        let frames_clone = frames_analyzed.clone();
        let running_clone = is_running.clone();
        let sample_rate = pipeline_config.sample_rate;

        std::thread::Builder::new()
            .name("audio-processor".to_string())
            .spawn(move || {
                let mut timestamp = 0.0;
                let dt = 1.0 / sample_rate as f64;

                while running_clone.load(Ordering::Relaxed) {
                    match sample_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(samples) => {
                            let sample_count = samples.len() as u64;
                            samples_clone.fetch_add(sample_count, Ordering::Relaxed);

                            let mut analyzer = analyzer_clone.write();
                            analyzer.process_samples(&samples, timestamp);
                            frames_clone.fetch_add(1, Ordering::Relaxed);

                            timestamp += samples.len() as f64 * dt;
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("Failed to spawn audio processor thread");

        Self {
            analyzer,
            reactive_controller,
            sample_sender: sample_tx,
            analysis_receiver: analysis_rx,
            config: pipeline_config.clone(),
            analysis_buffer: VecDeque::with_capacity(pipeline_config.analysis_buffer_size),
            audio_position: 0.0,
            samples_processed,
            frames_analyzed,
            dropped_samples,
            is_running,
            last_analysis_time: None,
        }
    }

    /// Send audio samples to the pipeline
    pub fn process_samples(&mut self, samples: &[f32]) {
        match self.sample_sender.try_send(samples.to_vec()) {
            Ok(()) => {
                self.audio_position += samples.len() as f64 / self.config.sample_rate as f64;
            }
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                self.dropped_samples
                    .fetch_add(samples.len() as u64, Ordering::Relaxed);
            }
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => {}
        }
    }

    /// Get the latest audio analysis (with latency compensation)
    pub fn get_analysis(&mut self) -> Option<AudioAnalysis> {
        // Drain all available analyses into buffer
        loop {
            match self.analysis_receiver.try_recv() {
                Ok(analysis) => {
                    let now = Instant::now();
                    self.analysis_buffer.push_back(TimestampedAnalysis {
                        analysis,
                        timestamp: now,
                        audio_position: self.audio_position,
                    });
                    self.last_analysis_time = Some(now);

                    // Keep buffer size bounded
                    while self.analysis_buffer.len() > self.config.analysis_buffer_size {
                        self.analysis_buffer.pop_front();
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        // Apply latency compensation by returning an older analysis
        if self.config.latency_ms > 0.0 {
            let latency_duration = Duration::from_secs_f32(self.config.latency_ms / 1000.0);
            let target_time = Instant::now() - latency_duration;

            // Find the analysis closest to target time
            self.analysis_buffer
                .iter()
                .filter(|a| a.timestamp <= target_time)
                .last()
                .map(|a| a.analysis.clone())
        } else {
            // No latency compensation, return latest
            self.analysis_buffer.back().map(|a| a.analysis.clone())
        }
    }

    /// Get smoothed audio analysis (averaged over buffer)
    pub fn get_smoothed_analysis(&self) -> Option<AudioAnalysis> {
        if self.analysis_buffer.is_empty() {
            return None;
        }

        let count = self.analysis_buffer.len() as f32;
        let mut smoothed = AudioAnalysis::default();

        for item in &self.analysis_buffer {
            smoothed.rms_volume += item.analysis.rms_volume / count;
            smoothed.peak_volume += item.analysis.peak_volume / count;

            // Average frequency bands
            for (i, band) in item.analysis.band_energies.iter().enumerate() {
                if i < smoothed.band_energies.len() {
                    smoothed.band_energies[i] += band / count;
                }
            }
        }

        // Beat detection uses OR logic (any beat in buffer counts)
        smoothed.beat_detected = self
            .analysis_buffer
            .iter()
            .any(|a| a.analysis.beat_detected);

        Some(smoothed)
    }

    /// Set latency compensation
    pub fn set_latency_compensation(&mut self, latency_ms: f32) {
        self.config.latency_ms = latency_ms.max(0.0);
    }

    /// Get latency compensation
    pub fn latency_compensation(&self) -> f32 {
        self.config.latency_ms
    }

    /// Get audio-reactive controller
    pub fn reactive_controller(&self) -> Arc<RwLock<AudioReactiveController>> {
        self.reactive_controller.clone()
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> AudioPipelineStats {
        AudioPipelineStats {
            samples_processed: self.samples_processed.load(Ordering::Relaxed),
            frames_analyzed: self.frames_analyzed.load(Ordering::Relaxed),
            buffer_fill: self.analysis_buffer.len() as f32
                / self.config.analysis_buffer_size as f32,
            estimated_latency_ms: self.estimate_latency(),
            dropped_samples: self.dropped_samples.load(Ordering::Relaxed),
        }
    }

    /// Estimate current audio latency based on buffer state
    fn estimate_latency(&self) -> f32 {
        if let (Some(oldest), Some(newest)) =
            (self.analysis_buffer.front(), self.analysis_buffer.back())
        {
            let buffer_time = newest.timestamp.duration_since(oldest.timestamp);
            buffer_time.as_secs_f32() * 1000.0
        } else {
            0.0
        }
    }

    /// Get current audio position in seconds
    pub fn audio_position(&self) -> f64 {
        self.audio_position
    }

    /// Reset audio position (for seeking)
    pub fn seek(&mut self, position: f64) {
        self.audio_position = position;
        self.analysis_buffer.clear();
    }

    /// Check if pipeline is active
    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

impl Drop for AudioMediaPipeline {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_media_pipeline_creation() {
        let config = AudioConfig::default();
        let pipeline = AudioMediaPipeline::new(config);
        assert_eq!(pipeline.latency_compensation(), 0.0);
    }

    #[test]
    fn test_latency_compensation() {
        let config = AudioConfig::default();
        let mut pipeline = AudioMediaPipeline::new(config);

        pipeline.set_latency_compensation(50.0);
        assert_eq!(pipeline.latency_compensation(), 50.0);
    }
}
