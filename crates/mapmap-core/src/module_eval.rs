//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::{
    BlendModeType, LayerType, LinkBehavior, LinkMode, MapFlowModule, MaskType, MeshType,
    ModulePartId, ModulePartType, ModulizerType, OutputType, SourceType, TriggerType,
};
use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

/// State for individual trigger nodes, stored in the evaluator
#[derive(Debug, Clone, Default)]
pub enum TriggerState {
    #[default]
    None,
    Random {
        /// The timestamp (in ms since start) when the next trigger is scheduled.
        next_fire_time_ms: u64,
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

/// Render operation containing all info needed to render a layer to an output
#[derive(Debug, Clone)]
pub struct RenderOp {
    /// The output node ID (Part ID)
    pub output_part_id: ModulePartId,
    /// The specific output type configuration
    pub output_type: OutputType,

    /// The layer node ID calling for this render
    pub layer_part_id: ModulePartId,
    /// The mesh geometry to use
    pub mesh: MeshType,
    /// Layer opacity
    pub opacity: f32,
    /// Layer blend mode
    pub blend_mode: Option<BlendModeType>,

    /// Source part ID (if any)
    pub source_part_id: Option<ModulePartId>,
    /// Source-specific properties (color, transform, flip)
    pub source_props: SourceProperties,
    /// Applied effects in order (Source -> Effect1 -> Effect2 -> ...)
    pub effects: Vec<ModulizerType>,
    /// Applied masks
    pub masks: Vec<MaskType>,
}

/// Evaluation result for a single frame
#[derive(Debug, Clone, Default)]
pub struct ModuleEvalResult {
    /// Trigger values: part_id -> (output_index -> value)
    pub trigger_values: HashMap<ModulePartId, Vec<f32>>,
    /// Source commands: part_id -> SourceCommand
    pub source_commands: HashMap<ModulePartId, SourceCommand>,
    /// Render operations to specific outputs
    pub render_ops: Vec<RenderOp>,
}

impl ModuleEvalResult {
    /// Clears the result for reuse, preserving capacity where possible
    pub fn clear(&mut self) {
        // Clear trigger values but keep the vectors to reuse their capacity
        for values in self.trigger_values.values_mut() {
            values.clear();
        }
        // Note: We don't remove keys from trigger_values map to reuse map capacity and vectors.
        // However, if the graph changes, we might accumulate stale keys.
        // For a fixed graph (most of the time), this is fine.
        // To be safe against memory leaks on graph changes, we could occasionally prune.
        // For now, simple reuse is a huge win.

        // Source commands are typically small (one per source), but we can clear the map
        self.source_commands.clear();
        // Render ops is a Vec, simply clear
        self.render_ops.clear();
    }
}

/// Command for a source node
#[derive(Debug, Clone)]
pub enum SourceCommand {
    /// Load and play a media file
    PlayMedia { path: String, trigger_value: f32 },
    /// Play a shader with parameters
    PlayShader {
        name: String,
        params: Vec<(String, f32)>,
        trigger_value: f32,
    },
    /// NDI input source
    NdiInput {
        source_name: Option<String>,
        trigger_value: f32,
    },
    /// Live camera input
    LiveInput { device_id: u32, trigger_value: f32 },
    #[cfg(target_os = "windows")]
    /// Spout input (Windows only)
    SpoutInput {
        sender_name: String,
        trigger_value: f32,
    },
}

/// Helper struct for chain tracing results
struct ProcessingChain {
    source_id: Option<ModulePartId>,
    source_props: SourceProperties,
    effects: Vec<ModulizerType>,
    masks: Vec<MaskType>,
    override_mesh: Option<MeshType>,
}

/// Module graph evaluator
pub struct ModuleEvaluator {
    /// Current trigger data from audio analysis
    audio_trigger_data: AudioTriggerData,
    /// Creation time for timing calculations
    start_time: Instant,
    /// Per-node state for stateful triggers (e.g., Random)
    trigger_states: HashMap<ModulePartId, TriggerState>,
    /// Reusable result buffer to avoid allocations
    cached_result: ModuleEvalResult,
}

impl Default for ModuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleEvaluator {
    /// Create a new module evaluator
    pub fn new() -> Self {
        Self {
            audio_trigger_data: AudioTriggerData::default(),
            start_time: Instant::now(),
            trigger_states: HashMap::new(),
            cached_result: ModuleEvalResult::default(),
        }
    }

    pub fn update_audio(&mut self, analysis: &AudioAnalysisV2) {
        self.audio_trigger_data.band_energies = analysis.band_energies;
        self.audio_trigger_data.rms_volume = analysis.rms_volume;
        self.audio_trigger_data.peak_volume = analysis.peak_volume;
        self.audio_trigger_data.beat_detected = analysis.beat_detected;
        self.audio_trigger_data.beat_strength = analysis.beat_strength;
        self.audio_trigger_data.bpm = analysis.tempo_bpm;
    }

    /// Evaluate a module for one frame
    /// Returns a reference to the reusable result buffer
    pub fn evaluate(&mut self, module: &MapFlowModule) -> &ModuleEvalResult {
        // Clear previous result for reuse
        self.cached_result.clear();
        // Since we cleared trigger_values via iteration (retaining keys),
        // we might have entries with empty vectors. This is fine as we will overwrite them.

        // === DIAGNOSTICS: Log module structure ===
        // (Diagnostic logging code removed for brevity/performance in hot path unless feature enabled?
        // keeping it as it was but maybe less frequently? leaving as is per instructions to preserve functionality)

        // Step 1: Evaluate all trigger nodes
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let values = self
                    .cached_result
                    .trigger_values
                    .entry(part.id)
                    .or_default();
                // Ensure vector is empty (it should be due to clear(), but for new entries it's new)
                // If it was an existing entry, clear() loop handled it.
                // But wait, if clear() loop cleared *all* values, then they are empty.
                // However, we need to be careful not to append to existing data if logic was different.
                // clear() handles it.
                Self::compute_trigger_output(
                    trigger_type,
                    &self.audio_trigger_data,
                    self.start_time,
                    values,
                );
            }
        }

        // Step 2: First propagation (Triggers -> Nodes)
        // This populates inputs for Master nodes if they use Trigger Input
        let mut trigger_inputs =
            self.compute_trigger_inputs(module, &self.cached_result.trigger_values);

        // Step 3: Process Master Links (Nodes -> Link Out)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                // Determine Master Activity
                let mut activity = 1.0; // Default active

                if part.link_data.trigger_input_enabled {
                    if let Some(&val) = trigger_inputs.get(&part.id) {
                        activity = val;
                    } else {
                        activity = 0.0;
                    }
                }

                // Write activity to Link Out socket
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    // Get/Create the buffer
                    let values = self
                        .cached_result
                        .trigger_values
                        .entry(part.id)
                        .or_default();
                    values.clear(); // Ensure clean slate even if we reused it
                    values.resize(output_count, 0.0);
                    values[output_count - 1] = activity;
                }
            }
        }

        // Step 4: Second propagation (Master Link Out -> Slave Link In)
        trigger_inputs = self.compute_trigger_inputs(module, &self.cached_result.trigger_values);

        // Step 5: Process Slave Behaviors (Invert Link Input)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Slave {
                if let Some(val) = trigger_inputs.get_mut(&part.id) {
                    if part.link_data.behavior == LinkBehavior::Inverted {
                        *val = 1.0 - (*val).clamp(0.0, 1.0);
                    }
                }
            }
        }

        // Step 6: Generate source commands
        for part in &module.parts {
            if let ModulePartType::Source(source_type) = &part.part_type {
                let trigger_value = trigger_inputs.get(&part.id).copied().unwrap_or(0.0);
                if let Some(cmd) = self.create_source_command(source_type, trigger_value) {
                    self.cached_result.source_commands.insert(part.id, cmd);
                }
            }
        }

        // Step 4: Trace Render Pipeline
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                if let Some(conn) = module.connections.iter().find(|c| c.to_part == part.id) {
                    if let Some(layer_part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                        let link_opacity =
                            trigger_inputs.get(&layer_part.id).copied().unwrap_or(1.0);

                        if let ModulePartType::Layer(layer_type) = &layer_part.part_type {
                            match layer_type {
                                LayerType::Single {
                                    mesh,
                                    opacity,
                                    blend_mode,
                                    ..
                                } => {
                                    let chain = self.trace_chain(module, layer_part.id);
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    self.cached_result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        source_props: chain.source_props,
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });
                                }
                                LayerType::Group {
                                    opacity,
                                    blend_mode,
                                    mesh,
                                    ..
                                } => {
                                    let chain = self.trace_chain(module, layer_part.id);
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    self.cached_result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        source_props: chain.source_props.clone(),
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });
                                }
                                LayerType::All { .. } => {
                                    // TODO: Handle global layers
                                }
                            }
                        }
                    } else {
                        tracing::warn!(
                            "ModuleEval: Output {} connected to non-Layer node {}",
                            part.id,
                            conn.from_part
                        );
                    }
                }
            }
        }

        &self.cached_result
    }

    /// Trace the processing input chain backwards from a start node (e.g. Layer input)
    fn trace_chain(&self, module: &MapFlowModule, start_node_id: ModulePartId) -> ProcessingChain {
        let mut effects = Vec::new();
        let mut masks = Vec::new();
        let mut override_mesh = None;
        let mut source_id = None;
        let mut source_props = SourceProperties::default_identity();
        let mut current_id = start_node_id;

        tracing::debug!("trace_chain: Starting from node {}", start_node_id);

        // Safety limit to prevent infinite loops in cyclic graphs
        for _iteration in 0..50 {
            if let Some(conn) = module.connections.iter().find(|c| c.to_part == current_id) {
                if let Some(part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                    match &part.part_type {
                        ModulePartType::Source(source_type) => {
                            source_id = Some(part.id);
                            // Extract SourceProperties from MediaFile
                            if let SourceType::MediaFile {
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
                            } = source_type
                            {
                                source_props = SourceProperties {
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
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            effects.insert(0, mod_type.clone()); // Prepend (execution order)
                            current_id = part.id;
                        }
                        ModulePartType::Mask(mask_type) => {
                            masks.insert(0, mask_type.clone());
                            current_id = part.id;
                        }
                        ModulePartType::Mesh(mesh_type) => {
                            if override_mesh.is_none() {
                                override_mesh = Some(mesh_type.clone());
                            }
                            current_id = part.id;
                        }
                        _ => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        ProcessingChain {
            source_id,
            source_props,
            effects,
            masks,
            override_mesh,
        }
    }

    /// Evaluate a trigger node and write output values to the provided buffer
    fn compute_trigger_output(
        trigger_type: &TriggerType,
        audio_data: &AudioTriggerData,
        start_time: Instant,
        output: &mut Vec<f32>,
    ) {
        match trigger_type {
            TriggerType::AudioFFT {
                band: _band,
                threshold: _threshold,
                output_config,
            } => {
                // Helper to push and optionally invert value
                let mut push_val = |name: &str, val: f32, out: &mut Vec<f32>| {
                    let inverted = output_config.inverted_outputs.contains(name);
                    let final_val = if inverted {
                        1.0 - val.clamp(0.0, 1.0)
                    } else {
                        val
                    };
                    out.push(final_val);
                };

                // Generate values based on config
                if output_config.frequency_bands {
                    let bands = [
                        "SubBass Out",
                        "Bass Out",
                        "LowMid Out",
                        "Mid Out",
                        "HighMid Out",
                        "UpperMid Out",
                        "Presence Out",
                        "Brilliance Out",
                        "Air Out",
                    ];
                    for (i, name) in bands.iter().enumerate() {
                        if i < audio_data.band_energies.len() {
                            push_val(name, audio_data.band_energies[i], output);
                        } else {
                            push_val(name, 0.0, output);
                        }
                    }
                }
                if output_config.volume_outputs {
                    push_val("RMS Volume", audio_data.rms_volume, output);
                    push_val("Peak Volume", audio_data.peak_volume, output);
                }
                if output_config.beat_output {
                    let val = if audio_data.beat_detected { 1.0 } else { 0.0 };
                    push_val("Beat Out", val, output);
                }
                if output_config.bpm_output {
                    let val = audio_data.bpm.unwrap_or(0.0) / 200.0;
                    push_val("BPM Out", val, output);
                }

                // Fallback
                if output.is_empty() {
                    let val = if audio_data.beat_detected { 1.0 } else { 0.0 };
                    let inverted = output_config.inverted_outputs.contains("Beat Out");
                    let final_val = if inverted { 1.0 - val } else { val };
                    output.push(final_val);
                }
            }
            TriggerType::Beat => {
                output.push(if audio_data.beat_detected { 1.0 } else { 0.0 });
            }
            TriggerType::Random { probability, .. } => {
                let random_value: f32 = rand::rng().random();
                output.push(if random_value < *probability {
                    1.0
                } else {
                    0.0
                });
            }
            TriggerType::Fixed {
                interval_ms,
                offset_ms,
            } => {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                let adjusted_time = elapsed_ms.saturating_sub(*offset_ms as u64);
                let interval = *interval_ms as u64;
                if interval == 0 {
                    output.push(1.0);
                } else {
                    let pulse_duration = (interval / 10).max(16);
                    let phase = adjusted_time % interval;
                    output.push(if phase < pulse_duration { 1.0 } else { 0.0 });
                }
            }
            TriggerType::Midi { .. } => {
                output.push(0.0);
            }
            TriggerType::Osc { .. } => {
                output.push(0.0);
            }
            TriggerType::Shortcut { .. } => {
                output.push(0.0);
            }
        }
    }

    /// Compute trigger input values for each part by propagating through connections
    fn compute_trigger_inputs(
        &self,
        module: &MapFlowModule,
        trigger_values: &HashMap<ModulePartId, Vec<f32>>,
    ) -> HashMap<ModulePartId, f32> {
        let mut inputs: HashMap<ModulePartId, f32> = HashMap::new();

        // For each connection, propagate the trigger value
        for conn in &module.connections {
            if let Some(values) = trigger_values.get(&conn.from_part) {
                if let Some(&value) = values.get(conn.from_socket) {
                    // Combine multiple inputs with max
                    let current = inputs.entry(conn.to_part).or_insert(0.0);
                    *current = current.max(value);
                }
            }
        }

        inputs
    }

    /// Create a source command based on source type and trigger value
    fn create_source_command(
        &self,
        source_type: &SourceType,
        trigger_value: f32,
    ) -> Option<SourceCommand> {
        // Only activate source if trigger is above threshold (0.1)
        if trigger_value < 0.1 {
            return None;
        }

        match source_type {
            SourceType::MediaFile { path, .. } => {
                if path.is_empty() {
                    return None;
                }
                Some(SourceCommand::PlayMedia {
                    path: path.clone(),
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
        }
    }
}
