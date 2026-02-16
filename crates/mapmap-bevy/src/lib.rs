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
        info!("Initializing Bevy integration (Full Asset Mode)...");

        let mut app = App::new();

        // Load essential plugins for 3D assets without opening a window
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        // Since we can't easily find the matching bevy_hierarchy version on crates.io,
        // we rely on the fact that bevy usually re-exports it.
        // If "bevy::hierarchy" is missing, it might be under a feature flag or re-exported differently.
        // In 0.16, if `default-features=false`, `bevy_hierarchy` might be excluded?
        // Let's try skipping explicit HierarchyPlugin addition.
        // `TransformPlugin` might add it as a dependency if needed, or it might panic if missing.
        // If `TransformPlugin` doesn't add it, we are in trouble without the dependency.
        // However, `bevy` feature "bevy_hierarchy" doesn't exist explicitly in Cargo.toml.
        // Let's try to assume it's included in `MinimalPlugins` or another plugin, OR
        // simply remove it and see if it compiles/runs.
        // If `bevy::hierarchy` module is missing, we can't name it.
        // app.add_plugins(bevy::hierarchy::HierarchyPlugin);
        app.add_plugins(bevy::transform::TransformPlugin);

        // Load PBR infrastructure so StandardMaterial and Mesh assets exist
        // We use the headless configuration parts of PbrPlugin
        app.add_plugins(bevy::pbr::PbrPlugin { ..default() });
        app.add_plugins(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings::default(),
            ),
            ..default()
        });
        app.add_plugins(bevy::core_pipeline::CorePipelinePlugin);

        // Register resources
        app.init_resource::<AudioInputResource>();
        app.init_resource::<BevyNodeMapping>();

        // Register components
        app.register_type::<AudioReactive>();
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();
        app.register_type::<Bevy3DShape>();

        // Re-enable all systems now that assets should be present
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                camera_control_system,
                text_3d_system,
                shape_system,
            ),
        );

        Self { app }
    }

    pub fn update(&mut self, audio_data: &mapmap_core::audio_reactive::AudioTriggerData) {
        if let Some(mut res) = self
            .app
            .world_mut()
            .get_resource_mut::<AudioInputResource>()
        {
            res.band_energies = audio_data.band_energies;
            res.rms_volume = audio_data.rms_volume;
            res.peak_volume = audio_data.peak_volume;
            res.beat_detected = audio_data.beat_detected;
        }
        self.app.update();
    }

    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        // Dummy for now, real readback needs RenderDevice synchronization
        Some((vec![0, 0, 0, 0], 1, 1))
    }

    pub fn apply_graph_state(&mut self, _module: &mapmap_core::module::MapFlowModule) {
        // Logic for syncing Bevy entities with MapFlow graph
    }
}
