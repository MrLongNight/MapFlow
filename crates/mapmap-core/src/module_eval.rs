//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::LinkMode;
use crate::module::{
    BlendModeType, LayerType, LinkBehavior, MapFlowModule, MaskType, MeshType, ModulePartId,
    ModulePartType, ModulizerType, OutputType, SharedMediaState, SourceType, TriggerType,
};
use rand::RngExt;
use std::cell::RefCell;
use std::collections::HashMap;

use std::sync::Arc;
use std::time::Instant;

/// State for individual trigger nodes, stored in the evaluator
#[derive(Debug, Clone, Default)]
pub enum TriggerState {
    #[default]
    /// No state
    None,
    /// Random trigger state
    Random {
        /// Accumulated time since last trigger
        timer: f32,
        /// Target interval for the next trigger
        target: f32,
    },
    /// Fixed trigger state
    Fixed {
        /// Accumulated time since last trigger
        timer: f32,
    },
}

/// Source-specific rendering properties (from MediaFile)
#[derive(Debug, Clone, Default)]
pub struct SourceProperties {
    /// Source opacity (multiplied with layer opacity)
    pub opacity: f32,
    /// Color correction: Brightness (-1.0 to 1.0)
    pub brightness: f32,
    /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
    pub contrast: f32,
    /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
    pub saturation: f32,
    /// Color correction: Hue shift (-180 to 180 degrees)
    pub hue_shift: f32,
    /// Transform: Scale X
    pub scale_x: f32,
    /// Transform: Scale Y
    pub scale_y: f32,
    /// Transform: Rotation in degrees
    pub rotation: f32,
    /// Transform: Offset X
    pub offset_x: f32,
    /// Transform: Offset Y
    pub offset_y: f32,
    /// Flip horizontally
    pub flip_horizontal: bool,
    /// Flip vertically
    pub flip_vertical: bool,
}

impl SourceProperties {
    /// Default source properties (no modifications)
    pub fn default_identity() -> Self {
        Self {
            opacity: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

/// Cached indices for a specific module graph revision
#[derive(Debug, Clone)]
pub struct ModuleGraphIndices {
    /// Cached map from Part ID to index in `module.parts`
    pub part_index_cache: HashMap<ModulePartId, usize>,
    /// Cached map from Part ID to list of indices in `module.connections` (incoming connections)
    pub conn_index_cache: HashMap<ModulePartId, Vec<usize>>,
    /// The graph revision this cache corresponds to
    pub last_revision: u64,
}

#[cfg(test)]
mod tests_evaluator {
    use super::*;
    use crate::audio::analyzer_v2::AudioAnalysisV2;
    use crate::module::{
        AudioTriggerOutputConfig, MapFlowModule, ModulePartType, SourceType, TriggerType,
    };

    fn create_test_module() -> MapFlowModule {
        MapFlowModule {
            id: 1,
            name: "Test Module".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: crate::module::ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        }
    }

    #[test]
    fn test_evaluator_initialization() {
        let evaluator = ModuleEvaluator::new();
        assert_eq!(evaluator.audio_trigger_data.rms_volume, 0.0);
    }

    #[test]
    fn test_trigger_fixed_interval() {
        let mut evaluator = ModuleEvaluator::new();
        let shared = crate::module::SharedMediaState::default();

        let mut module = create_test_module();
        let trigger_part = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 100,
            offset_ms: 0,
        });
        let part_id = module.add_part_with_type(trigger_part, (0.0, 0.0));

        // Initial eval - t=0.01 (small dt), phase=0.01 < 0.1 -> 0.0
        evaluator.set_delta_time(0.01);
        let result = evaluator.evaluate(&module, &shared, 0);
        let values = &result.trigger_values[&part_id];
        assert_eq!(values[0], 0.0);

        // Simulate 150ms passing
        evaluator.set_delta_time(0.15);

        let result = evaluator.evaluate(&module, &shared, 0);
        let values = &result.trigger_values[&part_id];
        // Now > 100ms should have passed, so it should have triggered
        assert_eq!(values[0], 1.0);
    }

    #[test]
    fn test_trigger_audio_fft() {
        let mut evaluator = ModuleEvaluator::new();

        // Setup Audio Analysis with a "Beat"
        let analysis = AudioAnalysisV2 {
            beat_detected: true,
            rms_volume: 0.8,
            ..Default::default()
        };

        evaluator.update_audio(&analysis);

        let mut module = create_test_module();

        // Setup AudioFFT Trigger expecting Beat Output and Volume
        let config = AudioTriggerOutputConfig {
            beat_output: true,
            volume_outputs: true,
            ..Default::default()
        };
        let trigger_part = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: crate::module::AudioBand::Bass, // irrelevant here
            threshold: 0.5,
            output_config: config,
        });

