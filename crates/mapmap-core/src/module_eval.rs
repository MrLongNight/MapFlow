//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::audio::analyzer_v2::AudioAnalysisV2;
use crate::audio_reactive::AudioTriggerData;
use crate::module::{
    BlendModeType, LayerType, MapFlowModule, MaskType, MeshType, ModulePartId, ModulePartType,
    ModulizerType, OutputType, SourceType, TriggerType,
};
use std::collections::HashMap;

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
}

/// Module graph evaluator
pub struct ModuleEvaluator {
    /// Current trigger data from audio analysis
    audio_trigger_data: AudioTriggerData,
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

        // Step 1: Evaluate all trigger nodes
        for part in &module.parts {
            if let ModulePartType::Trigger(trigger_type) = &part.part_type {
                let values = self.evaluate_trigger(trigger_type);
                result.trigger_values.insert(part.id, values);
            }
        }

        // Step 2: Propagate trigger values through the graph
        let trigger_inputs = self.compute_trigger_inputs(module, &result.trigger_values);

        // Step 3: Generate source commands
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
        for part in &module.parts {
            if let ModulePartType::Output(output_type) = &part.part_type {
                // Find connected input (should be a Layer)
                if let Some(conn) = module.connections.iter().find(|c| c.to_part == part.id) {
                    if let Some(layer_part) = module.parts.iter().find(|p| p.id == conn.from_part) {
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

                                    result.render_ops.push(RenderOp {
                                        output_part_id: part.id,
                                        output_type: output_type.clone(),
                                        layer_part_id: layer_part.id,
                                        mesh: mesh.clone(),
                                        opacity: *opacity,
                                        blend_mode: *blend_mode,
                                        source_part_id: chain.source_id,
                                        effects: chain.effects,
                                        masks: chain.masks,
                                    });
                                }
                                LayerType::Group { .. } => {
                                    // TODO: Handle groups
                                }
                                LayerType::All { .. } => {
                                    // TODO: Handle global layers (all)
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Trace the processing input chain backwards from a start node (e.g. Layer input)
    fn trace_chain(&self, module: &MapFlowModule, start_node_id: ModulePartId) -> ProcessingChain {
        let mut effects = Vec::new();
        let mut masks = Vec::new();
        let mut source_id = None;
        let mut current_id = start_node_id;

        // Safety limit to prevent infinite loops in cyclic graphs
        for _ in 0..50 {
            if let Some(conn) = module.connections.iter().find(|c| c.to_part == current_id) {
                if let Some(part) = module.parts.iter().find(|p| p.id == conn.from_part) {
                    match &part.part_type {
                        ModulePartType::Source(_) => {
                            source_id = Some(part.id);
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
                        _ => break, // Hit something else (e.g. Trigger), chain ends
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
            effects,
            masks,
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

                // Generate values based on config
                if output_config.frequency_bands {
                    values.extend_from_slice(&self.audio_trigger_data.band_energies);
                }
                if output_config.volume_outputs {
                    values.push(self.audio_trigger_data.rms_volume);
                    values.push(self.audio_trigger_data.peak_volume);
                }
                if output_config.beat_output {
                    values.push(if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    });
                }
                if output_config.bpm_output {
                    values.push(self.audio_trigger_data.bpm.unwrap_or(0.0) / 200.0);
                    // Normalize BPM
                }

                // Fallback: if empty, use beat
                if values.is_empty() {
                    values.push(if self.audio_trigger_data.beat_detected {
                        1.0
                    } else {
                        0.0
                    });
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
                // Placeholder for random
                vec![if 0.5 < *probability { 1.0 } else { 0.0 }]
            }
            TriggerType::Fixed { .. } => {
                // Fixed triggers need timing state, return 1.0 for now
                vec![1.0]
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
            SourceType::MediaFile { path } => {
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
