use mapmap_core::audio::AudioConfig;
use mapmap_core::audio_media_pipeline::{AudioMediaPipeline, AudioPipelineConfig};
use std::time::{Duration, Instant};

/// Helper to poll until a condition is met or timeout occurs.
/// Uses the "polling atomics pattern" to avoid flaky sleeps.
fn poll_until<F>(timeout: Duration, mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        // Yield to avoid burning CPU while waiting
        std::thread::yield_now();
        // Small sleep to be nice to the scheduler, but relies on polling
        std::thread::sleep(Duration::from_millis(1));
    }
    false
}

#[test]
fn test_audio_pipeline_data_flow() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig {
        sample_rate: 44100,
        latency_ms: 0.0,
        analysis_buffer_size: 8,
        auto_latency_detection: false,
    };

    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Initial stats should be zero
    let stats = pipeline.stats();
    assert_eq!(stats.samples_processed, 0);
    assert_eq!(stats.frames_analyzed, 0);

    // Send some samples
    // 1024 samples should be enough to trigger one analysis frame (FFT size 1024)
    let samples = vec![0.5; 1024];
    pipeline.process_samples(&samples);

    // Poll until samples are processed
    let success = poll_until(Duration::from_secs(2), || {
        pipeline.stats().samples_processed >= 1024
    });

    assert!(success, "Timed out waiting for samples to be processed");

    // Poll until a frame is analyzed
    let success = poll_until(Duration::from_secs(2), || {
        pipeline.stats().frames_analyzed >= 1
    });

    assert!(success, "Timed out waiting for frame analysis");

    // Verify we can get analysis
    let analysis = pipeline.get_analysis();
    assert!(analysis.is_some(), "Should have produced an analysis");

    let stats = pipeline.stats();
    assert!(stats.samples_processed >= 1024);
    assert!(stats.frames_analyzed >= 1);
}

#[test]
fn test_audio_pipeline_stats_smoothing() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig {
        sample_rate: 44100,
        latency_ms: 0.0,
        analysis_buffer_size: 4, // Small buffer for testing
        auto_latency_detection: false,
    };

    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Send enough data to fill the buffer
    // FFT size is 1024, buffer size 4 -> need at least 4 * 1024 samples (plus hop size considerations)
    // Actually, with overlap 0.5 (default), hop size is 512.
    // So to get 4 frames, we need:
    // Frame 1: 1024 samples
    // Frame 2: +512 samples
    // Frame 3: +512 samples
    // Frame 4: +512 samples
    // Total ~2560 samples. Let's send 4096 to be safe.
    let chunk_size = 512;
    let chunks = 8;

    for _ in 0..chunks {
        pipeline.process_samples(&vec![0.5; chunk_size]);
        // Small delay to ensure they don't all get batched weirdly, though queue should handle it
        std::thread::sleep(Duration::from_millis(1));
    }

    // Poll until we have enough frames analyzed
    let success = poll_until(Duration::from_secs(2), || {
        pipeline.stats().frames_analyzed >= 4
    });
    assert!(success, "Timed out waiting for multiple frames");

    // Pull analyses to fill the buffer
    // We need to call get_analysis() to move from receiver to buffer
    // Wait until buffer fills
    let success = poll_until(Duration::from_secs(2), || {
        pipeline.get_analysis(); // Drain receiver
        pipeline.stats().buffer_fill >= 0.5 // At least half full
    });
    assert!(success, "Buffer did not fill as expected");

    // Now test smoothing
    let smoothed = pipeline.get_smoothed_analysis();
    assert!(smoothed.is_some());

    let analysis = smoothed.unwrap();
    // Since we sent constant 0.5 samples, RMS should be consistent
    assert!(analysis.rms_volume > 0.0);
}

#[test]
fn test_dropped_samples() {
    let audio_config = AudioConfig::default();
    let pipeline_config = AudioPipelineConfig::default();
    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // The internal channel size is 64.
    // If we send > 64 chunks without the consumer keeping up, we should drop samples.
    // The consumer is running in a separate thread.
    // To ensure drops, we can send very fast.

    let chunk = vec![0.0; 128];
    let mut sent_chunks = 0;

    // Try to overflow the channel
    for _ in 0..200 {
        pipeline.process_samples(&chunk);
        sent_chunks += 1;
    }

    // It's possible the consumer is fast enough to not drop anything on some machines,
    // but typically 200 chunks sent in a tight loop should overflow a size 64 channel
    // if the processing (FFT) takes any time.

    // We expect the sum of processed and dropped samples to equal what we sent.

    let expected_total_samples = sent_chunks * 128;

    // Wait for consumer to finish whatever it has
    let success = poll_until(Duration::from_secs(2), || {
        let s = pipeline.stats();
        // samples_processed + dropped_samples should equal sent
        // Note: this assumes no inflight samples in channel when we check,
        // or rather, we wait until they are processed.

        let processed = s.samples_processed;
        let dropped = s.dropped_samples;

        processed + dropped == (expected_total_samples as u64)
    });

    let stats = pipeline.stats();
    assert!(success,
        "Accounting mismatch: processed({}) + dropped({}) != sent({})",
        stats.samples_processed, stats.dropped_samples, expected_total_samples);

    // We can't strictly assert dropped > 0 because on a fast machine with slow sender
    // (e.g. debugging/instrumentation) it might keep up.
    // But in a real "test dropped samples" scenario we'd want to see drops.
    // For stability, we just check accounting.
}
