//!
//! Global module data.
//!

use crate::module::types::connection::ModuleConnection;
use crate::module::types::hue::HueNodeType;
use crate::module::types::layer::LayerType;
use crate::module::types::mask::{MaskShape, MaskType};
use crate::module::types::modulizer::ModulizerType;
use crate::module::types::node_link::NodeLinkData;
use crate::module::types::output::OutputType;
use crate::module::types::part::{ModulePart, ModulePartType, PartType};
use crate::module::types::socket::ModuleSocketDirection;
use crate::module::types::source::{BevyShapeType, SourceType};
use crate::module::types::trigger::TriggerType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

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

/// Validation error for a graph connection.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConnectionValidationError {
    /// Source part does not exist.
    #[error("source part {0} does not exist")]
    MissingSourcePart(ModulePartId),
    /// Target part does not exist.
    #[error("target part {0} does not exist")]
    MissingTargetPart(ModulePartId),
    /// Source socket ID is invalid.
    #[error("source socket {socket_id} is invalid on part {part_id}")]
    InvalidSourceSocket {
        /// Part ID.
        part_id: ModulePartId,
        /// Socket ID string.
        socket_id: String,
    },
    /// Target socket ID is invalid.
    #[error("target socket {socket_id} is invalid on part {part_id}")]
    InvalidTargetSocket {
        /// Part ID.
        part_id: ModulePartId,
        /// Socket ID string.
        socket_id: String,
    },
    /// Sockets are not direction-compatible.
    #[error("connection requires an output socket feeding an input socket")]
    InvalidSocketDirection,
    /// Sockets carry incompatible data types.
    #[error("socket types are incompatible")]
    IncompatibleSocketTypes,
    /// Connections between the same part are not allowed.
    #[error("self-connections are not allowed")]
    SelfConnection,
}

/// Summary of automatic graph repair actions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleRepairReport {
    /// Number of parts whose socket schema was refreshed.
    pub refreshed_parts: usize,
    /// Number of invalid or duplicate connections removed.
    pub removed_connections: usize,
    /// Number of invalid trigger target mappings removed.
    pub removed_trigger_targets: usize,
    /// Number of part configurations normalized.
    pub normalized_parts: usize,
}

impl ModuleRepairReport {
    /// Returns true if any repair changed the graph.
    pub fn changed(&self) -> bool {
        self.refreshed_parts > 0
            || self.removed_connections > 0
            || self.removed_trigger_targets > 0
            || self.normalized_parts > 0
    }
}

impl MapFlowModule {
    /// Add a part to this module with proper socket configuration
    pub fn add_part(&mut self, part_type: PartType, position: (f32, f32)) -> ModulePartId {
        let id = self.next_part_id;
        self.next_part_id += 1;
        let requested_part_type = match part_type {
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
            PartType::Output => ModulePartType::Output(OutputType::Projector {
                id: 0,
                name: String::new(),
                hide_cursor: true,
                target_screen: 0,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 0,
                output_height: 0,
                output_fps: 60.0,
                ndi_enabled: false,
                ndi_stream_name: String::new(),
            }),
        };
        let module_part_type = self.normalize_inserted_part_type(requested_part_type);

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
        let part_type = self.normalize_inserted_part_type(part_type);

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
        from_socket: String,
        to_part: ModulePartId,
        to_socket: String,
    ) {
        let _ = self.connect_parts(from_part, from_socket, to_part, to_socket);
    }

