//! Ableton Link integration (placeholder wrapper)
//!
//! This module wires in the `ableton-link-rs` crate so that trigger nodes
//! can rely on Link tempo information without panicking when the service
//! is unavailable. The lightweight wrapper avoids spawning background tasks
//! until explicitly requested by the caller.

use std::marker::PhantomData;

use ableton_link_rs::link::{clock::Clock, tempo::Tempo};

use crate::{error::ControlError, Result};

/// Minimal Ableton Link handle
pub struct AbletonLinkHandle {
    _tempo: Tempo,
    _clock: Clock,
    _marker: PhantomData<()>,
}

impl AbletonLinkHandle {
    /// Create a lightweight handle with default tempo.
    ///
    /// The underlying `ableton-link-rs` types are constructed but no async
    /// tasks are spawned here, keeping initialization cheap.
    pub fn new(default_bpm: f64) -> Result<Self> {
        let tempo = Tempo::new(default_bpm);
        let clock = Clock::default();
        Ok(Self {
            _tempo: tempo,
            _clock: clock,
            _marker: PhantomData,
        })
    }

    /// Return the configured default tempo.
    pub fn tempo_bpm(&self) -> f64 {
        self._tempo.bpm()
    }

    /// Update the tempo value locally.
    pub fn set_tempo_bpm(&mut self, bpm: f64) -> Result<()> {
        if bpm <= 0.0 {
            return Err(ControlError::InvalidParameter(
                "Tempo must be positive".to_string(),
            ));
        }
        self._tempo.set_bpm(bpm);
        Ok(())
    }
}
