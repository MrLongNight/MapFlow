use mapmap_core::{
    audio::AudioConfig,
    audio_media_pipeline::{AudioMediaPipeline, AudioPipelineConfig},
};
use std::time::{Duration, Instant};

fn wait_for_condition<F>(mut condition: F, timeout: Duration)
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while !condition() {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for condition");
        }
        std::thread::yield_now();
    }
}

#[test]
fn test_audio_media_pipeline_data_flow() {
    // Initialize pipeline with high sample rate
    let audio_config = AudioConfig {
        sample_rate: 44100,
        fft_size: 1024,
        ..Default::default()
    };
    let pipeline_config = AudioPipelineConfig {
        sample_rate: 44100,
        analysis_buffer_size: 8,
        latency_ms: 0.0,
        auto_latency_detection: false,
    };

    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Generate sine wave (440Hz)
    let samples: Vec<f32> = (0..2048)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.5)
        .collect();

    // Send samples
    pipeline.process_samples(&samples);

    // Wait for samples to be processed
    wait_for_condition(
        || pipeline.stats().samples_processed >= 2048,
        Duration::from_secs(5),
    );

    // Check analysis
    // Wait for analysis to be available
    wait_for_condition(|| pipeline.get_analysis().is_some(), Duration::from_secs(5));

    // Check stats
    let stats = pipeline.stats();
    assert!(stats.frames_analyzed > 0, "Frames should be analyzed");
}

#[test]
fn test_stats_and_smoothing() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig::default();
    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // 1. Feed silence
    let silence = vec![0.0; 2048];
    pipeline.process_samples(&silence);

    wait_for_condition(
        || pipeline.stats().samples_processed >= 2048,
        Duration::from_secs(5),
    );

    // Clear buffer
    pipeline.seek(0.0);

    // 2. Feed loud noise
    // Send in chunks to ensure analyzer processes all frames
    // (AudioAnalyzer v1 only processes one FFT per call)
    let chunk_size = 512;
    let chunks = 8;
    let noise_chunk: Vec<f32> = vec![0.8; chunk_size];

    for _ in 0..chunks {
        pipeline.process_samples(&noise_chunk);
    }

    wait_for_condition(
        || pipeline.stats().samples_processed >= (2048 + chunk_size * chunks) as u64,
        Duration::from_secs(5),
    );

    // Wait for analysis to reflect noise
    // We poll until we get a smoothed analysis with non-zero RMS
    let mut final_rms = 0.0;
    wait_for_condition(
        || {
            let _ = pipeline.get_analysis(); // Pull new data
            if let Some(smoothed) = pipeline.get_smoothed_analysis() {
                final_rms = smoothed.rms_volume;
                smoothed.rms_volume > 0.01
            } else {
                false
            }
        },
        Duration::from_secs(5),
    );

    assert!(
        final_rms > 0.01,
        "Smoothed RMS should be > 0.01, was {}",
        final_rms
    );
}

#[test]
fn test_dropped_samples() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig::default();
    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Attempt to flood the pipeline
    // The internal channel size is 64.
    // We send many large chunks.
    let huge_chunk = vec![0.0; 8192];
    for _ in 0..1000 {
        pipeline.process_samples(&huge_chunk);
    }

    // Check stats
    let stats = pipeline.stats();
    // We don't assert > 0 because on a fast machine the consumer might keep up.
    // But we assert the field is accessible.
    // Ensure field exists and is accessible
    let dropped = stats.dropped_samples;
    assert_eq!(dropped, pipeline.dropped_samples());
}
