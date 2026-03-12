pub mod canvas_ui;
pub mod controller;
pub mod diagnostics;
pub mod draw;
pub mod geometry;
pub mod inspector;
pub mod interaction_logic;
pub mod mesh;
pub mod node_rendering;
pub mod renderer;
pub mod state;
pub mod types;
pub mod utils;

#[derive(Debug, Clone, Copy)]
pub struct ModuleCanvasRenderOptions {
    pub meter_style: crate::config::AudioMeterStyle,
    pub node_animations_enabled: bool,
    pub short_circuit_animation_enabled: bool,
    pub animation_profile: crate::config::AnimationProfile,
    pub reduce_motion_enabled: bool,
}

impl From<&crate::config::UserConfig> for ModuleCanvasRenderOptions {
    fn from(config: &crate::config::UserConfig) -> Self {
        Self {
            meter_style: config.meter_style,
            node_animations_enabled: config.node_animations_enabled,
            short_circuit_animation_enabled: config.short_circuit_animation_enabled,
            animation_profile: config.animation_profile,
            reduce_motion_enabled: config.reduce_motion_enabled,
        }
    }
}

pub use state::ModuleCanvas;