        let part_id = module.add_part_with_type(trigger_part, (0.0, 0.0));

        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);
        let values = &result.trigger_values[&part_id];

        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 0.8); // RMS
        assert_eq!(values[2], 1.0); // Beat detected
    }

    #[test]
    fn test_evaluator_propagation() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        let t_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        });
        let t_id = module.add_part_with_type(t_type, (0.0, 0.0));

        let s_id = module.add_part(crate::module::PartType::Source, (200.0, 0.0));
        module.add_connection(t_id, 0, s_id, 0);

        let shared = crate::module::SharedMediaState::default();
        evaluator.set_delta_time(0.01);
        let _result = evaluator.evaluate(&module, &shared, 0);

        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        evaluator.set_delta_time(0.01);
        let result = evaluator.evaluate(&module, &shared, 0);
        assert!(result.source_commands.contains_key(&s_id));

        module.remove_connection(t_id, 0, s_id, 0);

        evaluator.set_delta_time(0.01);
        let result = evaluator.evaluate(&module, &shared, 1);

        assert!(result.source_commands.contains_key(&s_id));
    }

    #[test]
    fn test_full_evaluation_pipeline() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        let t_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        });
        let t_id = module.add_part_with_type(t_type, (0.0, 0.0));

        let s_id = module.add_part(crate::module::PartType::Source, (100.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &mut part.part_type
            {
                *path = "test.mp4".to_string();
            }
        }

        let l_id = module.add_part(crate::module::PartType::Layer, (200.0, 0.0));
        let o_id = module.add_part(crate::module::PartType::Output, (300.0, 0.0));

        module.add_connection(t_id, 0, s_id, 0);
        module.add_connection(s_id, 0, l_id, 0);
        module.add_connection(l_id, 0, o_id, 0);

        evaluator.set_delta_time(0.01);
        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        assert_eq!(result.render_ops.len(), 1);
        let op = &result.render_ops[0];
        assert_eq!(op.output_part_id, o_id);
        assert_eq!(op.layer_part_id, l_id);
        assert_eq!(op.source_part_id, Some(s_id));

        assert!(result.source_commands.contains_key(&s_id));
        if let Some(SourceCommand::PlayMedia { path, .. }) = result.source_commands.get(&s_id) {
            assert_eq!(path, "test.mp4");
        } else {
            panic!("Expected PlayMedia command");
        }
    }

    #[test]
    fn test_link_system_master_slave() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        let m_type = ModulePartType::Trigger(TriggerType::Fixed {
            interval_ms: 0,
            offset_ms: 0,
        });
        let m_id = module.add_part_with_type(m_type, (0.0, 0.0));

        if let Some(part) = module.parts.iter_mut().find(|p| p.id == m_id) {
            part.link_data.mode = LinkMode::Master;
            part.link_data.trigger_input_enabled = true;
            part.outputs.push(crate::module::ModuleSocket {
                name: "Link Out".to_string(),
                socket_type: crate::module::ModuleSocketType::Link,
            });
            part.inputs.push(crate::module::ModuleSocket {
                name: "Trigger In (Vis)".to_string(),
                socket_type: crate::module::ModuleSocketType::Trigger,
            });
        }

        let t_id = module.add_part_with_type(
            ModulePartType::Trigger(TriggerType::Fixed {
                interval_ms: 0,
                offset_ms: 0,
            }),
            (-100.0, 0.0),
        );

        module.add_connection(t_id, 0, m_id, 0);

        let s_id = module.add_part(crate::module::PartType::Layer, (100.0, 0.0));
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == s_id) {
            part.link_data.mode = LinkMode::Slave;
            part.inputs.push(crate::module::ModuleSocket {
                name: "Link In".to_string(),
                socket_type: crate::module::ModuleSocketType::Link,
            });
        }

        module.add_connection(m_id, 1, s_id, 2);

        evaluator.set_delta_time(0.01);
        let result = evaluator.evaluate(&module, &crate::module::SharedMediaState::default(), 0);

        let m_values = &result.trigger_values[&m_id];
        assert!(m_values.len() >= 2);
        assert_eq!(m_values[1], 1.0);
    }

    #[test]
    fn test_render_op_pooling() {
        let mut evaluator = ModuleEvaluator::new();
        let mut module = create_test_module();

        let l_id = module.add_part(crate::module::PartType::Layer, (0.0, 0.0));
        let o_id = module.add_part(crate::module::PartType::Output, (100.0, 0.0));
        module.add_connection(l_id, 0, o_id, 0);

        let shared = crate::module::SharedMediaState::default();

        evaluator.set_delta_time(0.01);
        evaluator.evaluate(&module, &shared, 0);
        assert_eq!(evaluator.cached_result.render_ops.len(), 1);
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 0);

        evaluator.set_delta_time(0.01);
        evaluator.evaluate(&module, &shared, 0);
        assert_eq!(evaluator.cached_result.render_ops.len(), 1);
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 0);

        module.remove_connection(l_id, 0, o_id, 0);
        evaluator.set_delta_time(0.01);
        evaluator.evaluate(&module, &shared, 1);

        assert_eq!(evaluator.cached_result.render_ops.len(), 0);
        assert_eq!(evaluator.cached_result.spare_render_ops.len(), 1);
    }
}

