//!
//! Layer composition types.
//!

use serde::{Deserialize, Serialize};

/// Composition metadata and master controls (Phase 1, Month 5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Composition {
    /// Composition name
    pub name: String,
    /// Optional description
    pub description: String,
    /// Master opacity (M) - global opacity multiplier (Phase 1, Month 4)
    pub master_opacity: f32,
    /// Master blackout (B) - if true, everything is black (Phase 1, Month 6)
    pub master_blackout: bool,
    /// Master speed (S) - global speed multiplier (Phase 1, Month 5)
    pub master_speed: f32,
    /// Composition size in pixels (width, height)
    pub size: (u32, u32),
    /// Frame rate (FPS) for playback
    pub frame_rate: f32,
}

impl Default for Composition {
    fn default() -> Self {
        Self {
            name: "Untitled Composition".to_string(),
            description: String::new(),
            master_opacity: 1.0,
            master_blackout: false,
            master_speed: 1.0,
            size: (1920, 1080),
            frame_rate: 60.0,
        }
    }
}

impl Composition {
    /// Create a new composition
    pub fn new(name: impl Into<String>, size: (u32, u32), frame_rate: f32) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            master_opacity: 1.0,
            master_blackout: false,
            master_speed: 1.0,
            size,
            frame_rate,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set master opacity (clamped 0.0-1.0)
    pub fn set_master_opacity(&mut self, opacity: f32) {
        self.master_opacity = opacity.clamp(0.0, 1.0);
    }

    /// Set master speed (clamped 0.1-10.0)
    pub fn set_master_speed(&mut self, speed: f32) {
        self.master_speed = speed.clamp(0.1, 10.0);
    }
}
