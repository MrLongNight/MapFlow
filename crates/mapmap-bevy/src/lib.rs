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
        let render_queue = RenderQueue(std::sync::Arc::new(WgpuWrapper::new((*queue).clone())));
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
        app.register_type::<BevyCamera>();
        app.register_type::<BevyCameraState>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                hex_grid_system,
                camera_control_system,
            ),
        );

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

    /// Sync module graph state to Bevy entities
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        let world = self.app.world_mut();

        for part in &module.parts {
            // Check if we have an entity for this part
            let entity = world
                .resource::<BevyNodeMapping>()
                .entities
                .get(&part.id)
                .copied();

            use mapmap_core::module::{ModulePartType, SourceType};

            if let ModulePartType::Source(SourceType::BevyCamera {
                mode,
                target,
                position,
                distance,
                speed,
                direction,
            }) = &part.part_type
            {
                // Map Core Mode to Bevy Mode
                let bevy_mode = match mode {
                    mapmap_core::module::BevyCameraMode::Orbit => BevyCameraMode::Orbit,
                    mapmap_core::module::BevyCameraMode::Fly => BevyCameraMode::Fly,
                    mapmap_core::module::BevyCameraMode::Static => BevyCameraMode::Static,
                };

                let target = Vec3::from(*target);
                let position = Vec3::from(*position);
                let direction = Vec3::from(*direction);

                if let Some(e) = entity {
                    if let Some(mut cam) = world.get_mut::<BevyCamera>(e) {
                        cam.mode = bevy_mode;
                        cam.target = target;
                        cam.position = position;
                        cam.distance = *distance;
                        cam.speed = *speed;
                        cam.direction = direction;
                    }
                } else {
                    // Spawn new
                    let id = world
                        .spawn((
                            BevyCamera {
                                mode: bevy_mode,
                                target,
                                position,
                                distance: *distance,
                                speed: *speed,
                                direction,
                            },
                            BevyCameraState {
                                current_pos: position,
                                current_angle: 0.0,
                            },
                        ))
                        .id();
                    world
                        .resource_mut::<BevyNodeMapping>()
                        .entities
                        .insert(part.id, id);
                }
            }
        }
    }
}

fn print_status_system() {
    // Placeholder system
}
