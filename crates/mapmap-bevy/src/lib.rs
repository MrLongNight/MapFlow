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

    /// Retrieve the rendered frame data (BGRA8).
    pub fn get_image_data(&self) -> Option<(Vec<u8>, u32, u32)> {
        let output = self.app.world().get_resource::<BevyRenderOutput>()?;
        let data_guard = output.last_frame_data.lock().ok()?;
        let data = data_guard.clone()?;
        Some((data, output.width, output.height))
    }

    /// Sync the MapFlow module graph state to Bevy entities.
    pub fn apply_graph_state(&mut self, module: &mapmap_core::module::MapFlowModule) {
        let world = self.app.world_mut();

        let mut mapping = std::mem::take(&mut *world.resource_mut::<BevyNodeMapping>());
        let mut active_parts = std::collections::HashSet::new();

        for part in &module.parts {
            active_parts.insert(part.id);

            if let mapmap_core::module::ModulePartType::Source(
                mapmap_core::module::SourceType::BevyCamera {
                    mode,
                    position,
                    look_at,
                    up,
                    fov,
                    speed,
                },
            ) = &part.part_type
            {
                let entity = if let Some(&e) = mapping.entities.get(&part.id) {
                    if world.get_entity(e).is_ok() {
                        e
                    } else {
                        world.spawn(BevyCamera::default()).id()
                    }
                } else {
                    world.spawn(BevyCamera::default()).id()
                };

                mapping.entities.insert(part.id, entity);

                if let Some(mut comp) = world.get_mut::<BevyCamera>(entity) {
                    comp.mode = *mode;
                    comp.position = Vec3::from_array(*position);
                    comp.look_at = Vec3::from_array(*look_at);
                    comp.up = Vec3::from_array(*up);
                    comp.fov = *fov;
                    comp.speed = *speed;
                }
            }
        }

        // Cleanup removed parts
        mapping.entities.retain(|id, entity| {
            if !active_parts.contains(id) {
                let _ = world.despawn(*entity);
                false
            } else {
                true
            }
        });

        *world.resource_mut::<BevyNodeMapping>() = mapping;
    }
}

fn print_status_system() {
    // Placeholder system
}
