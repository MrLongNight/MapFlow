//! Bevy integration for MapFlow.
//!
//! This crate provides a bridge between MapFlow's orchestration and the Bevy game engine
//! for high-performance 3D rendering and audio reactivity.

pub mod components;
pub mod particles;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use components::*;
use resources::*;
use systems::*;
use tracing::info;
use wgpu;

/// Struct to manage the Bevy application instance.
pub struct BevyRunner {
    app: App,
}

impl BevyRunner {
    /// Creates a new BevyRunner instance with a shared WGPU context.
    pub fn new(
        instance: std::sync::Arc<wgpu::Instance>,
        adapter: std::sync::Arc<wgpu::Adapter>,
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
    ) -> Self {
        info!("Initializing Bevy integration with shared WGPU context...");

        use bevy::render::renderer::{WgpuWrapper, RenderInstance, RenderAdapter, RenderDevice, RenderQueue, RenderAdapterInfo};

        let mut app = App::new();

        // Wrap wgpu resources into Bevy's wrapper types (WgpuWrapper is required for Send/Sync)
        let info = adapter.get_info();
        let render_instance = RenderInstance(std::sync::Arc::new(WgpuWrapper::new((*instance).clone())));
        let render_adapter = RenderAdapter(std::sync::Arc::new(WgpuWrapper::new((*adapter).clone())));
        let render_device = RenderDevice::from((*device).clone());
        let render_queue = RenderQueue::from((*queue).clone());
        let adapter_info = RenderAdapterInfo(WgpuWrapper::new(info));

        // Use DefaultPlugins but with shared WGPU resources.
        app.add_plugins(
            DefaultPlugins
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::manual(
                        render_device,
                        render_queue,
                        adapter_info,
                        render_adapter,
                        render_instance,
                    ),
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<bevy::winit::WinitPlugin>(),
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

        app.add_plugins(particles::ParticlePlugin);

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
}

fn print_status_system() {
    // Placeholder system
}
