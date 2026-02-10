//! Ableton Link integration (placeholder wrapper)
//!
//! This module wires in the `ableton-link-rs` crate so that trigger nodes
//! can rely on Link tempo information without panicking when the service
//! is unavailable. The lightweight wrapper avoids spawning background tasks
//! until explicitly requested by the caller.

use ableton_link_rs::link::{clock::Clock, tempo::Tempo};

use crate::{error::ControlError, Result};

/// Minimal Ableton Link handle
pub struct AbletonLinkHandle {
    _tempo: Tempo,
    _clock: Clock,
}

impl AbletonLinkHandle {
    /// Create a lightweight handle with default tempo.
    ///
    /// The underlying `ableton-link-rs` types are constructed but no async
    /// tasks are spawned here, keeping initialization cheap.
    pub fn new(default_bpm: f64) -> Result<Self> {
        if !(20.0..=300.0).contains(&default_bpm) {
            return Err(ControlError::LinkError(
                "Tempo must be between 20 and 300 BPM".to_string(),
            ));
        }
        Ok(Self {
            _tempo: Tempo::new(default_bpm),
            _clock: Clock::default(),
        })
    }

    /// Return the configured default tempo.
    pub fn tempo_bpm(&self) -> f64 {
        self._tempo.bpm()
    }

    /// Update the tempo value locally.
    pub fn set_tempo_bpm(&mut self, bpm: f64) -> Result<()> {
        if !(20.0..=300.0).contains(&bpm) {
            return Err(ControlError::LinkError(
                "Tempo must be between 20 and 300 BPM".to_string(),
            ));
        }
        self._tempo = Tempo::new(bpm);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_handle_creation() {
        // Valid creation
        let handle = AbletonLinkHandle::new(120.0).expect("Should create with 120 BPM");
        assert!((handle.tempo_bpm() - 120.0).abs() < f64::EPSILON);

        // Invalid creation (too low)
        let res = AbletonLinkHandle::new(10.0);
        assert!(res.is_err());
        match res {
            Err(ControlError::LinkError(msg)) => {
                assert_eq!(msg, "Tempo must be between 20 and 300 BPM")
            }
            _ => panic!("Expected LinkError"),
        }

        // Invalid creation (too high)
        let res = AbletonLinkHandle::new(350.0);
        assert!(res.is_err());
    }

    #[test]
    fn test_link_handle_set_tempo() {
        let mut handle = AbletonLinkHandle::new(120.0).unwrap();

        // Valid update
        handle.set_tempo_bpm(140.0).expect("Should set tempo");
        assert!((handle.tempo_bpm() - 140.0).abs() < f64::EPSILON);

        // Invalid update
        let res = handle.set_tempo_bpm(10.0);
        assert!(res.is_err());
        assert!(
            (handle.tempo_bpm() - 140.0).abs() < f64::EPSILON,
            "Tempo should not change on error"
        );
    }
}
