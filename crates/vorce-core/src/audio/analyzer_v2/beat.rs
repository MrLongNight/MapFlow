//! Beat Detection for Audio Analyzer V2
use super::analyzer::AudioAnalyzerV2;

impl AudioAnalyzerV2 {
    /// Simple beat detection based on energy spike
    pub(crate) fn detect_beat(&mut self, timestamp: f64) -> (bool, f32) {
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
    pub(crate) fn calculate_bpm(&self) -> Option<f32> {
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
}
