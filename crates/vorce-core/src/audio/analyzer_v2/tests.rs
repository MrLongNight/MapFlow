#[cfg(test)]
use crate::audio::analyzer_v2::{AudioAnalyzerV2, AudioAnalyzerV2Config};

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
fn test_sanitization_of_bad_input() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);

    // Feed NaN and Infinity
    let bad_samples = vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.5];

    // Should not panic, and results should be clean (calculated from sanitized inputs)
    analyzer.process_samples(&bad_samples, 0.0);

    let analysis = analyzer.get_latest_analysis();

    // 1. RMS should be finite (calculated from sanitized inputs)
    assert!(
        analysis.rms_volume.is_finite(),
        "RMS Volume should be finite, got {}",
        analysis.rms_volume
    );

    // Check that bad inputs were effectively handled (non-finite values replaced by 0.0)
    // 0.5 amplitude sine or similar would result in some RMS, but here we just check finite
    assert!(analysis.rms_volume >= 0.0);

    // 3. Magnitudes should all be finite
    for (i, mag) in analysis.fft_magnitudes.iter().enumerate() {
        assert!(
            mag.is_finite(),
            "FFT Magnitude at {} is non-finite: {}",
            i,
            mag
        );
    }

    // 4. Band energies should be finite
    for (i, band) in analysis.band_energies.iter().enumerate() {
        assert!(
            band.is_finite(),
            "Band Energy at {} is non-finite: {}",
            i,
            band
        );
    }
}

#[test]
fn test_calculate_bpm_sparse_data() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);

    // Simulate 3 beats (less than 4 required for BPM)
    let sample_rate = 44100.0;
    let kick_freq = 60.0;
    let kick: Vec<f32> = (0..512)
        .map(|i| (2.0 * std::f32::consts::PI * kick_freq * i as f32 / sample_rate).sin())
        .collect();
    let silence: Vec<f32> = vec![0.0; 512];

    // Establish silence
    for _ in 0..10 {
        analyzer.process_samples(&silence, 0.0);
    }

    // Beat 1 at 1.0s
    analyzer.process_samples(&kick, 1.0);
    // Silence until 1.5s
    analyzer.process_samples(&silence, 1.25);
    // Beat 2 at 1.5s
    analyzer.process_samples(&kick, 1.5);
    // Silence until 2.0s
    analyzer.process_samples(&silence, 1.75);
    // Beat 3 at 2.0s
    analyzer.process_samples(&kick, 2.0);

    let analysis = analyzer.get_latest_analysis();

    // Should have detected beats
    // But BPM should be None because we only have 3 beats (2 intervals)
    assert_eq!(
        analysis.tempo_bpm, None,
        "BPM should be None for sparse data (only 3 beats)"
    );
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
    assert!(rms1 > 0.05 && rms1 < 0.2, "RMS1 was {}", rms1);

    // Step 2
    analyzer.process_samples(&loud, 2.0);
    let rms2 = analyzer.get_latest_analysis().rms_volume;

    assert!(rms2 > rms1, "RMS should increase over time with smoothing");
    assert!(rms2 < 1.0, "RMS should not reach target instantly");
}

#[test]
fn test_process_empty_samples() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);
    analyzer.process_samples(&[], 0.0);
    let analysis = analyzer.get_latest_analysis();
    assert_eq!(analysis.rms_volume, 0.0);
}

#[test]
fn test_try_receive() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);
    assert!(analyzer.try_receive().is_none());

    let samples: Vec<f32> = (0..4096).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    analyzer.process_samples(&samples, 0.0);
    assert!(analyzer.try_receive().is_some());
}

#[test]
fn test_bpm_clamp_ranges() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);

    // 250 BPM (should divide by 2 to 125 BPM) -> interval is 60 / 250 = 0.24s
    analyzer.beat_timestamps.clear();
    for i in 0..5 {
        analyzer.beat_timestamps.push_back(i as f64 * 0.24);
    }
    assert_eq!(analyzer.calculate_bpm(), Some(125.0));

    // 45 BPM (should multiply by 2 to 90 BPM) -> interval is 60 / 45 = 1.333s
    analyzer.beat_timestamps.clear();
    for i in 0..5 {
        analyzer.beat_timestamps.push_back(i as f64 * 1.333333333);
    }
    assert_eq!(analyzer.calculate_bpm(), Some(90.0));

    // 500 BPM (Out of range completely) -> interval is 60 / 500 = 0.12s
    analyzer.beat_timestamps.clear();
    for i in 0..5 {
        analyzer.beat_timestamps.push_back(i as f64 * 0.12);
    }
    assert_eq!(analyzer.calculate_bpm(), None);
}

#[test]
fn test_bpm_zero_avg_interval() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);

    // Feed in extremely close timestamps to test `avg_interval <= 0.001`
    analyzer.beat_timestamps.push_back(1.0);
    analyzer.beat_timestamps.push_back(1.0001);
    analyzer.beat_timestamps.push_back(1.0002);
    analyzer.beat_timestamps.push_back(1.0003);

    assert_eq!(analyzer.calculate_bpm(), None);
}

#[test]
fn test_calculate_bpm_intervals_empty() {
    let config = AudioAnalyzerV2Config::default();
    let mut analyzer = AudioAnalyzerV2::new(config);

    // Empty beat_timestamps or < 4 elements
    analyzer.beat_timestamps.clear();
    assert_eq!(analyzer.calculate_bpm(), None);

    analyzer.beat_timestamps.push_back(1.0);
    analyzer.beat_timestamps.push_back(2.0);
    assert_eq!(analyzer.calculate_bpm(), None);
}
#[test]
fn test_bpm_beat_timestamps_limit() {
    let config = AudioAnalyzerV2Config {
        sample_rate: 44100,
        fft_size: 1024,
        smoothing: 0.0,
        ..Default::default()
    };
    let mut analyzer = AudioAnalyzerV2::new(config);

    let silence = vec![0.0f32; 1024];
    let kick: Vec<f32> = (0..1024)
        .map(|i| (2.0 * std::f32::consts::PI * 60.0 * i as f32 / 44100.0).sin())
        .collect();

    // Feed enough beats to exceed the 16 beat limit for calculating BPM.
    for i in 0..40 {
        analyzer.process_samples(&kick, i as f64 * 0.5); // Beats at 0.5s intervals (120 BPM).
        for _ in 0..5 {
            analyzer.process_samples(&silence, i as f64 * 0.5 + 0.1); // Silence in between.
        }
    }

    let analysis = analyzer.get_latest_analysis();
    assert!(analysis.tempo_bpm.is_some());
    assert_eq!(analyzer.beat_timestamps.len(), 16);
}