/// Render operation containing all info needed to render a layer to an output
#[derive(Debug, Clone)]
pub struct RenderOp {
    pub output_part_id: ModulePartId,
    pub output_type: OutputType,
    pub layer_part_id: ModulePartId,
    pub mesh: MeshType,
    pub opacity: f32,
    pub blend_mode: Option<BlendModeType>,
    pub mapping_mode: bool,
    pub source_part_id: Option<ModulePartId>,
    pub source_props: SourceProperties,
    pub effects: Vec<ModulizerType>,
    pub masks: Vec<MaskType>,
}

/// Evaluation result for a single frame
#[derive(Debug, Clone, Default)]
pub struct ModuleEvalResult {
    pub trigger_values: HashMap<ModulePartId, Vec<f32>>,
    pub source_commands: HashMap<ModulePartId, SourceCommand>,
    pub render_ops: Vec<RenderOp>,
    pub spare_render_ops: Vec<RenderOp>,
}

impl ModuleEvalResult {
    pub fn clear(&mut self) {
        for values in self.trigger_values.values_mut() {
            values.clear();
        }
        self.source_commands.clear();
        self.spare_render_ops.append(&mut self.render_ops);
    }
}

/// Command for a source node
#[derive(Debug, Clone)]
pub enum SourceCommand {
    PlayMedia {
        path: String,
        trigger_value: f32,
    },
    PlaySharedMedia {
        id: String,
        path: String,
        trigger_value: f32,
    },
    PlayShader {
        name: String,
        params: Vec<(String, f32)>,
        trigger_value: f32,
    },
    NdiInput {
        source_name: Option<String>,
        trigger_value: f32,
    },
    LiveInput {
        device_id: u32,
        trigger_value: f32,
    },
    #[cfg(target_os = "windows")]
    SpoutInput {
        sender_name: String,
        trigger_value: f32,
    },
    BevyInput {
        trigger_value: f32,
    },
    Bevy3DModel {
        path: String,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
        trigger_value: f32,
    },
    HueOutput {
        brightness: f32,
        hue: Option<f32>,
        saturation: Option<f32>,
        strobe: Option<f32>,
        ids: Option<Vec<String>>,
    },
}

/// Module graph evaluator
pub struct ModuleEvaluator {
    audio_trigger_data: AudioTriggerData,
    trigger_states: HashMap<ModulePartId, TriggerState>,
    cached_result: ModuleEvalResult,
    indices_cache: HashMap<crate::module::ModuleId, Arc<ModuleGraphIndices>>,
    active_keys: std::collections::HashSet<String>,
    trigger_smoothing_state: RefCell<HashMap<(ModulePartId, usize), (f32, u64)>>,
    manual_triggers: std::collections::HashSet<ModulePartId>,
    current_frame: u64,
    last_eval_time: Instant,
    current_dt: f32,
}

