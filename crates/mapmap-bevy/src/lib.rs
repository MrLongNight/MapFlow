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
        app.register_type::<Bevy3DText>();
        app.register_type::<BevyCamera>();
        app.register_type::<Bevy3DShape>();

        // Register systems
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                hex_grid_system,
                text_3d_system,
                camera_control_system,
                shape_system,
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
        let module_id = module.id;
        let mut needed_entities = std::collections::HashSet::new();
        let mut to_spawn = Vec::new();

        // Scope for resource borrow
        {
            let world = self.app.world_mut();
            let mapping = world.resource::<BevyNodeMapping>();

            for part in &module.parts {
                if let mapmap_core::module::ModulePartType::Source(source) = &part.part_type {
                    if is_bevy_source(source) {
                        let key = (module_id, part.id);
                        needed_entities.insert(key);
                        if !mapping.entities.contains_key(&key) {
                            to_spawn.push((part.id, source.clone()));
                        }
                    }
                }
            }
        }

        // Spawn new entities
        for (part_id, _) in to_spawn {
            let entity = self
                .app
                .world_mut()
                .spawn((
                    Transform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    GlobalTransform::default(),
                ))
                .id();

            let mut mapping = self.app.world_mut().resource_mut::<BevyNodeMapping>();
            mapping.entities.insert((module_id, part_id), entity);
        }

        // Despawn removed entities for this module
        let to_despawn: Vec<((u64, u64), Entity)> = {
            let mapping = self.app.world().resource::<BevyNodeMapping>();
            mapping
                .entities
                .iter()
                .filter(|(&(mid, _), _)| mid == module_id)
                .filter(|(key, _)| !needed_entities.contains(key))
                .map(|(&k, &e)| (k, e))
                .collect()
        };

        for (key, entity) in to_despawn {
            self.app.world_mut().despawn(entity);
            self.app
                .world_mut()
                .resource_mut::<BevyNodeMapping>()
                .entities
                .remove(&key);
        }

        // Update components
        let entity_map: Vec<(u64, Entity)> = {
            let mapping = self.app.world().resource::<BevyNodeMapping>();
            mapping
                .entities
                .iter()
                .filter(|(&(mid, _), _)| mid == module_id)
                .map(|(&(_, pid), &e)| (pid, e))
                .collect()
        };
        let entity_lookup: std::collections::HashMap<u64, Entity> =
            entity_map.into_iter().collect();

        for part in &module.parts {
            if let mapmap_core::module::ModulePartType::Source(source) = &part.part_type {
                if let Some(&entity) = entity_lookup.get(&part.id) {
                    sync_bevy_part(self.app.world_mut(), entity, source);
                }
            }
        }
    }
}

fn is_bevy_source(source: &mapmap_core::module::SourceType) -> bool {
    use mapmap_core::module::SourceType;
    matches!(
        source,
        SourceType::Bevy
            | SourceType::BevyAtmosphere { .. }
            | SourceType::BevyHexGrid { .. }
            | SourceType::BevyParticles { .. }
            | SourceType::Bevy3DShape { .. }
            | SourceType::Bevy3DText { .. }
            | SourceType::BevyCamera { .. }
    )
}

fn sync_bevy_part(world: &mut World, entity: Entity, source: &mapmap_core::module::SourceType) {
    use mapmap_core::module::SourceType;

    // 1. Sync Transform (for spatial nodes)
    let transform_data = match source {
        SourceType::BevyHexGrid {
            position,
            rotation,
            scale,
            ..
        } => Some((
            Vec3::from(*position),
            Vec3::from(*rotation),
            Vec3::splat(*scale),
        )),
        SourceType::BevyParticles {
            position, rotation, ..
        } => Some((Vec3::from(*position), Vec3::from(*rotation), Vec3::ONE)),
        SourceType::Bevy3DShape {
            position,
            rotation,
            scale,
            ..
        } => Some((
            Vec3::from(*position),
            Vec3::from(*rotation),
            Vec3::from(*scale),
        )),
        SourceType::Bevy3DText {
            position, rotation, ..
        } => Some((
            Vec3::from(*position),
            Vec3::from(*rotation),
            Vec3::ONE,
        )),
        _ => None,
    };

    if let Some((pos, rot, scale)) = transform_data {
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation = pos;
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                rot.x.to_radians(),
                rot.y.to_radians(),
                rot.z.to_radians(),
            );
            transform.scale = scale;
        }
    }

    // 2. Sync Specific Components
    match source {
        SourceType::Bevy3DShape {
            shape_type,
            color,
            unlit,
            ..
        } => {
            world.entity_mut(entity).insert(Bevy3DShape {
                shape_type: *shape_type,
                color: *color,
                unlit: *unlit,
            });
        }
        SourceType::Bevy3DText {
            text,
            font_size,
            color,
            alignment,
            ..
        } => {
            let align_enum = match alignment.as_str() {
                "Center" => BevyTextAlignment::Center,
                "Right" => BevyTextAlignment::Right,
                "Justify" => BevyTextAlignment::Justify,
                _ => BevyTextAlignment::Left,
            };
            world.entity_mut(entity).insert(Bevy3DText {
                text: text.clone(),
                font_size: *font_size,
                color: *color,
                alignment: align_enum,
            });
        }
        SourceType::BevyCamera {
            mode,
            fov,
            active,
        } => {
            // Convert Core mode to Component mode
            let comp_mode = match mode {
                mapmap_core::module::BevyCameraMode::Orbit {
                    radius,
                    speed,
                    target,
                    height,
                } => crate::components::BevyCameraMode::Orbit {
                    radius: *radius,
                    speed: *speed,
                    target: Vec3::from(*target),
                    height: *height,
                },
                mapmap_core::module::BevyCameraMode::Fly { speed, sensitivity } => {
                    crate::components::BevyCameraMode::Fly {
                        speed: *speed,
                        sensitivity: *sensitivity,
                    }
                }
                mapmap_core::module::BevyCameraMode::Static { position, look_at } => {
                    crate::components::BevyCameraMode::Static {
                        position: Vec3::from(*position),
                        look_at: Vec3::from(*look_at),
                    }
                }
            };

            world.entity_mut(entity).insert(BevyCamera {
                mode: comp_mode,
                fov: *fov,
                active: *active,
            });
        }
        SourceType::BevyHexGrid {
            radius,
            rings,
            pointy_top,
            spacing,
            ..
        } => {
            world.entity_mut(entity).insert(BevyHexGrid {
                radius: *radius,
                rings: *rings,
                pointy_top: *pointy_top,
                spacing: *spacing,
            });
        }
        SourceType::BevyParticles {
            rate,
            lifetime,
            speed,
            color_start,
            color_end,
            ..
        } => {
            world.entity_mut(entity).insert(BevyParticles {
                rate: *rate,
                lifetime: *lifetime,
                speed: *speed,
                color_start: *color_start,
                color_end: *color_end,
            });
        }
        _ => {}
    }
}

fn print_status_system() {
    // Placeholder system
}
