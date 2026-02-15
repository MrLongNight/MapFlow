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
        app.register_type::<BevyCameraConfig>();
        app.register_type::<CameraMode>();

        // Register systems
        app.add_systems(Startup, setup_3d_scene);
        app.add_systems(Update, print_status_system);
        app.add_systems(
            Update,
            (
                audio_reaction_system,
                hex_grid_system,
                text_3d_system,
                particle_system,
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
        let mut mapping_resource = world.resource_mut::<BevyNodeMapping>();
        let mut mapping = std::mem::take(&mut mapping_resource.entities);
        let mut active_ids = std::collections::HashSet::new();
        let mut camera_node_found = false;

        for part in &module.parts {
            if let mapmap_core::module::ModulePartType::Source(source) = &part.part_type {
                match source {
                    mapmap_core::module::SourceType::Bevy3DText {
                        text,
                        font_size,
                        color,
                        position,
                        rotation,
                        alignment,
                    } => {
                        active_ids.insert(part.id);

                        let entity = if let Some(&e) = mapping.get(&part.id) {
                            if world.get_entity(e).is_ok() {
                                e
                            } else {
                                world.spawn_empty().id()
                            }
                        } else {
                            world.spawn_empty().id()
                        };
                        mapping.insert(part.id, entity);

                        let align_enum = match alignment.as_str() {
                            "Center" => BevyTextAlignment::Center,
                            "Right" => BevyTextAlignment::Right,
                            "Justify" => BevyTextAlignment::Justify,
                            _ => BevyTextAlignment::Left,
                        };

                        let rotation_quat = Quat::from_euler(
                            EulerRot::XYZ,
                            rotation[0].to_radians(),
                            rotation[1].to_radians(),
                            rotation[2].to_radians(),
                        );

                        let mut entity_mut = world.entity_mut(entity);
                        let new_text_comp = Bevy3DText {
                            text: text.clone(),
                            font_size: *font_size,
                            color: *color,
                            alignment: align_enum,
                        };

                        // Check change to avoid triggering Changed<Bevy3DText> every frame
                        let needs_update = if let Some(current) = entity_mut.get::<Bevy3DText>() {
                            current.text != new_text_comp.text
                                || (current.font_size - new_text_comp.font_size).abs()
                                    > f32::EPSILON
                                || current.color != new_text_comp.color
                                || current.alignment != new_text_comp.alignment
                        } else {
                            true
                        };

                        if needs_update {
                            entity_mut.insert(new_text_comp);
                        }

                        entity_mut.insert(Transform {
                            translation: Vec3::from(*position),
                            rotation: rotation_quat,
                            scale: Vec3::ONE,
                        });

                        if !entity_mut.contains::<GlobalTransform>() {
                            entity_mut.insert((
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ));
                        }
                    }
                    mapmap_core::module::SourceType::BevyCamera {
                        mode,
                        target,
                        position,
                        radius,
                        speed,
                        yaw,
                        pitch,
                    } => {
                        camera_node_found = true;
                        // Find the shared engine camera
                        // Since there is only one shared engine camera, we don't map it by part ID
                        // but rather look for the component.
                        let mut camera_entity = None;
                        {
                            let mut query =
                                world.query_filtered::<Entity, With<SharedEngineCamera>>();
                            if let Some(e) = query.iter(world).next() {
                                camera_entity = Some(e);
                            }
                        }

                        if let Some(entity) = camera_entity {
                            let bevy_mode = match mode {
                                mapmap_core::module::CameraMode::Orbit => CameraMode::Orbit,
                                mapmap_core::module::CameraMode::Fly => CameraMode::Fly,
                                mapmap_core::module::CameraMode::Static => CameraMode::Static,
                            };

                            let mut entity_mut = world.entity_mut(entity);
                            entity_mut.insert(BevyCameraConfig {
                                mode: bevy_mode,
                                target: *target,
                                position: *position,
                                radius: *radius,
                                speed: *speed,
                                yaw: *yaw,
                                pitch: *pitch,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        // Cleanup
        mapping.retain(|id, entity| {
            if !active_ids.contains(id) {
                // Only despawn if it looks like one of ours (has Bevy3DText)
                // This prevents deleting entities from other node types if we add them later to mapping
                // For now, since only Bevy3DText uses mapping, it's safe-ish.
                world.despawn(*entity);
                false
            } else {
                true
            }
        });

        world.resource_mut::<BevyNodeMapping>().entities = mapping;

        // Cleanup Camera Config if node removed
        if !camera_node_found {
            let mut camera_entity = None;
            {
                let mut query = world.query_filtered::<Entity, With<SharedEngineCamera>>();
                if let Some(e) = query.iter(world).next() {
                    camera_entity = Some(e);
                }
            }
            if let Some(entity) = camera_entity {
                world.entity_mut(entity).remove::<BevyCameraConfig>();
            }
        }
    }
}

fn print_status_system() {
    // Placeholder system
}