    /// Method implementation.
    pub fn remove_connection(
        &mut self,
        from_part: ModulePartId,
        from_socket: String,
        to_part: ModulePartId,
        to_socket: String,
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
        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            let (new_inputs, new_outputs) = part.compute_sockets();
            part.inputs = new_inputs;
            part.outputs = new_outputs;
            let valid_mappable_inputs: HashSet<usize> = part
                .schema()
                .inspector
                .mappable_input_indices
                .into_iter()
                .collect();
            part.trigger_targets
                .retain(|socket_idx, _| valid_mappable_inputs.contains(socket_idx));
        }
        let _ = self.repair_graph();
    }

    /// Method implementation.
    pub fn update_part_outputs(&mut self, part_id: ModulePartId) {
        self.update_part_sockets(part_id);
    }

    /// Get a part by ID.
    pub fn part(&self, part_id: ModulePartId) -> Option<&ModulePart> {
        self.parts.iter().find(|part| part.id == part_id)
    }

    /// Validate a connection against the current schema.
    pub fn validate_connection(
        &self,
        from_part: ModulePartId,
        from_socket: String,
        to_part: ModulePartId,
        to_socket: String,
    ) -> Result<(), ConnectionValidationError> {
        if from_part == to_part {
            return Err(ConnectionValidationError::SelfConnection);
        }

        let source = self
            .part(from_part)
            .ok_or(ConnectionValidationError::MissingSourcePart(from_part))?;
        let target = self
            .part(to_part)
            .ok_or(ConnectionValidationError::MissingTargetPart(to_part))?;

        let source_socket = source.outputs.iter().find(|s| s.id == from_socket).ok_or(
            ConnectionValidationError::InvalidSourceSocket {
                part_id: from_part,
                socket_id: from_socket,
            },
        )?;
        let target_socket = target.inputs.iter().find(|s| s.id == to_socket).ok_or(
            ConnectionValidationError::InvalidTargetSocket {
                part_id: to_part,
                socket_id: to_socket,
            },
        )?;

        if source_socket.direction != ModuleSocketDirection::Output
            || target_socket.direction != ModuleSocketDirection::Input
        {
            return Err(ConnectionValidationError::InvalidSocketDirection);
        }

        if !source_socket.is_compatible_with(target_socket) {
            return Err(ConnectionValidationError::IncompatibleSocketTypes);
        }

        Ok(())
    }

    /// Connect two validated sockets. Existing target links are replaced for single-input sockets.
    pub fn connect_parts(
        &mut self,
        from_part: ModulePartId,
        from_socket: String,
        to_part: ModulePartId,
        to_socket: String,
    ) -> Result<bool, ConnectionValidationError> {
        self.validate_connection(from_part, from_socket.clone(), to_part, to_socket.clone())?;

        let target_accepts_multiple = self
            .part(to_part)
            .and_then(|part| part.inputs.iter().find(|s| s.id == to_socket))
            .map(|socket| socket.accepts_multiple_connections)
            .unwrap_or(false);

        if !target_accepts_multiple {
            self.connections
                .retain(|conn| !(conn.to_part == to_part && conn.to_socket == to_socket));
        }

        let candidate = ModuleConnection {
            from_part,
            from_socket,
            to_part,
            to_socket,
        };
        if self.connections.contains(&candidate) {
            return Ok(false);
        }

        self.connections.push(candidate);
        Ok(true)
    }

    /// Refresh socket schema, remove invalid references and normalize unstable defaults.
    pub fn repair_graph(&mut self) -> ModuleRepairReport {
        let mut report = ModuleRepairReport::default();

        let mut used_output_ids = HashSet::new();
        let mut next_output_id = 1_u64;
        let mut used_layer_ids = HashSet::new();
        let mut next_layer_id = 1_u64;

        for part in &mut self.parts {
            let mut normalized = false;

            match &mut part.part_type {
                ModulePartType::Output(OutputType::Projector { id, name, .. }) => {
                    if *id == 0 || !used_output_ids.insert(*id) {
                        while used_output_ids.contains(&next_output_id) {
                            next_output_id += 1;
                        }
                        *id = next_output_id;
                        used_output_ids.insert(*id);
                        next_output_id += 1;
                        normalized = true;
                    }
                    if name.trim().is_empty() {
                        *name = format!("Output {}", *id);
                        normalized = true;
                    }
                }
                ModulePartType::Layer(crate::module::types::layer::LayerType::Single {
                    id,
                    name,
                    ..
                }) => {
                    if *id == 0 || !used_layer_ids.insert(*id) {
                        while used_layer_ids.contains(&next_layer_id) {
                            next_layer_id += 1;
                        }
                        *id = next_layer_id;
                        used_layer_ids.insert(*id);
                        next_layer_id += 1;
                        normalized = true;
                    }
                    if name.trim().is_empty() {
                        *name = format!("Layer {}", *id);
                        normalized = true;
                    }
                }
                _ => {}
            }

            let (new_inputs, new_outputs) = part.compute_sockets();
            if part.inputs != new_inputs || part.outputs != new_outputs {
                part.inputs = new_inputs;
                part.outputs = new_outputs;
                report.refreshed_parts += 1;
            }

            let valid_mappable_inputs: HashSet<usize> = part
                .schema()
                .inspector
                .mappable_input_indices
                .into_iter()
                .collect();
            let before_targets = part.trigger_targets.len();
            part.trigger_targets
                .retain(|socket_idx, _| valid_mappable_inputs.contains(socket_idx));
            report.removed_trigger_targets += before_targets - part.trigger_targets.len();

            if normalized {
                report.normalized_parts += 1;
            }
        }

        let existing_part_ids: HashSet<ModulePartId> =
            self.parts.iter().map(|part| part.id).collect();
        let mut seen_connections = HashSet::new();
        let mut repaired_connections = Vec::with_capacity(self.connections.len());

        for connection in self.connections.iter().cloned() {
            let unique_key = (
                connection.from_part,
                connection.from_socket.clone(),
                connection.to_part,
                connection.to_socket.clone(),
            );

            let is_valid = existing_part_ids.contains(&connection.from_part)
                && existing_part_ids.contains(&connection.to_part)
                && self
                    .validate_connection(
                        connection.from_part,
                        connection.from_socket.clone(),
                        connection.to_part,
                        connection.to_socket.clone(),
                    )
                    .is_ok();

            if is_valid && seen_connections.insert(unique_key) {
                repaired_connections.push(connection);
            } else {
                report.removed_connections += 1;
            }
        }

        self.connections = repaired_connections;
        report
    }

    fn normalize_inserted_part_type(&self, part_type: ModulePartType) -> ModulePartType {
        match part_type {
            ModulePartType::Output(OutputType::Projector {
                id,
                name,
                hide_cursor,
                target_screen,
                show_in_preview_panel,
                extra_preview_window,
                output_width,
                output_height,
                output_fps,
                ndi_enabled,
                ndi_stream_name,
            }) => {
                let used_ids: HashSet<u64> = self
                    .parts
                    .iter()
                    .filter_map(|part| {
                        if let ModulePartType::Output(OutputType::Projector { id, .. }) =
                            &part.part_type
                        {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut resolved_id = id.max(1);
                while used_ids.contains(&resolved_id) {
                    resolved_id += 1;
                }
                ModulePartType::Output(OutputType::Projector {
                    id: resolved_id,
                    name: if name.trim().is_empty() {
                        format!("Output {}", resolved_id)
                    } else {
                        name
                    },
                    hide_cursor,
                    target_screen,
                    show_in_preview_panel,
                    extra_preview_window,
                    output_width,
                    output_height,
                    output_fps,
                    ndi_enabled,
                    ndi_stream_name,
                })
            }
            ModulePartType::Layer(crate::module::types::layer::LayerType::Single {
                id,
                name,
                opacity,
                blend_mode,
                mesh,
                mapping_mode,
            }) => {
                let used_ids: HashSet<u64> = self
                    .parts
                    .iter()
                    .filter_map(|part| {
                        if let ModulePartType::Layer(
                            crate::module::types::layer::LayerType::Single { id, .. },
                        ) = &part.part_type
                        {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut resolved_id = id.max(1);
                while used_ids.contains(&resolved_id) {
                    resolved_id += 1;
                }
                ModulePartType::Layer(crate::module::types::layer::LayerType::Single {
                    id: resolved_id,
                    name: if name.trim().is_empty() {
                        format!("Layer {}", resolved_id)
                    } else {
                        name
                    },
                    opacity,
                    blend_mode,
                    mesh,
                    mapping_mode,
                })
            }
            other => other,
        }
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
