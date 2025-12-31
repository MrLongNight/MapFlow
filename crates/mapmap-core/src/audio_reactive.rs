//! Data structures for audio-reactive components.

/// Live audio data for trigger nodes
#[derive(Debug, Clone, Default)]
pub struct AudioTriggerData {
    /// 9 frequency band energies [SubBass, Bass, LowMid, Mid, HighMid, UpperMid, Presence, Brilliance, Air]
    pub band_energies: [f32; 9],
    /// RMS volume (0.0 - 1.0)
    pub rms_volume: f32,
    /// Peak volume (0.0 - 1.0)
    pub peak_volume: f32,
    /// Beat detected this frame
    pub beat_detected: bool,
    /// Beat strength (0.0 - 1.0)
    pub beat_strength: f32,
    /// Estimated BPM (optional)
    pub bpm: Option<f32>,
}
