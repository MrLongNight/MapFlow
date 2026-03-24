//!
//! Layer composition types.
//!

use serde::{Deserialize, Serialize};

/// Composition metadata and master controls (Phase 1, Month 5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
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

#[cfg(test)]
mod tests_guardian {
    use super::*;

    #[test]
    fn test_composition_default_values() {
        let comp = Composition::default();
        assert_eq!(comp.name, "Untitled Composition");
        assert_eq!(comp.description, "");
        assert_eq!(comp.master_opacity, 1.0);
        assert!(!comp.master_blackout);
        assert_eq!(comp.master_speed, 1.0);
        assert_eq!(comp.size, (1920, 1080));
        assert_eq!(comp.frame_rate, 60.0);
    }

    #[test]
    fn test_composition_new_initialization() {
        let comp = Composition::new("Test Composition", (800, 600), 30.0);
        assert_eq!(comp.name, "Test Composition");
        assert_eq!(comp.description, "");
        assert_eq!(comp.master_opacity, 1.0);
        assert!(!comp.master_blackout);
        assert_eq!(comp.master_speed, 1.0);
        assert_eq!(comp.size, (800, 600));
        assert_eq!(comp.frame_rate, 30.0);
    }

    #[test]
    fn test_composition_with_description_builder() {
        let comp = Composition::default().with_description("A test composition description");
        assert_eq!(comp.description, "A test composition description");
    }

    #[test]
    fn test_composition_set_master_opacity_bounds() {
        let mut comp = Composition::default();

        comp.set_master_opacity(0.5);
        assert_eq!(comp.master_opacity, 0.5);

        comp.set_master_opacity(-1.0);
        assert_eq!(comp.master_opacity, 0.0); // Clamped to 0.0

        comp.set_master_opacity(2.0);
        assert_eq!(comp.master_opacity, 1.0); // Clamped to 1.0
    }

    #[test]
    fn test_composition_set_master_speed_bounds() {
        let mut comp = Composition::default();

        comp.set_master_speed(5.0);
        assert_eq!(comp.master_speed, 5.0);

        comp.set_master_speed(0.0);
        assert_eq!(comp.master_speed, 0.1); // Clamped to 0.1

        comp.set_master_speed(15.0);
        assert_eq!(comp.master_speed, 10.0); // Clamped to 10.0
    }

    #[test]
    fn test_composition_backward_compatibility() {
        // Missing master_blackout and master_speed
        let json = r#"{
            "name": "Old Comp",
            "description": "Old",
            "master_opacity": 0.8,
            "size": [1920, 1080],
            "frame_rate": 60.0
        }"#;

        let comp: Composition =
            serde_json::from_str(json).expect("Should deserialize old composition");
        assert_eq!(comp.name, "Old Comp");
        assert_eq!(comp.master_opacity, 0.8);
        assert!(!comp.master_blackout); // Defaulted
        assert_eq!(comp.master_speed, 1.0); // Defaulted
    }
}
