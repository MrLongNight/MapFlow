//! FFT Processing for Audio Analyzer V2
use super::analyzer::AudioAnalyzerV2;
use num_complex::Complex;
use tracing::trace;

impl AudioAnalyzerV2 {
    /// Perform FFT on current buffer
    pub(crate) fn perform_fft(&mut self) {
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
        #[allow(clippy::manual_is_multiple_of)]
        if self.fft_count % 100 == 0 {
            trace!(
                "FFT #{}: bands={:?}",
                self.fft_count,
                &self.smoothed_bands[..3]
            );
        }
    }

    /// Calculate energy in each of 9 frequency bands
    pub(crate) fn update_band_energies(&mut self) {
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
}