impl Default for ModuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleEvaluator {
    pub fn new() -> Self {
        Self {
            audio_trigger_data: AudioTriggerData::default(),
            trigger_states: HashMap::new(),
            cached_result: ModuleEvalResult::default(),
            indices_cache: HashMap::new(),
            active_keys: std::collections::HashSet::new(),
            trigger_smoothing_state: RefCell::new(HashMap::new()),
            manual_triggers: std::collections::HashSet::new(),
            current_frame: 0,
            last_eval_time: Instant::now(),
            current_dt: 0.0,
        }
    }

    pub fn set_delta_time(&mut self, dt: f32) {
        self.current_dt = dt.min(0.5);
        self.current_frame += 1;
        self.last_eval_time = Instant::now();
        self.manual_triggers.clear();
    }

    pub fn trigger_node(&mut self, part_id: ModulePartId) {
        self.manual_triggers.insert(part_id);
    }

    pub fn update_audio(&mut self, analysis: &AudioAnalysisV2) {
        self.audio_trigger_data.band_energies = analysis.band_energies;
        self.audio_trigger_data.rms_volume = analysis.rms_volume;
        self.audio_trigger_data.peak_volume = analysis.peak_volume;
        self.audio_trigger_data.beat_detected = analysis.beat_detected;
        self.audio_trigger_data.beat_strength = analysis.beat_strength;
        self.audio_trigger_data.bpm = analysis.tempo_bpm;
    }

    pub fn update_keys(&mut self, keys: &std::collections::HashSet<String>) {
        self.active_keys = keys.clone();
    }

    fn apply_smoothing(
        &self,
        part_id: ModulePartId,
        socket_idx: usize,
        target_val: f32,
        mode: &crate::module::TriggerMappingMode,
    ) -> f32 {
        if let crate::module::TriggerMappingMode::Smoothed { attack, release } = mode {
            let state_key = (part_id, socket_idx);
            let mut cache = self.trigger_smoothing_state.borrow_mut();
            let (mut current_val, last_frame) =
                cache.get(&state_key).copied().unwrap_or((target_val, 0));
            if last_frame != self.current_frame {
                let time_constant = if target_val > current_val {
                    *attack
                } else {
                    *release
                };
                if time_constant > 0.001 {
                    let alpha = 1.0 - (-self.current_dt / time_constant).exp();
                    current_val = current_val + (target_val - current_val) * alpha;
                } else {
                    current_val = target_val;
                }
                cache.insert(state_key, (current_val, self.current_frame));
            }
            current_val
        } else {
            target_val
        }
    }

    fn get_spare_render_op(&mut self) -> RenderOp {
        self.cached_result
            .spare_render_ops
            .pop()
            .unwrap_or_else(|| RenderOp {
                output_part_id: 0,
                output_type: OutputType::Projector {
                    id: 0,
                    name: String::new(),
                    hide_cursor: false,
                    target_screen: 0,
                    show_in_preview_panel: true,
                    extra_preview_window: false,
                    output_width: 1920,
                    output_height: 1080,
                    output_fps: 60.0,
                    ndi_enabled: false,
                    ndi_stream_name: String::new(),
                },
                layer_part_id: 0,
                mesh: MeshType::default(),
                opacity: 1.0,
                blend_mode: None,
                mapping_mode: false,
                source_part_id: None,
                source_props: SourceProperties::default_identity(),
                effects: Vec::new(),
                masks: Vec::new(),
            })
    }

