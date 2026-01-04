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
    pub fn evaluate(&self, module: &MapFlowModule) -> ModuleEvalResult {
        let mut result = ModuleEvalResult::default();

        // === DIAGNOSTICS: Log module structure ===
        let output_count = module
            .parts
            .iter()
            .filter(|p| matches!(p.part_type, ModulePartType::Output(_)))
            .count();
        let layer_count = module
            .parts
            .iter()
            .filter(|p| matches!(p.part_type, ModulePartType::Layer(_)))
            .count();
        let source_count = module
            .parts
            .iter()
            .filter(|p| matches!(p.part_type, ModulePartType::Source(_)))
            .count();
        let trigger_count = module
            .parts
            .iter()
            .filter(|p| matches!(p.part_type, ModulePartType::Trigger(_)))
            .count();

        tracing::debug!(
            "ModuleEval: parts={} (outputs={}, layers={}, sources={}, triggers={}), connections={}",
            module.parts.len(),
            output_count,
            layer_count,
            source_count,
            trigger_count,
            module.connections.len()
        );

        // Step 1: Evaluate all trigger nodes
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let values = self.evaluate_trigger(trigger_type);
                result.trigger_values.insert(part.id, values);
            }
        }

        // Step 2: First propagation (Triggers -> Nodes)
        // This populates inputs for Master nodes if they use Trigger Input
        let mut trigger_inputs = self.compute_trigger_inputs(module, &result.trigger_values);

        // Step 3: Process Master Links (Nodes -> Link Out)
        for part in &module.parts {
            if part.link_data.mode == LinkMode::Master {
                // Determine Master Activity
                let mut activity = 1.0; // Default active

                if part.link_data.trigger_input_enabled {
                    // Check if we received a trigger signal
                    // If enabled but no signal connected/active, strictly it should be 0.0?
                    // "Der Layer mit master link benötigt dann ein Trigger Signal..." -> Yes.
                    if let Some(&val) = trigger_inputs.get(&part.id) {
                        activity = val;
                    } else {
                        activity = 0.0;
                    }
                }

                // Write activity to Link Out socket
                // Link Out is appended at the end of outputs list
                if !part.outputs.is_empty() {
                    let output_count = part.outputs.len();
                    // Create a vector of zeros, set last to activity
                    // Note: We assume only Link Out carries signal from a Layer/Effect node
                    // Media outputs carry textures (not modeled here in trigger_values)
                    let mut values = vec![0.0; output_count];
                    values[output_count - 1] = activity;

                    result.trigger_values.insert(part.id, values);
                }
            }
        }

        // Step 4: Second propagation (Master Link Out -> Slave Link In)
        // Re-compute to propagate Link signals to Slaves
        trigger_inputs = self.compute_trigger_inputs(module, &result.trigger_values);

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
                    result.source_commands.insert(part.id, cmd);
                }
            }
        }

        // Step 4: Trace Render Pipeline
        // Start from Output nodes and trace back to Layers, then to Sources/Effects
        let mut outputs_without_connection = Vec::new();
        let mut outputs_without_layer = Vec::new();
        let mut outputs_without_source = Vec::new();

        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                // Find connected input (should be a Layer)
                if let Some(conn) = module.connections.iter().find(|c| c.to_part == part.id) {
                    if let Some(layer_part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                        // Apply Link System Opacity (from trigger_inputs)
                        // If the Layer is a Slave or has Trigger Input enabled, its effective opacity is modulated here.
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
                                    // Trace back from Layer to find Source chain
                                    let chain = self.trace_chain(module, layer_part.id);

                                    // Use override mesh from Mesh Node if present, otherwise use Layer's internal mesh
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });

                                    if chain.source_id.is_none() {
                                        outputs_without_source.push(part.id);
                                    }
                                }
                                LayerType::Group {
                                    opacity,
                                    blend_mode,
                                    mesh,
                                    ..
                                } => {
                                    // Groups function similarly to Single layers for rendering context
                                    let chain = self.trace_chain(module, layer_part.id);
                                    let final_mesh = chain.override_mesh.unwrap_or(mesh.clone());

                                    result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: final_mesh,
                                        opacity: *opacity * link_opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });

                                    if chain.source_id.is_none() {
                                        outputs_without_source.push(part.id);
                                    }
                                }
                                LayerType::All { .. } => {
                                    // TODO: Handle global layers (all)
                                }
                            }
                        }
                    } else {
                        outputs_without_layer.push(part.id);
                        tracing::warn!(
                            "ModuleEval: Output {} connected to non-Layer node {}",
                            part.id,
                            conn.from_part
                        );
                    }
                } else {
                    outputs_without_connection.push(part.id);
                }
            }
        }

        // === DIAGNOSTICS: Log summary ===
        if result.render_ops.is_empty() && output_count > 0 {
            tracing::warn!(
                "ModuleEval: No render_ops generated despite {} output nodes!",
                output_count
            );
            if !outputs_without_connection.is_empty() {
                tracing::warn!(
                    "  → {} outputs have NO incoming connection: {:?}",
                    outputs_without_connection.len(),
                    outputs_without_connection
                );
            }
            if !outputs_without_layer.is_empty() {
                tracing::warn!(
                    "  → {} outputs connected to non-Layer nodes: {:?}",
                    outputs_without_layer.len(),
                    outputs_without_layer
                );
            }
            if !outputs_without_source.is_empty() {
                tracing::warn!(
                    "  → {} outputs have Layer but no Source connected: {:?}",
                    outputs_without_source.len(),
                    outputs_without_source
                );
            }
        } else {
            tracing::debug!(
                "ModuleEval: Generated {} render_ops",
                result.render_ops.len()
            );
        }

        result
    }

    /// Trace the processing input chain backwards from a start node (e.g. Layer input)
    fn trace_chain(&self, module: &MapFlowModule, start_node_id: ModulePartId) -> ProcessingChain {
        let mut effects = Vec::new();
        let mut masks = Vec::new();
        let mut override_mesh = None;
        let mut source_id = None;
        let mut current_id = start_node_id;

        tracing::debug!("trace_chain: Starting from node {}", start_node_id);
        tracing::debug!(
            "trace_chain: Module has {} connections",
            module.connections.len()
        );

        // Safety limit to prevent infinite loops in cyclic graphs
        for iteration in 0..50 {
            tracing::debug!(
                "trace_chain: Iteration {}, looking for connection TO node {}",
                iteration,
                current_id
            );

            if let Some(conn) = module
                .connections
                .iter()
                .find(|c| c.to_part == current_id && c.to_socket == 0)
            {
                tracing::debug!(
                    "trace_chain: Found connection from {} to {} (socket {} -> {})",
                    conn.from_part,
                    conn.to_part,
                    conn.from_socket,
                    conn.to_socket
                );

                if let Some(part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                    tracing::debug!(
                        "trace_chain: Upstream node {} is {:?}",
                        part.id,
                        std::mem::discriminant(&part.part_type)
                    );

                    match &part.part_type {
                        ModulePartType::Source(_) => {
                            source_id = Some(part.id);
                            tracing::debug!(
                                "trace_chain: Found Source node {}, chain complete!",
                                part.id
                            );
                            break;
                        }
                        ModulePartType::Modulizer(mod_type) => {
                            effects.insert(0, mod_type.clone()); // Prepend (execution order)
                            current_id = part.id;
                            tracing::debug!(
                                "trace_chain: Found Modulizer, continuing from {}",
                                part.id
                            );
                        }
                        ModulePartType::Mask(mask_type) => {
                            masks.insert(0, mask_type.clone());
                            current_id = part.id;
                            tracing::debug!("trace_chain: Found Mask, continuing from {}", part.id);
                        }
                        ModulePartType::Mesh(mesh_type) => {
                            // If we encounter a mesh node, it overrides the layer's mesh
                            // We capture the mesh and continue tracing upstream (input 0)
                            if override_mesh.is_none() {
                                override_mesh = Some(mesh_type.clone());
                            }
                            current_id = part.id;
                            tracing::debug!("trace_chain: Found Mesh, continuing from {}", part.id);
                        }
                        other => {
                            tracing::debug!(
                                "trace_chain: Found unsupported node type {:?}, breaking",
                                std::mem::discriminant(other)
                            );
                            break;
                        }
                    }
                } else {
                    tracing::debug!(
                        "trace_chain: Connection from_part {} not found in parts!",
                        conn.from_part
                    );
                    break;
                }
            } else {
                tracing::debug!("trace_chain: No connection found TO node {}", current_id);
                break;
            }
        }

        tracing::debug!(
            "trace_chain: Result - source_id={:?}, effects={}, masks={}, override_mesh={}",
            source_id,
            effects.len(),
            masks.len(),
            override_mesh.is_some()
        );

        ProcessingChain {
            source_id,
            effects,
            masks,
            override_mesh,
        }
    }

    /// Evaluate a trigger node and return output values
    fn evaluate_trigger(&self, trigger_type: &TriggerType) -> Vec<f32> {
        match trigger_type {
            TriggerType::AudioFFT {
                band: _band,
                threshold: _threshold,
                output_config,
            } => {
                let mut values = Vec::new();

                // Helper to push and optionally invert value
                let mut push_val = |name: &str, val: f32| {
                    let inverted = output_config.inverted_outputs.contains(name);
                    let final_val = if inverted {
                        1.0 - val.clamp(0.0, 1.0)
                    } else {
                        val
                    };
                    values.push(final_val);
                };

                // Generate values based on config
                // ORDER MUST MATCH AudioTriggerOutputConfig::generate_outputs
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
                        if i < self.audio_trigger_data.band_energies.len() {
                            push_val(name, self.audio_trigger_data.band_energies[i]);
                        } else {
                            push_val(name, 0.0);
                        }
                    }
                }
                if output_config.volume_outputs {
                    push_val("RMS Volume", self.audio_trigger_data.rms_volume);
                    push_val("Peak Volume", self.audio_trigger_data.peak_volume);
                }
                if output_config.beat_output {
                    let val = if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    };
                    push_val("Beat Out", val);
                }
                if output_config.bpm_output {
                    let val = self.audio_trigger_data.bpm.unwrap_or(0.0) / 200.0;
                    push_val("BPM Out", val);
                }

                // Fallback: if empty, add beat output (matches generate_outputs fallback)
                if values.is_empty() {
                    let val = if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    };
                    // Note: generate_outputs fallback uses "Beat Out" name, so we check that
                    // But effectively we just push the value.
                    // If we want to support inversion on fallback, we need to check "Beat Out"
                    let inverted = output_config.inverted_outputs.contains("Beat Out");
                    let final_val = if inverted { 1.0 - val } else { val };
                    values.push(final_val);
                }

                values
            }
            TriggerType::Beat => {
                vec![if self.audio_trigger_data.beat_detected {
                    1.0
                } else {
                    0.0
                }]
            }
            TriggerType::Random { probability, .. } => {
                // Generate random value and compare to probability
                let random_value: f32 = rand::rng().random();
                vec![if random_value < *probability {
                    1.0
                } else {
                    0.0
                }]
            }
            TriggerType::Fixed {
                interval_ms,
                offset_ms,
            } => {
                // Calculate elapsed time since start
                let elapsed_ms = self.start_time.elapsed().as_millis() as u64;
                // Apply offset and check if we're in the "on" phase
                let adjusted_time = elapsed_ms.saturating_sub(*offset_ms as u64);
                let interval = *interval_ms as u64;
                if interval == 0 {
                    vec![1.0] // Avoid division by zero
                } else {
                    // Trigger is "on" for 10% of the interval (pulse)
                    let pulse_duration = (interval / 10).max(16); // At least 16ms
                    let phase = adjusted_time % interval;
                    vec![if phase < pulse_duration { 1.0 } else { 0.0 }]
                }
            }
            TriggerType::Midi { .. } => {
                // MIDI triggers need external input
                vec![0.0]
            }
            TriggerType::Osc { .. } => {
                // OSC triggers need external input
                vec![0.0]
            }
            TriggerType::Shortcut { .. } => {
                // Shortcut triggers need keyboard input
                vec![0.0]
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
                    // Combine multiple inputs with max (or could use add/multiply)
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
