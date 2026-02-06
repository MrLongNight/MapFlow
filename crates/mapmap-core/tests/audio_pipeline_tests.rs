use mapmap_core::audio::AudioConfig;
use mapmap_core::audio_media_pipeline::{AudioMediaPipeline, AudioPipelineConfig};
use std::thread;
use std::time::{Duration, Instant};

fn create_test_samples(count: usize) -> Vec<f32> {
    // Generate a simple sine wave
    (0..count).map(|i| (i as f32 * 0.1).sin()).collect()
}

fn wait_for_condition<F>(timeout: Duration, mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        thread::yield_now();
        thread::sleep(Duration::from_millis(1));
    }
    false
}

#[test]
fn test_data_flow_analysis() {
    let audio_config = AudioConfig::default();
    let mut pipeline = AudioMediaPipeline::new(audio_config);

    // Feed some samples
    // Default FFT size is 1024. We need at least that many to get an analysis.
    // Feed enough samples to trigger multiple analyses.
    let chunk_size = 512;
    let chunks = 10;

    for _ in 0..chunks {
        pipeline.process_samples(&create_test_samples(chunk_size));
        thread::sleep(Duration::from_millis(10)); // Simulate real-time feeding
    }

    // Poll for analysis
    let success = wait_for_condition(Duration::from_secs(2), || {
        if let Some(analysis) = pipeline.get_analysis() {
            // Check if we got valid data
            analysis.rms_volume > 0.0
        } else {
            false
        }
    });

    assert!(success, "Failed to receive valid audio analysis");
}

#[test]
fn test_stats_and_smoothing() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig {
        analysis_buffer_size: 4,
        ..AudioPipelineConfig::default()
    };
    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Feed enough data to fill buffer
    let chunk_size = 512;

    // We need to feed continuously to keep the buffer populated
    let start = Instant::now();
    let mut _samples_sent = 0;

    while start.elapsed() < Duration::from_secs(1) {
        pipeline.process_samples(&create_test_samples(chunk_size));
        _samples_sent += chunk_size;

        // Try to get analysis to populate internal buffer
        let _ = pipeline.get_analysis();

        thread::sleep(Duration::from_millis(5));

        if pipeline.stats().buffer_fill >= 1.0 {
            break;
        }
    }

    let stats = pipeline.stats();
    assert!(stats.samples_processed > 0, "No samples processed");
    assert!(stats.frames_analyzed > 0, "No frames analyzed");

    // Check smoothed analysis
    if let Some(smoothed) = pipeline.get_smoothed_analysis() {
        assert!(smoothed.rms_volume >= 0.0);
        // It might be 0.0 if not enough energy, but our sine wave has energy.
        assert!(smoothed.rms_volume > 0.0, "Smoothed RMS should be > 0");
    } else {
        panic!("Smoothed analysis returned None but buffer should be full");
    }
}

#[test]
fn test_dropped_samples() {
    let audio_config = AudioConfig::default();
    let mut pipeline = AudioMediaPipeline::new(audio_config);

    // Flood the pipeline
    // The internal channel size is 64.
    // We send more than 64 chunks very quickly.

    let chunk_size = 512;
    let chunks_to_send = 200;

    for _ in 0..chunks_to_send {
        pipeline.process_samples(&create_test_samples(chunk_size));
        // No sleep here, we want to overflow
    }

    // Wait a bit for the processing thread to (fail to) catch up and stats to update
    thread::sleep(Duration::from_millis(100));

    let dropped = pipeline.dropped_samples();
    assert!(dropped > 0, "Expected dropped samples, got {}", dropped);
}
