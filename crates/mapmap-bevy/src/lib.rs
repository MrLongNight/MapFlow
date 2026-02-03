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

        // Minimal plugins for headless/embedded execution
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(bevy::scene::ScenePlugin);
        app.add_plugins(bevy::render::RenderPlugin::default());
        app.add_plugins(bevy::pbr::PbrPlugin::default()); // Includes StandardMaterial

        // Register Extensions
        app.add_plugins(bevy_enoki::EnokiPlugin);
        app.add_plugins(bevy_mod_outline::OutlinePlugin);

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyRenderOutput>();

        // Setup ExtractResourcePlugin to sync BevyRenderOutput to Render App
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            BevyRenderOutput,
        >::default());

        // Register components
        app.register_type::<AudioReactive>();

        // Register systems
        app.add_systems(Startup, setup_3d_scene);
        app.add_systems(Update, (print_status_system, audio_reaction_system));

        // Add readback system to the RENDER APP, not the main app
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

    /// Get the rendered image data.
    /// Returns (data, width, height).
    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        // Access shared data from resource
        if let Some(render_output) = self.app.world().get_resource::<BevyRenderOutput>() {
            if let Ok(lock) = render_output.last_frame_data.lock() {
                if let Some(data) = &*lock {
                    return Some((data.clone(), render_output.width, render_output.height));
                }
            }
        }

        None
    }
}

impl bevy::render::extract_resource::ExtractResource for BevyRenderOutput {
    type Source = Self;
    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

fn print_status_system() {
    // Placeholder system
}
