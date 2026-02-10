//! Bevy integration for MapFlow.
//!
//! This crate provides a bridge between MapFlow's orchestration and the Bevy game engine
//! for high-performance 3D rendering and audio reactivity.

pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;

/// Struct to manage the Bevy application instance.
pub struct BevyRunner {
    app: App,
}

impl Default for BevyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BevyRunner {
    /// Creates a new BevyRunner instance.
    pub fn new() -> Self {
        info!("Initializing Bevy integration...");

        let mut app = App::new();

        // Use DefaultPlugins with standard rendering creation
        app.add_plugins(
            DefaultPlugins
                .set(bevy::render::RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::log::LogPlugin>(),
        );

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyRenderOutput>();
        app.init_resource::<BevyNodeMapping>();

        // Setup ExtractResourcePlugin to sync BevyRenderOutput to Render App
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            BevyRenderOutput,
        >::default());

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<BevyAtmosphere>();
        app.register_type::<BevyHexGrid>();
        app.register_type::<BevyParticles>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(Update, (audio_reaction_system, hex_grid_system));

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(bevy::render::Render, frame_readback_system);

        Self { app }
    }

    /// Update the Bevy world manually.
    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
        // Update resource from input data
        let mut res = self.app.world_mut().resource_mut::<AudioInputResource>();
        res.band_energies = audio_data.band_energies;
        res.rms_volume = audio_data.rms_volume;
        res.peak_volume = audio_data.peak_volume;
        res.beat_detected = audio_data.beat_detected;

        // Run schedule
        self.app.update();
    }

    /// Retrieve the rendered frame data (BGRA8).
    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        let output = self.app.world().get_resource::<BevyRenderOutput>()?;
        let data_guard = output.last_frame_data.lock().ok()?;
        let data = data_guard.clone()?;
        Some((data, output.width, output.height))
    }

    /// Sync the MapFlow module graph state to Bevy entities.
    pub fn apply_graph_state(&mut self, _module: &mapmap_core::module::MapFlowModule) {
        let _mapping = self.app.world_mut().resource_mut::<BevyNodeMapping>();
        // TODO: Implement full graph sync (spawn/despawn entities based on module parts)
        // For now, this is a placeholder to satisfy the API.
        // Real implementation would iterate module.parts, check mapping, spawn/update components.
    }
}

fn print_status_system() {
    // Placeholder system
}
