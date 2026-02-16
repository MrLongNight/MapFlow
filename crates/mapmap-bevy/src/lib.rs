//! Bevy integration for MapFlow.

pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;

pub struct BevyRunner {
    app: App,
}

impl Default for BevyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BevyRunner {
    pub fn new() -> Self {
        info!("Initializing Bevy integration (Safe Mode)...");

        let mut app = App::new();

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::render::RenderPlugin>() // Disable default render flow
        );

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();
        app.register_type::<Bevy3DShape>();

        // Register systems
        app.add_systems(Update, (
            audio_reaction_system,
            text_3d_system,
            camera_control_system,
            shape_system,
        ));

        Self { app }
    }

    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
        if let Some(mut res) = self.app.world_mut().get_resource_mut::<AudioInputResource>() {
            res.band_energies = audio_data.band_energies;
            res.rms_volume = audio_data.rms_volume;
            res.peak_volume = audio_data.peak_volume;
            res.beat_detected = audio_data.beat_detected;
        }
        self.app.update();
    }

    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        // Return dummy data for now to prevent crash
        None
    }

    pub fn apply_graph_state(&mut self, _module: &mapmap_core::module::MapFlowModule) {
        // Safe no-op for now
    }
}
