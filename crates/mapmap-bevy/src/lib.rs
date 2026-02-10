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
        app.register_type::<Bevy3DShape>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(Update, (audio_reaction_system, hex_grid_system, shape_system));

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
        // Scope for World mutation
        let world = self.app.world_mut();

        world.resource_scope::<BevyNodeMapping, _>(|world, mut mapping| {
            // Track IDs of Bevy3DShape parts we process
            let mut processed_bevy_parts = std::collections::HashSet::new();

            for part in &module.parts {
                if let mapmap_core::module::ModulePartType::Source(
                    mapmap_core::module::SourceType::Bevy3DShape {
                        shape_type,
                        position,
                        rotation,
                        scale,
                        color,
                        unlit,
                    },
                ) = &part.part_type
                {
                    processed_bevy_parts.insert(part.id);

                    let bevy_shape = Bevy3DShape {
                        shape_type: *shape_type,
                        position: Vec3::from_array(*position),
                        rotation: Vec3::from_array(*rotation),
                        scale: Vec3::from_array(*scale),
                        color: LinearRgba::from_f32_array(*color),
                        unlit: *unlit,
                    };

                    if let Some(&entity) = mapping.entities.get(&part.id) {
                        if let Some(mut shape_comp) = world.get_mut::<Bevy3DShape>(entity) {
                            *shape_comp = bevy_shape;
                        } else if let Ok(mut entity_cmds) = world.get_entity_mut(entity) {
                            entity_cmds.insert(bevy_shape);
                        } else {
                            // Entity doesn't exist (externally despawned?), re-spawn
                            let new_entity = world.spawn(bevy_shape).id();
                            mapping.entities.insert(part.id, new_entity);
                        }
                    } else {
                        let entity = world.spawn(bevy_shape).id();
                        mapping.entities.insert(part.id, entity);
                    }
                }
            }

            // Cleanup
            let mut to_remove = Vec::new();

            // Build set of ALL current part IDs in the module to detect deletions
            let all_part_ids: std::collections::HashSet<u64> =
                module.parts.iter().map(|p| p.id).collect();

            for (part_id, &entity) in mapping.entities.iter() {
                // If the part was processed as a Bevy3DShape this frame, keep it.
                if processed_bevy_parts.contains(part_id) {
                    continue;
                }

                // If the part ID no longer exists in the graph, we must delete the entity.
                if !all_part_ids.contains(part_id) {
                    to_remove.push(*part_id);
                    continue;
                }

                // The part exists but is NOT a Bevy3DShape (anymore, or never was).
                // Check if the entity has Bevy3DShape component.
                // If it does, it means the node CHANGED type from Shape -> Something Else.
                // In this case, we should clean up the Bevy entity.
                if world.get::<Bevy3DShape>(entity).is_some() {
                    to_remove.push(*part_id);
                } else {
                    // Entity exists, Part exists (as non-Shape), Entity does NOT have Shape component.
                    // This implies it's another type of Bevy node (e.g. HexGrid) managed elsewhere or manually.
                    // We LEAVE IT ALONE to avoid breaking other systems.
                }
            }

            for part_id in to_remove {
                if let Some(entity) = mapping.entities.remove(&part_id) {
                    world.despawn(entity);
                }
            }
        });
    }
}

fn print_status_system() {
    // Placeholder system
}
