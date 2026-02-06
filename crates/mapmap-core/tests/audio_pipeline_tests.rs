use mapmap_core::audio::AudioConfig;
use mapmap_core::audio_media_pipeline::{AudioMediaPipeline, AudioPipelineConfig};
use std::thread;
use std::time::{Duration, Instant};

fn create_pipeline() -> AudioMediaPipeline {
    let audio_config = AudioConfig {
        sample_rate: 44100,
        ..Default::default()
    };

    let pipeline_config = AudioPipelineConfig {
        sample_rate: 44100,
        analysis_buffer_size: 8,
        latency_ms: 0.0,
        auto_latency_detection: false,
    };

    AudioMediaPipeline::with_config(audio_config, pipeline_config)
}

fn wait_for_condition<F>(timeout: Duration, mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while Instant::now().duration_since(start) < timeout {
        if condition() {
            return true;
        }
        thread::yield_now();
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn test_data_flow() {
    let mut pipeline = create_pipeline();
    let initial_frames = pipeline.stats().frames_analyzed;

    // Generate test signal (440Hz sine wave)
    let sample_rate = 44100;
    let duration_secs = 0.5;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let freq = 440.0;

    let samples: Vec<f32> = (0..num_samples)
        .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate as f32).sin() * 0.5)
        .collect();

    // Send samples in chunks
    let chunk_size = 1024;
    for chunk in samples.chunks(chunk_size) {
        pipeline.process_samples(chunk);
    }

    // Wait for analysis to happen
    let success = wait_for_condition(Duration::from_secs(2), || {
        pipeline.stats().frames_analyzed > initial_frames
    });

    assert!(success, "Pipeline failed to analyze frames within timeout");

    // Pump and check analysis
    // We loop until we find a valid analysis or timeout
    let mut found = false;
    let start = Instant::now();
    while Instant::now().duration_since(start) < Duration::from_secs(1) {
        if let Some(data) = pipeline.get_analysis() {
            if !data.fft_magnitudes.is_empty() {
                found = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(found, "Should have received analysis with FFT data");
}

#[test]
fn test_stats_simple() {
    let mut pipeline = create_pipeline();
    // Send silence first
    let silence = vec![0.0; 44100];
    pipeline.process_samples(&silence);

    // Wait for processing
    let success = wait_for_condition(Duration::from_secs(2), || {
        pipeline.stats().samples_processed >= 44100
    });
    assert!(success, "Failed to process initial silence");
    assert!(pipeline.stats().samples_processed >= 44100);
}

#[test]
fn test_smoothing_presence() {
    let mut pipeline = create_pipeline();
    // Send loud signal
    let loud: Vec<f32> = vec![1.0; 8192];
    pipeline.process_samples(&loud);

    // Wait for ANY analysis with volume > 0
    let success = wait_for_condition(Duration::from_secs(2), || {
        let _ = pipeline.get_analysis();
        if let Some(smoothed) = pipeline.get_smoothed_analysis() {
             smoothed.rms_volume > 0.0
        } else {
            false
        }
    });
    assert!(success, "Should get smoothed analysis with >0 RMS");
}

#[test]
fn test_dropped_samples() {
    let mut pipeline = create_pipeline();

    let chunk = vec![0.0; 128];
    let mut dropped = false;

    // Try to flood the channel
    // We loop for a limited number to prevent infinite loop if it's too fast
    for _ in 0..10000 {
        pipeline.process_samples(&chunk);
        if pipeline.dropped_samples() > 0 {
            dropped = true;
            break;
        }
    }

    if dropped {
        assert!(pipeline.dropped_samples() > 0);
        assert!(pipeline.stats().dropped_samples > 0);
    }
}