    pub fn evaluate(
        &mut self,
        module: &MapFlowModule,
        shared_state: &SharedMediaState,
        graph_revision: u64,
    ) -> &ModuleEvalResult {
        let mut rng = rand::rng();
        self.cached_result.clear();
        let indices_valid = if let Some(cache) = self.indices_cache.get(&module.id) {
            cache.last_revision == graph_revision
        } else {
            false
        };
        if !indices_valid {
            let mut part_index_cache = HashMap::new();
            let mut conn_index_cache = HashMap::new();
            for (idx, part) in module.parts.iter().enumerate() {
                part_index_cache.insert(part.id, idx);
            }
            for (idx, conn) in module.connections.iter().enumerate() {
                conn_index_cache
                    .entry(conn.to_part)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
            self.indices_cache.insert(
                module.id,
                Arc::new(ModuleGraphIndices {
                    part_index_cache,
                    conn_index_cache,
                    last_revision: graph_revision,
                }),
            );
        }
        let indices = self.indices_cache[&module.id].clone();
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let state = self.trigger_states.entry(part.id).or_default();
                let values = self
                    .cached_result
                    .trigger_values
                    .entry(part.id)
                    .or_default();
                values.clear();
                let manual_fired = self.manual_triggers.contains(&part.id);
                Self::compute_trigger_output(
                    trigger_type,
                    state,
                    &self.audio_trigger_data,
                    self.current_dt,
                    shared_state,
                    &self.active_keys,
                    manual_fired,
                    values,
                    &mut rng,
                );
            }
        }
        let mut trigger_inputs =
            self.compute_trigger_inputs(module, &self.cached_result.trigger_values);
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                let mut activity = 1.0;
                if part.link_data.trigger_input_enabled {
                    activity = trigger_inputs.get(&part.id).copied().unwrap_or(0.0);
                }
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    let values = self
                        .cached_result
                        .trigger_values
                        .entry(part.id)
                        .or_default();
                    values.clear();
                    values.resize(output_count, 0.0);
                    values[output_count - 1] = activity;
                }
            }
        }
        trigger_inputs = self.compute_trigger_inputs(module, &self.cached_result.trigger_values);
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Slave {
                if let Some(val) = trigger_inputs.get_mut(&part.id) {
                    if part.link_data.behavior == LinkBehavior::Inverted {
                        *val = 1.0 - (*val).clamp(0.0, 1.0);
                    }
                }
            }
        }
        let socket_inputs = self.compute_socket_inputs(module, &self.cached_result.trigger_values);
        for part in &module.parts {
            if let ModulePartType::Source(source_type) = &part.part_type {
                let trigger_value = trigger_inputs.get(&part.id).copied().unwrap_or(1.0);
                if let Some(mut cmd) =
                    self.create_source_command(source_type, trigger_value, shared_state)
                {
                    for (socket_idx, config) in &part.trigger_targets {
                        if let Some(socket_vals) = socket_inputs.get(&part.id) {
                            if let Some(&raw_val) = socket_vals.get(socket_idx) {
                                let val = self.apply_smoothing(
                                    part.id,
                                    *socket_idx,
                                    config.apply(raw_val),
                                    &config.mode,
                                );
                                match (&mut cmd, &config.target) {
                                    (
                                        SourceCommand::Bevy3DModel { position, .. },
                                        crate::module::TriggerTarget::Position3D,
                                    ) => {
                                        position[0] = val;
                                        position[1] = val;
                                        position[2] = val;
                                    }
                                    (
                                        SourceCommand::Bevy3DModel { scale, .. },
                                        crate::module::TriggerTarget::Scale3D,
                                    ) => {
                                        scale[0] = val;
                                        scale[1] = val;
                                        scale[2] = val;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    self.cached_result.source_commands.insert(part.id, cmd);
                }
            }
            if let ModulePartType::Hue(hue_node) = &part.part_type {
                let s_inputs = socket_inputs.get(&part.id);
                let brightness = s_inputs.and_then(|m| m.get(&0)).copied().unwrap_or(0.0);
                let hue = s_inputs.and_then(|m| m.get(&1)).copied();
                let strobe = s_inputs.and_then(|m| m.get(&2)).copied();
                use crate::module::HueNodeType;
                let ids = match hue_node {
                    HueNodeType::SingleLamp { id, .. } => Some(vec![id.clone()]),
                    HueNodeType::MultiLamp { ids, .. } => Some(ids.clone()),
                    HueNodeType::EntertainmentGroup { .. } => None,
                };
                self.cached_result.source_commands.insert(
                    part.id,
                    SourceCommand::HueOutput {
                        brightness,
                        hue,
                        saturation: None,
                        strobe,
                        ids,
                    },
                );
            }
        }
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                if let Some(conn_idx) = indices
                    .conn_index_cache
                    .get(&part.id)
                    .and_then(|v| v.first())
                    .copied()
                {
                    let conn = &module.connections[conn_idx];
                    if let Some(&layer_idx) = indices.part_index_cache.get(&conn.from_part) {
                        let layer_part = &module.parts[layer_idx];
                        let link_opacity =
                            trigger_inputs.get(&layer_part.id).copied().unwrap_or(1.0);
                        if let ModulePartType::Layer(layer_type) = &layer_part.part_type {
                            let (mesh, opacity, blend_mode, mapping_mode) = match layer_type {
                                LayerType::Single {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    mapping_mode,
                                    ..
                                } => (mesh, opacity, blend_mode, mapping_mode),
                                LayerType::Group {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    mapping_mode,
                                    ..
                                } => (mesh, opacity, blend_mode, mapping_mode),
                                _ => continue,
                            };
                            let mut op = self.get_spare_render_op();
                            op.output_part_id = part.id;
                            op.output_type = output_type.clone();
                            op.layer_part_id = layer_part.id;
                            op.opacity = *opacity * link_opacity;
                            op.blend_mode = *blend_mode;
                            op.mapping_mode = *mapping_mode;
                            self.trace_chain_into(layer_part.id, module, &mut op, mesh, &indices);
                            self.cached_result.render_ops.push(op);
                        }
                    }
                }
            }
        }
        &self.cached_result
    }

    fn trace_chain_into(
        &self,
        start_node_id: ModulePartId,
        module: &MapFlowModule,
        op: &mut RenderOp,
        default_mesh: &MeshType,
        indices: &ModuleGraphIndices,
    ) {
        op.effects.clear();
        op.masks.clear();
        op.source_part_id = None;
        op.source_props = SourceProperties::default_identity();
        let mut override_mesh = None;
        let mut current_id = start_node_id;
        let trigger_values = &self.cached_result.trigger_values;
        for _ in 0..50 {
            if let Some(&part_idx) = indices.part_index_cache.get(&current_id) {
                let part = &module.parts[part_idx];
                for (socket_idx, config) in &part.trigger_targets {
                    let mut trigger_val = 0.0;
                    if let Some(conn_indices) = indices.conn_index_cache.get(&current_id) {
                        for &c_idx in conn_indices {
                            let conn = &module.connections[c_idx];
                            if conn.to_socket == *socket_idx {
                                if let Some(vals) = trigger_values.get(&conn.from_part) {
                                    if let Some(v) = vals.get(conn.from_socket) {
                                        trigger_val = *v;
                                    }
                                }
                                break;
                            }
                        }
                    }
                    let val = self.apply_smoothing(
                        part.id,
                        *socket_idx,
                        config.apply(trigger_val),
                        &config.mode,
                    );
                    match &config.target {
                        crate::module::TriggerTarget::Opacity => op.source_props.opacity = val,
                        crate::module::TriggerTarget::Brightness => {
                            op.source_props.brightness = val
                        }
                        crate::module::TriggerTarget::Contrast => op.source_props.contrast = val,
                        crate::module::TriggerTarget::Saturation => {
                            op.source_props.saturation = val
                        }
                        crate::module::TriggerTarget::HueShift => op.source_props.hue_shift = val,
                        crate::module::TriggerTarget::ScaleX => op.source_props.scale_x = val,
                        crate::module::TriggerTarget::ScaleY => op.source_props.scale_y = val,
                        crate::module::TriggerTarget::Rotation => op.source_props.rotation = val,
                        crate::module::TriggerTarget::OffsetX => op.source_props.offset_x = val,
                        crate::module::TriggerTarget::OffsetY => op.source_props.offset_y = val,
                        crate::module::TriggerTarget::FlipH => {
                            op.source_props.flip_horizontal = val > 0.5
                        }
                        crate::module::TriggerTarget::FlipV => {
                            op.source_props.flip_vertical = val > 0.5
                        }
                        crate::module::TriggerTarget::Param(name) => {
                            if let Some(ModulizerType::Effect { params, .. }) =
                                op.effects.last_mut()
                            {
                                params.insert(name.clone(), val);
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Some(conn_idx) = indices
                .conn_index_cache
                .get(&current_id)
                .and_then(|v| v.first())
                .copied()
            {
                let conn = &module.connections[conn_idx];
                if let Some(&part_idx) = indices.part_index_cache.get(&conn.from_part) {
                    let part = &module.parts[part_idx];
                    match &part.part_type {
                        ModulePartType::Source(source_type) => {
                            op.source_part_id = Some(part.id);
                            let mut props = SourceProperties::default_identity();
                            match source_type {
                                SourceType::MediaFile {
                                    opacity,
                                    brightness,
                                    contrast,
                                    saturation,
                                    hue_shift,
                                    scale_x,
                                    scale_y,
                                    rotation,
                                    offset_x,
                                    offset_y,
                                    flip_horizontal,
                                    flip_vertical,
                                    ..
                                } => {
                                    props = SourceProperties {
                                        opacity: *opacity,
                                        brightness: *brightness,
                                        contrast: *contrast,
                                        saturation: *saturation,
                                        hue_shift: *hue_shift,
                                        scale_x: *scale_x,
                                        scale_y: *scale_y,
                                        rotation: *rotation,
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                        flip_horizontal: *flip_horizontal,
                                        flip_vertical: *flip_vertical,
                                    };
                                }
                                _ => {}
                            }
                            op.source_props = props;
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            op.effects.push(mod_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mask(mask_type) => {
                            op.masks.push(mask_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mesh(mesh_type) => {
                            if override_mesh.is_none() {
                                override_mesh = Some(mesh_type.clone());
                            }
                            current_id = part.id;
                        }
                        _ => break,
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        op.effects.reverse();
        op.masks.reverse();
        op.mesh = override_mesh.unwrap_or_else(|| default_mesh.clone());
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_trigger_output(
        trigger_type: &TriggerType,
        state: &mut TriggerState,
        audio_data: &AudioTriggerData,
        dt: f32,
        shared_state: &SharedMediaState,
        active_keys: &std::collections::HashSet<String>,
        manual_fired: bool,
        output: &mut Vec<f32>,
        rng: &mut impl rand::Rng,
    ) {
        let push_val_internal = |val: f32, out: &mut Vec<f32>, invert: bool| {
            let base_val = if manual_fired { 1.0 } else { val };
            let final_val = if invert {
                1.0 - base_val.clamp(0.0, 1.0)
            } else {
                base_val
            };
            out.push(final_val);
        };
        match trigger_type {
            TriggerType::AudioFFT {
                threshold,
                output_config,
                ..
            } => {
                if output_config.frequency_bands {
                    for i in 0..9 {
                        let val = audio_data.band_energies[i];
                        push_val_internal(if val > *threshold { val } else { 0.0 }, output, false);
                    }
                }
                if output_config.volume_outputs {
                    push_val_internal(audio_data.rms_volume, output, false);
                    push_val_internal(audio_data.peak_volume, output, false);
                }
                if output_config.beat_output {
                    push_val_internal(
                        if audio_data.beat_detected { 1.0 } else { 0.0 },
                        output,
                        false,
                    );
                }
            }
            TriggerType::Beat => push_val_internal(
                if audio_data.beat_detected { 1.0 } else { 0.0 },
                output,
                false,
            ),
            TriggerType::Random {
                min_interval_ms,
                max_interval_ms,
                probability,
            } => {
                if !matches!(state, TriggerState::Random { .. }) {
                    *state = TriggerState::Random {
                        timer: 0.0,
                        target: rng.random_range(
                            (*min_interval_ms as f32 / 1000.0)..=(*max_interval_ms as f32 / 1000.0),
                        ),
                    };
                }
                if let TriggerState::Random { timer, target } = state {
                    *timer += dt;
                    let mut triggered = false;
                    if *timer >= *target {
                        *timer = 0.0;
                        *target = rng.random_range(
                            (*min_interval_ms as f32 / 1000.0)..=(*max_interval_ms as f32 / 1000.0),
                        );
                        if rng.random_range(0.0..=1.0) < *probability {
                            triggered = true;
                        }
                    }
                    push_val_internal(if triggered { 1.0 } else { 0.0 }, output, false);
                }
            }
            TriggerType::Fixed { interval_ms, .. } => {
                if !matches!(state, TriggerState::Fixed { .. }) {
                    *state = TriggerState::Fixed { timer: 0.0 };
                }
                if let TriggerState::Fixed { timer } = state {
                    let interval = *interval_ms as f32 / 1000.0;
                    if interval <= 0.0 {
                        push_val_internal(1.0, output, false);
                    } else {
                        *timer += dt;
                        let mut triggered = false;
                        if *timer >= interval {
                            *timer -= interval;
                            triggered = true;
                        }
                        push_val_internal(if triggered { 1.0 } else { 0.0 }, output, false);
                    }
                }
            }
            TriggerType::Midi { channel, note, .. } => {
                if let Some(&value) = shared_state.active_midi_cc.get(&(*channel, *note)) {
                    output.push(value as f32 / 127.0);
                } else {
                    let mut active_val = 0.0;
                    for (ev_ch, ev_note, velocity) in &shared_state.active_midi_events {
                        if ev_ch == channel && ev_note == note {
                            active_val = *velocity as f32 / 127.0;
                            break;
                        }
                    }
                    output.push(active_val);
                }
            }
            TriggerType::Osc { address } => {
                let mut active_val = 0.0;
                if let Some(values) = shared_state.active_osc_messages.get(address) {
                    active_val = values.first().copied().unwrap_or(1.0);
                }
                output.push(active_val);
            }
            TriggerType::Shortcut {
                key_code,
                modifiers,
            } => {
                let is_pressed = active_keys.contains(key_code);
                let mut modifiers_match = true;
                if *modifiers & 1 != 0 && !active_keys.contains("Shift") {
                    modifiers_match = false;
                }
                if *modifiers & 2 != 0 && !active_keys.contains("Control") {
                    modifiers_match = false;
                }
                if *modifiers & 4 != 0 && !active_keys.contains("Alt") {
                    modifiers_match = false;
                }
                push_val_internal(
                    if is_pressed && modifiers_match {
                        1.0
                    } else {
                        0.0
                    },
                    output,
                    false,
                );
            }
        }
    }

    fn compute_trigger_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, f32> {
        let mut inputs = HashMap::new();
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    let current = inputs.entry(conn.to_part).or_insert(0.0);
                    *current = f32::max(*current, value);
                }
            }
        }
        inputs
    }

    fn compute_socket_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, HashMap<usize, f32>> {
        let mut inputs: HashMap<ModulePartId, HashMap<usize, f32>> = HashMap::new();
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    let part_inputs = inputs.entry(conn.to_part).or_default();
                    let current = part_inputs.entry(conn.to_socket).or_insert(0.0);
                    *current = f32::max(*current, value);
                }
            }
        }
        inputs
    }

    fn create_source_command(
        &self,
        source_type: &SourceType,
        trigger_value: f32,
        shared_state: &SharedMediaState,
    ) -> Option<SourceCommand> {
        if trigger_value < 0.1 {
            return None;
        }
        match source_type {
            SourceType::MediaFile { path, .. }
            | SourceType::VideoUni { path, .. }
            | SourceType::ImageUni { path, .. } => {
                if path.is_empty() {
                    return None;
                }
                Some(SourceCommand::PlayMedia {
                    path: path.clone(),
                    trigger_value,
                })
            }
            SourceType::VideoMulti { shared_id, .. } | SourceType::ImageMulti { shared_id, .. } => {
                shared_state
                    .get(shared_id)
                    .map(|item| SourceCommand::PlaySharedMedia {
                        id: shared_id.clone(),
                        path: item.path.clone(),
                        trigger_value,
                    })
            }
            SourceType::Shader { name, params } => Some(SourceCommand::PlayShader {
                name: name.clone(),
                params: params.clone(),
                trigger_value,
            }),
            SourceType::NdiInput { source_name } => Some(SourceCommand::NdiInput {
                source_name: source_name.clone(),
                trigger_value,
            }),
            SourceType::LiveInput { device_id } => Some(SourceCommand::LiveInput {
                device_id: *device_id,
                trigger_value,
            }),
            #[cfg(target_os = "windows")]
            SourceType::SpoutInput { sender_name } => Some(SourceCommand::SpoutInput {
                sender_name: sender_name.clone(),
                trigger_value,
            }),
            SourceType::Bevy3DModel {
                path,
                position,
                rotation,
                scale,
                ..
            } => Some(SourceCommand::Bevy3DModel {
                path: path.clone(),
                position: *position,
                rotation: *rotation,
                scale: *scale,
                trigger_value,
            }),
            _ => Some(SourceCommand::BevyInput { trigger_value }),
        }
    }
}
