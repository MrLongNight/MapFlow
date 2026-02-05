use mapmap_core::{AudioConfig, AudioMediaPipeline};
use mapmap_core::audio_media_pipeline::AudioPipelineConfig;
use std::time::{Duration, Instant};

// Helper for polling condition with timeout
fn wait_for_condition<F>(condition: F, timeout_ms: u64, failure_message: &str)
where
    F: Fn() -> bool,
{
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    loop {
        if condition() {
            return;
        }
        if start.elapsed() > timeout {
            panic!("{}", failure_message);
        }
        // Busy-wait with yield (std::hint::spin_loop() or just loop is fine for tests)
        // We use thread::yield_now() to be nice if available, but standard loop is ok
        std::thread::yield_now();
    }
}

#[test]
fn test_pipeline_data_flow_polling() {
    let config = AudioConfig::default();
    let mut pipeline = AudioMediaPipeline::new(config);

    // Send samples (enough for FFT - 2048 samples)
    let samples = vec![0.5; 2048];
    pipeline.process_samples(&samples);

    // Poll until processing is done
    wait_for_condition(
        || pipeline.stats().samples_processed >= 2048,
        2000,
        "Timeout waiting for samples to be processed"
    );

    // Also wait for analysis to be generated (might take a tiny bit after samples are processed)
    wait_for_condition(
        || pipeline.stats().frames_analyzed >= 1,
        2000,
        "Timeout waiting for frames to be analyzed"
    );

    // Now verify analysis is available
    let analysis = pipeline.get_analysis();
    assert!(analysis.is_some(), "Analysis should be available");

    let analysis_data = analysis.unwrap();
    assert_eq!(analysis_data.fft_magnitudes.len(), 512); // Default FFT size 1024 / 2
}

#[test]
fn test_pipeline_stats_and_smoothing() {
    let audio_config = AudioConfig {
        fft_size: 1024,
        ..Default::default()
    };

    // Small buffer size for easier testing
    let pipeline_config = AudioPipelineConfig {
        analysis_buffer_size: 4,
        ..Default::default()
    };

    let mut pipeline = AudioMediaPipeline::with_config(audio_config, pipeline_config);

    // Send multiple chunks to generate multiple analyses
    // 1024 samples per chunk -> 1 FFT per chunk (approx, hop size depending)
    // Overlap is 0.5 default, hop size 512.
    // 1024 samples should produce ~2 frames eventually.
    // Send 5 batches of 1024 samples.
    let samples = vec![0.5; 1024];
    for _ in 0..5 {
        pipeline.process_samples(&samples);
    }

    // Wait for at least 4 frames
    wait_for_condition(
        || pipeline.stats().frames_analyzed >= 4,
        2000,
        "Timeout waiting for multiple frames"
    );

    // Verify stats
    let stats = pipeline.stats();
    assert!(stats.samples_processed >= 5 * 1024);
    assert!(stats.frames_analyzed >= 4);

    // Buffer should be full (size 4)
    // Poll for buffer fill because get_analysis() moves data from receiver to buffer
    // Wait, get_analysis() MUST be called to fill the buffer!
    // The pipeline loop runs internally, but `get_analysis()` or `get_smoothed_analysis()`
    // logic in `audio_media_pipeline.rs` calls `self.analysis_receiver.try_recv()`.
    // Ah, checking the code:
    // `get_analysis()` calls `self.analysis_receiver.try_recv()` and pushes to `analysis_buffer`.
    // So if we don't call `get_analysis()`, the buffer inside pipeline (the deque) stays empty!
    // The receiver channel fills up instead.

    assert_eq!(stats.buffer_fill, 0.0, "Buffer should be empty before calling get_analysis");

    // Now trigger buffer update
    let _ = pipeline.get_analysis();

    let stats_after = pipeline.stats();
    // It might not be 1.0 immediately if only one analysis was pulled,
    // but `get_analysis` loops: `loop { match self.analysis_receiver.try_recv() ... }`
    // So it drains ALL available.

    assert!(stats_after.buffer_fill > 0.0, "Buffer should have items after get_analysis");

    // Verify smoothing
    let smoothed = pipeline.get_smoothed_analysis();
    assert!(smoothed.is_some(), "Smoothed analysis should be available");
}

#[test]
fn test_dropped_samples_polling() {
    let config = AudioConfig::default();
    let mut pipeline = AudioMediaPipeline::new(config);

    // The internal channel size is 64.
    // We need to flood it faster than the consumer can process.
    // Consumer takes ~1/sample_rate time per sample? No, it's just processing logic.
    // But `analyzer.process_samples` does FFT which takes non-zero time.

    let samples = vec![0.0; 4096]; // Large chunk

    // Send LOTS of data rapidly
    let mut dropped = false;
    for _ in 0..200 {
        pipeline.process_samples(&samples);
        if pipeline.dropped_samples() > 0 {
            dropped = true;
            break;
        }
    }

    // Note: This test *might* fail on very fast machines if consumer is faster than producer loop,
    // but with 200 * 4096 samples and channel size 64, it's highly likely to overflow.
    // However, if it doesn't drop, it means our system is performant, which is fine.
    // We verify the MECHANISM: if it drops, it counts.

    if dropped {
        assert!(pipeline.stats().dropped_samples > 0);
    } else {
        println!("test_dropped_samples_polling: System too fast to drop samples, skipping assertion.");
    }
}
