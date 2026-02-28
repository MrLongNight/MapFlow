use crate::module::types::connection::ModuleConnection;
use crate::module::types::hue::HueNodeType;
use crate::module::types::layer::LayerType;
use crate::module::types::mask::{MaskShape, MaskType};
use crate::module::types::modulizer::ModulizerType;
use crate::module::types::node_link::NodeLinkData;
use crate::module::types::output::OutputType;
use crate::module::types::part::{ModulePart, ModulePartType, PartType};
use crate::module::types::source::{BevyShapeType, SourceType};
use crate::module::types::trigger::TriggerType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a Module
pub type ModuleId = u64;
/// Unique identifier for a Part within a Module
pub type ModulePartId = u64;

/// Represents a complete visual programming graph (Scene/Module)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapFlowModule {
    /// Unique identifier for this entity.
    pub id: ModuleId,
    /// Display name
    pub name: String,
    /// UI color for the module button
    pub color: [f32; 4],
    /// List of nodes (parts)
    pub parts: Vec<ModulePart>,
    /// List of wires (connections)
    pub connections: Vec<ModuleConnection>,
    /// How the module plays back
    pub playback_mode: ModulePlaybackMode,
    /// Counter for generating part IDs (persistent)
    #[serde(default = "crate::module::config::default_next_part_id")]
    /// The ID to be assigned to the next created part within a module.
    pub next_part_id: ModulePartId,
}

impl MapFlowModule {
    /// Add a part to this module with proper socket configuration
    pub fn add_part(&mut self, part_type: PartType, position: (f32, f32)) -> ModulePartId {
        let id = self.next_part_id;
        self.next_part_id += 1;
        let module_part_type = match part_type {
            PartType::Trigger => ModulePartType::Trigger(TriggerType::Beat),
            PartType::Source => ModulePartType::Source(SourceType::MediaFile {
                path: String::new(),
                speed: 1.0,
                loop_enabled: true,
                start_time: 0.0,
                end_time: 0.0,
                opacity: 1.0,
                blend_mode: None,
                brightness: 0.0,
                contrast: 1.0,
                saturation: 1.0,
                hue_shift: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                target_width: None,
                target_height: None,
                target_fps: None,
                flip_horizontal: false,
                flip_vertical: false,
                reverse_playback: false,
            }),
            PartType::Bevy3DShape => ModulePartType::Source(SourceType::Bevy3DShape {
                shape_type: BevyShapeType::Cube,
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                unlit: false,
                outline_width: 0.0,
                outline_color: [1.0, 1.0, 1.0, 1.0],
            }),
            PartType::BevyParticles => ModulePartType::Source(SourceType::BevyParticles {
                rate: 100.0,
                lifetime: 2.0,
                speed: 1.0,
                color_start: [1.0, 1.0, 1.0, 1.0],
                color_end: [1.0, 1.0, 1.0, 0.0],
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
            }),
            PartType::Mask => ModulePartType::Mask(MaskType::Shape(MaskShape::Rectangle)),
            PartType::Modulator => ModulePartType::Modulizer(ModulizerType::Effect {
                effect_type: crate::module::types::modulizer::EffectType::Blur,
                params: std::collections::HashMap::new(),
            }),
            PartType::Mesh => ModulePartType::Mesh(crate::module::types::mesh::MeshType::Grid {
                cols: 10,
                rows: 10,
            }),
            PartType::Layer => ModulePartType::Layer(LayerType::Single {
                id: 0,
                name: "Layer 1".to_string(),
                opacity: 1.0,
                blend_mode: None,
                mesh: crate::module::config::default_mesh_quad(),
                mapping_mode: false,
            }),

            PartType::Hue => ModulePartType::Hue(HueNodeType::SingleLamp {
                id: String::new(),
                name: "New Lamp".to_string(),
                brightness: 1.0,
                color: [1.0, 1.0, 1.0],
                effect: None,
                effect_active: false,
            }),
            PartType::Output => {
                let used_ids: Vec<u64> = self
                    .parts
                    .iter()
                    .filter_map(|p| {
                        if let ModulePartType::Output(OutputType::Projector { id, .. }) =
                            &p.part_type
                        {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut next_id = 1;
                while used_ids.contains(&next_id) {
                    next_id += 1;
                }

                ModulePartType::Output(OutputType::Projector {
                    id: next_id,
                    name: format!("Output {}", next_id),
                    hide_cursor: true,
                    target_screen: 0,
                    show_in_preview_panel: true,
                    extra_preview_window: false,
                    output_width: 0,
                    output_height: 0,
                    output_fps: 60.0,
                    ndi_enabled: false,
                    ndi_stream_name: String::new(),
                })
            }
        };

        let mut part = ModulePart {
            id,
            part_type: module_part_type,
            position,
            size: None,
            link_data: NodeLinkData::default(),
            inputs: vec![],
            outputs: vec![],
            trigger_targets: HashMap::new(),
        };

        let (inputs, outputs) = part.compute_sockets();
        part.inputs = inputs;
        part.outputs = outputs;

        self.parts.push(part);
        id
    }

    /// Method implementation.
    pub fn add_part_with_type(
        &mut self,
        part_type: ModulePartType,
        position: (f32, f32),
    ) -> ModulePartId {
        let id = self.next_part_id;
        self.next_part_id += 1;

        let mut part = ModulePart {
            id,
            part_type,
            position,
            size: None,
            link_data: NodeLinkData::default(),
            inputs: vec![],
            outputs: vec![],
            trigger_targets: HashMap::new(),
        };

        let (inputs, outputs) = part.compute_sockets();
        part.inputs = inputs;
        part.outputs = outputs;

        self.parts.push(part);
        id
    }

    /// Method implementation.
    pub fn update_part_position(&mut self, part_id: ModulePartId, new_position: (f32, f32)) {
        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            part.position = new_position;
        }
    }

    /// Method implementation.
    pub fn add_connection(
        &mut self,
        from_part: ModulePartId,
        from_socket: usize,
        to_part: ModulePartId,
        to_socket: usize,
    ) {
        self.connections.push(ModuleConnection {
            from_part,
            from_socket,
            to_part,
            to_socket,
        });
    }

    /// Method implementation.
    pub fn remove_connection(
        &mut self,
        from_part: ModulePartId,
        from_socket: usize,
        to_part: ModulePartId,
        to_socket: usize,
    ) {
        self.connections.retain(|c| {
            !(c.from_part == from_part
                && c.from_socket == from_socket
                && c.to_part == to_part
                && c.to_socket == to_socket)
        });
    }

    /// Method implementation.
    pub fn update_part_sockets(&mut self, part_id: ModulePartId) {
        let mut in_count = 0;
        let mut out_count = 0;

        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            let (new_inputs, new_outputs) = part.compute_sockets();
            part.inputs = new_inputs;
            part.outputs = new_outputs;
            in_count = part.inputs.len();
            out_count = part.outputs.len();
        }

        if in_count > 0 || out_count > 0 {
            self.connections.retain(|c| {
                if c.to_part == part_id && c.to_socket >= in_count {
                    return false;
                }
                if c.from_part == part_id && c.from_socket >= out_count {
                    return false;
                }
                true
            });
        }
    }

    /// Method implementation.
    pub fn update_part_outputs(&mut self, part_id: ModulePartId) {
        self.update_part_sockets(part_id);
    }
}

/// Defines how the module handles time and looping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePlaybackMode {
    /// Play for a fixed duration
    TimelineDuration {
        /// Duration in milliseconds
        duration_ms: u64,
    },
    /// Loop indefinitely until user switches module
    LoopUntilManualSwitch,
}
