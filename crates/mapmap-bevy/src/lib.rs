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
        info!("Initializing Bevy integration (Final Stable Mode)...");

        let mut app = App::new();

        // Use MinimalPlugins to avoid any GPU/Windowing dependencies
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        
        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();
        app.register_type::<Bevy3DShape>();

        // IMPORTANT: We only add systems that DON'T depend on Mesh/Material assets
        // which are missing because we don't load the full PbrPlugin yet.
        app.add_systems(Update, (
            audio_reaction_system,
            // text_3d_system, // Depends on Assets<Mesh>
            camera_control_system,
            // shape_system,   // Depends on Assets<Mesh>
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
        // Dummy 1x1 to satisfy the pipeline
        Some((vec![0, 0, 0, 0], 1, 1))
    }

    pub fn apply_graph_state(&mut self, _module: &mapmap_core::module::MapFlowModule) {
        // Safe no-op
    }
}
