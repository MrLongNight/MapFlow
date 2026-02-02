//! Module - Core Data Structure
//!
//! Defines the graph structure of a MapFlow project, including Parts (nodes),
//! Connections (edges), and their types (Source, Layer, Output, etc.).
//!
//! # Core Structures #
//!
//! - [`MapFlowModule`]: The top-level container for a visual programming graph.
//! - [`ModulePart`]: A node in the graph (Source, Filter, Output).
//! - [`ModuleConnection`]: A wire connecting two sockets.
//! - [`ModuleManager`]: Manages multiple modules (scenes).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a Module
pub type ModuleId = u64;
/// Unique identifier for a Part within a Module
pub type ModulePartId = u64;

// Default value helpers for serde
fn default_speed() -> f32 {
    1.0
}
fn default_opacity() -> f32 {
    1.0
}
fn default_contrast() -> f32 {
    1.0
}
fn default_saturation() -> f32 {
    1.0
}
fn default_scale() -> f32 {
    1.0
}

fn default_next_part_id() -> ModulePartId {
    1
}

/// Represents a complete visual programming graph (Scene/Module)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapFlowModule {
    /// Unique ID
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
    #[serde(default = "default_next_part_id")]
    pub next_part_id: ModulePartId,
}

impl MapFlowModule {
    /// Add a part to this module with proper socket configuration
    /// Note: This is now a lower-level method. Use ModuleManager::add_part_to_module instead.
    /// Add a part to this module with proper socket configuration (Internal use)
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
            PartType::Mask => ModulePartType::Mask(MaskType::Shape(MaskShape::Rectangle)),
            PartType::Modulator => ModulePartType::Modulizer(ModulizerType::Effect {
                effect_type: EffectType::Blur,
                params: std::collections::HashMap::new(),
            }),
            PartType::Mesh => ModulePartType::Mesh(MeshType::Grid { cols: 10, rows: 10 }),
            PartType::Layer => ModulePartType::Layer(LayerType::Single {
                id: 0,
                name: "Layer 1".to_string(),
                opacity: 1.0,
                blend_mode: None,
                mesh: default_mesh_quad(),
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
                // Auto-assign next available Output ID
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

        // Compute initial sockets
        let (inputs, outputs) = part.compute_sockets();
        part.inputs = inputs;
        part.outputs = outputs;

        self.parts.push(part);
        id
    }

    /// Add a part with a specific ModulePartType (for dropdown menus)
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

        // Compute initial sockets
        let (inputs, outputs) = part.compute_sockets();
        part.inputs = inputs;
        part.outputs = outputs;

        self.parts.push(part);
        id
    }

    /// Update the position of a part
    pub fn update_part_position(&mut self, part_id: ModulePartId, new_position: (f32, f32)) {
        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            part.position = new_position;
        }
    }

    /// Add a connection between two parts
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

    /// Remove a connection
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

    /// Regenerate sockets for a part based on its current configuration
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

        // Cleanup connections that are now out of bounds
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

    /// Legacy wrapper for backward compatibility (renamed from update_part_outputs)
    pub fn update_part_outputs(&mut self, part_id: ModulePartId) {
        self.update_part_sockets(part_id);
    }
}

/// Defines how the module handles time and looping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePlaybackMode {
    /// Play for a fixed duration (Phase 7)
    TimelineDuration {
        /// Duration in milliseconds
        duration_ms: u64,
    },
    /// Loop indefinitely until user switches module
    LoopUntilManualSwitch,
}

/// A node in the visual graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModulePart {
    /// Unique ID
    pub id: ModulePartId,
    /// Type and configuration data
    pub part_type: ModulePartType,
    /// 2D Position on canvas
    pub position: (f32, f32),
    /// Custom size (width, height). If None, uses default size.
    #[serde(default)]
    pub size: Option<(f32, f32)>,
    /// Link system configuration
    #[serde(default)]
    pub link_data: NodeLinkData,
    /// Input sockets
    pub inputs: Vec<ModuleSocket>,
    /// Output sockets
    pub outputs: Vec<ModuleSocket>,
    /// Trigger target configuration (Input Socket Index -> Target Parameter)
    #[serde(default)]
    pub trigger_targets: HashMap<usize, TriggerConfig>,
}

/// Target parameter for a trigger input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerTarget {
    /// No target (default)
    #[default]
    None,
    /// Opacity/Transparency
    Opacity,
    /// Brightness
    Brightness,
    /// Contrast
    Contrast,
    /// Saturation
    Saturation,
    /// Hue Shift
    HueShift,
    /// Scale X
    ScaleX,
    /// Scale Y
    ScaleY,
    /// Rotation
    Rotation,
    /// Specific Effect Parameter (by name)
    Param(String),
}

/// Mapping mode for trigger value transformation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerMappingMode {
    /// Direct mapping: output = lerp(min, max, trigger_value)
    #[default]
    Direct,
    /// Fixed value when triggered: output = max_value when trigger > threshold
    Fixed,
    /// Random value in [min, max] range when triggered
    RandomInRange,
    /// Smoothed with attack/release
    Smoothed {
        /// Attack time in seconds
        attack: f32,
        /// Release time in seconds
        release: f32,
    },
}

/// Configuration for how a trigger input maps to a target parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerConfig {
    /// Target parameter to control
    pub target: TriggerTarget,
    /// Mapping mode
    pub mode: TriggerMappingMode,
    /// Minimum output value
    pub min_value: f32,
    /// Maximum output value
    pub max_value: f32,
    /// Invert the trigger value (1 - value)
    pub invert: bool,
    /// Threshold for Fixed mode (trigger activates when value > threshold)
    pub threshold: f32,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            target: TriggerTarget::None,
            mode: TriggerMappingMode::Direct,
            min_value: 0.0,
            max_value: 1.0,
            invert: false,
            threshold: 0.5,
        }
    }
}

impl TriggerConfig {
    /// Create a config for a specific target with default mapping
    pub fn for_target(target: TriggerTarget) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Apply the mapping mode to transform the raw trigger value
    pub fn apply(&self, raw_value: f32) -> f32 {
        let value = if self.invert {
            1.0 - raw_value
        } else {
            raw_value
        };

        match &self.mode {
            TriggerMappingMode::Direct => {
                self.min_value + (self.max_value - self.min_value) * value
            }
            TriggerMappingMode::Fixed => {
                if value > self.threshold {
                    self.max_value
                } else {
                    self.min_value
                }
            }
            TriggerMappingMode::RandomInRange => {
                if value > 0.0 {
                    use rand::Rng;
                    let mut rng = rand::rng();
                    rng.random_range(self.min_value..=self.max_value)
                } else {
                    self.min_value
                }
            }
            TriggerMappingMode::Smoothed { .. } => {
                // TODO: Implement stateful smoothing
                self.min_value + (self.max_value - self.min_value) * value
            }
        }
    }
}

/// Configuration for the Link System (Master/Slave nodes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeLinkData {
    /// Link mode (Off, Master, Slave)
    pub mode: LinkMode,
    /// Behavior when linked
    pub behavior: LinkBehavior,
    /// Whether the Trigger Input socket is enabled
    pub trigger_input_enabled: bool,
}

impl Default for NodeLinkData {
    fn default() -> Self {
        Self {
            mode: LinkMode::Off,
            behavior: LinkBehavior::SameAsMaster,
            trigger_input_enabled: false,
        }
    }
}

/// Link mode for a node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkMode {
    /// Not linked
    #[default]
    Off,
    /// Controls other nodes
    Master,
    /// Controlled by another node
    Slave,
}

/// Behavior of a slave node relative to its master
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkBehavior {
    /// Same visibility/state as master
    #[default]
    SameAsMaster,
    /// Inverted visibility/state
    Inverted,
}

/// A connection point on a node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSocket {
    /// Label for the socket
    pub name: String,
    /// Data type accepted/provided
    pub socket_type: ModuleSocketType,
}

/// Type of data carried by a connection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleSocketType {
    /// Logic signal (0.0-1.0)
    Trigger,
    /// Video/Image stream
    Media,
    /// GPU shader effect
    Effect,
    /// Compositing layer
    Layer,
    /// Physical output
    Output,
    /// Link signal
    Link,
}

impl ModulePart {
    /// Calculate the current sockets based on configuration
    pub fn compute_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        let (mut inputs, mut outputs) = self.part_type.get_default_sockets();

        // Apply Link System Sockets
        // Link Out (Master)
        if self.link_data.mode == LinkMode::Master {
            outputs.push(ModuleSocket {
                name: "Link Out".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        // Link In (Slave)
        if self.link_data.mode == LinkMode::Slave {
            inputs.push(ModuleSocket {
                name: "Link In".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        // Trigger Input (Visibility Control)
        // Available if enabled, for Master or normal nodes.
        // Slave nodes rely on Link In, but technically could have both?
        // Logic: Master sends Its visibility.  It can be controlled by Trigger In.
        // Slave receives visibility.
        if self.link_data.trigger_input_enabled {
            inputs.push(ModuleSocket {
                name: "Trigger In (Vis)".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        (inputs, outputs)
    }
}

impl ModulePartType {
    /// Get the default input/output sockets for this part type
    pub fn get_default_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        match self {
            ModulePartType::Trigger(trigger_type) => {
                let outputs = match trigger_type {
                    TriggerType::AudioFFT { output_config, .. } => output_config.generate_outputs(),
                    _ => vec![ModuleSocket {
                        name: "Trigger Out".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    }],
                };
                (vec![], outputs) // No inputs - triggers are sources
            }
            ModulePartType::Source(_) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Mask(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Mask In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Modulizer(_) => (
                vec![
                    ModuleSocket {
                        name: "Media In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Layer(_) => (
                vec![
                    ModuleSocket {
                        name: "Input".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Trigger".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Output".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Output(out) => match out {
                OutputType::Hue { .. } => (
                    vec![
                        ModuleSocket {
                            name: "Layer In".to_string(),
                            socket_type: ModuleSocketType::Layer,
                        },
                        ModuleSocket {
                            name: "Trigger In".to_string(),
                            socket_type: ModuleSocketType::Trigger,
                        },
                    ],
                    vec![],
                ),
                _ => (
                    vec![ModuleSocket {
                        name: "Layer In".to_string(),
                        socket_type: ModuleSocketType::Layer,
                    }],
                    vec![], // No outputs - outputs are sinks
                ),
            },
            ModulePartType::Mesh(_) => (
                vec![
                    ModuleSocket {
                        name: "Vertex In".to_string(), // Optional vertex modification?
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Control In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Geometry Out".to_string(),
                    socket_type: ModuleSocketType::Media, // simplified
                }],
            ),
            ModulePartType::Hue(_) => (
                vec![
                    ModuleSocket {
                        name: "Brightness".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                    ModuleSocket {
                        name: "Color (RGB)".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Strobe".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![], // Hue is a sink (Output)
            ),
        }
    }
}

/// Comprehensive enum of all node types and their specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePartType {
    /// Generates logic signals (Beat, MIDI, etc.)
    Trigger(TriggerType),
    /// Generates video content (File, Camera, Shader)
    Source(SourceType),
    /// Generates grayscale masks
    Mask(MaskType),
    /// Modifies content (Effects, Blending)
    Modulizer(ModulizerType),
    /// Compositing layer
    Layer(LayerType),
    /// Geometry definition
    Mesh(MeshType),
    /// Philips Hue Integration
    Hue(HueNodeType),
    /// Final output (Projector, Network)
    Output(OutputType),
}

/// Simplified part type for UI creation (categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    /// Logic triggers
    Trigger,
    /// Video sources
    Source,
    /// Masks
    Mask,
    /// Effects and modifiers
    Modulator,
    /// Warping meshes
    Mesh,
    /// Layers
    Layer,
    /// Philips Hue
    Hue,
    /// Outputs
    Output,
}

/// Types of Philips Hue Nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueNodeType {
    /// Controls a single lamp
    SingleLamp {
        /// Lamp ID (from bridge)
        id: String,
        /// Display Name
        name: String,
        /// Brightness (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        brightness: f32,
        /// Color (RGB)
        #[serde(default = "default_hue_color")]
        color: [f32; 3],
        /// Active effect (e.g. "colorloop")
        #[serde(default)]
        effect: Option<String>,
        /// Whether the effect is currently running
        #[serde(default)]
        effect_active: bool,
    },
    /// Controls multiple specific lamps together
    MultiLamp {
        /// List of Lamp IDs
        ids: Vec<String>,
        /// Display Name
        name: String,
        /// Brightness (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        brightness: f32,
        /// Color (RGB)
        #[serde(default = "default_hue_color")]
        color: [f32; 3],
        /// Active effect (e.g. "colorloop")
        #[serde(default)]
        effect: Option<String>,
        /// Whether the effect is currently running
        #[serde(default)]
        effect_active: bool,
    },
    /// Controls a whole entertainment group
    EntertainmentGroup {
        /// Group Name/ID
        name: String,
        /// Brightness (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        brightness: f32,
        /// Color (RGB)
        #[serde(default = "default_hue_color")]
        color: [f32; 3],
        /// Active effect (e.g. "colorloop")
        #[serde(default)]
        effect: Option<String>,
        /// Whether the effect is currently running
        #[serde(default)]
        effect_active: bool,
    },
}

fn default_hue_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

/// Types of logic triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Audio FFT analysis with configurable outputs
    AudioFFT {
        /// Frequency band selection (primary)
        band: AudioBand,
        /// Activation threshold (0.0-1.0)
        threshold: f32,
        /// Which outputs are enabled
        output_config: AudioTriggerOutputConfig,
    },
    /// Random trigger with configurable interval and probability
    Random {
        /// Minimum time between triggers
        min_interval_ms: u32,
        /// Maximum time between triggers
        max_interval_ms: u32,
        /// Probability of firing (0.0-1.0)
        probability: f32,
    },
    /// Fixed time-based trigger
    Fixed {
        /// Interval in milliseconds
        interval_ms: u32,
        /// Initial delay/offset
        offset_ms: u32,
    },
    /// MIDI note/CC trigger
    Midi {
        /// Device name
        device: String,
        /// MIDI Channel
        channel: u8,
        /// Note number
        note: u8,
    },
    /// OSC message trigger
    Osc {
        /// OSC Address
        address: String,
    },
    /// Keyboard shortcut trigger
    Shortcut {
        /// Key code
        key_code: String,
        /// Modifiers bitmask
        modifiers: u8, // Ctrl=1, Shift=2, Alt=4
    },
    /// Beat detection (legacy)
    Beat,
}

/// Audio frequency bands for FFT trigger
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioBand {
    /// Sub-bass frequencies (20-60Hz)
    SubBass,
    /// Bass frequencies (60-250Hz)
    Bass,
    /// Low-mid frequencies (250-500Hz)
    LowMid,
    /// Mid frequencies (500-1kHz)
    Mid,
    /// High-mid frequencies (1-2kHz)
    HighMid,
    /// Upper-mid frequencies (2-4kHz)
    UpperMid,
    /// Presence frequencies (4-6kHz)
    Presence,
    /// Brilliance frequencies (6-12kHz)
    Brilliance,
    /// Air frequencies (12-20kHz)
    Air,
    /// Peak amplitude
    Peak,
    /// Beats per minute
    BPM,
}

/// Configuration for which outputs are enabled on an AudioFFT trigger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTriggerOutputConfig {
    /// Enable individual frequency band outputs (9 outputs)
    pub frequency_bands: bool,
    /// Enable volume outputs (RMS, Peak)
    pub volume_outputs: bool,
    /// Enable beat detection output
    pub beat_output: bool,
    /// Enable BPM output
    pub bpm_output: bool,
    /// Set of output names that should be inverted
    #[serde(default)]
    pub inverted_outputs: std::collections::HashSet<String>,
}

impl Default for AudioTriggerOutputConfig {
    fn default() -> Self {
        Self {
            frequency_bands: false, // Off by default
            volume_outputs: false,  // Off by default
            beat_output: true,      // ON by default - main use case
            bpm_output: false,      // Off by default
            inverted_outputs: std::collections::HashSet::new(),
        }
    }
}

impl AudioTriggerOutputConfig {
    /// Generate output sockets based on this configuration
    pub fn generate_outputs(&self) -> Vec<ModuleSocket> {
        let mut outputs = Vec::new();

        // Frequency band outputs (9 bands)
        if self.frequency_bands {
            outputs.push(ModuleSocket {
                name: "SubBass Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Bass Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "LowMid Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Mid Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "HighMid Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "UpperMid Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Presence Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Brilliance Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Air Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        // Volume outputs
        if self.volume_outputs {
            outputs.push(ModuleSocket {
                name: "RMS Volume".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
            outputs.push(ModuleSocket {
                name: "Peak Volume".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        // Beat output
        if self.beat_output {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        // BPM output
        if self.bpm_output {
            outputs.push(ModuleSocket {
                name: "BPM Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        // Fallback:  if nothing is enabled, add at least beat output
        if outputs.is_empty() {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        outputs
    }
}

/// Types of media sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    /// Video or Image file
    MediaFile {
        /// File path
        path: String,
        /// Playback speed multiplier (1.0 = normal)
        #[serde(default = "default_speed")]
        speed: f32,
        /// Loop playback
        #[serde(default)]
        loop_enabled: bool,
        /// Start time in seconds (for clips)
        #[serde(default)]
        start_time: f32,
        /// End time in seconds (0 = full duration)
        #[serde(default)]
        end_time: f32,
        /// Transparency/Opacity (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        opacity: f32,
        /// Blend mode for compositing
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        /// Color correction:  Brightness (-1.0 to 1.0)
        #[serde(default)]
        brightness: f32,
        /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_contrast")]
        contrast: f32,
        /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_saturation")]
        saturation: f32,
        /// Color correction: Hue shift (-180 to 180 degrees)
        #[serde(default)]
        hue_shift: f32,
        /// Transform:  Scale X
        #[serde(default = "default_scale")]
        scale_x: f32,
        /// Transform: Scale Y
        #[serde(default = "default_scale")]
        scale_y: f32,
        /// Transform: Rotation in degrees
        #[serde(default)]
        rotation: f32,
        /// Transform: Position offset X
        #[serde(default)]
        offset_x: f32,
        /// Transform: Position offset Y
        #[serde(default)]
        offset_y: f32,
        /// Target output width (None = use original resolution)
        #[serde(default)]
        target_width: Option<u32>,
        /// Target output height (None = use original resolution)
        #[serde(default)]
        target_height: Option<u32>,
        /// Target FPS override (None = use original FPS)
        #[serde(default)]
        target_fps: Option<f32>,
        /// Flip video horizontally
        #[serde(default)]
        flip_horizontal: bool,
        /// Flip video vertically
        #[serde(default)]
        flip_vertical: bool,
        /// Play video in reverse
        #[serde(default)]
        reverse_playback: bool,
    },
    /// Procedural Shader
    Shader {
        /// Shader name
        name: String,
        /// Shader parameters
        params: Vec<(String, f32)>,
    },
    /// Live Camera/Capture input
    LiveInput {
        /// Device index
        device_id: u32,
    },
    /// NDI network video source
    NdiInput {
        /// The name of the NDI source to connect to.
        /// If None, the first available source will be used.
        source_name: Option<String>,
    },
    /// Spout shared texture (Windows only)
    #[cfg(target_os = "windows")]
    SpoutInput {
        /// Sender name
        sender_name: String,
    },
    /// Single-instance Video Source (Uni)
    /// Behave like MediaFile but strictly for Video
    VideoUni {
        /// File path
        path: String,
        /// Playback speed multiplier (1.0 = normal)
        #[serde(default = "default_speed")]
        speed: f32,
        /// Loop playback
        #[serde(default)]
        loop_enabled: bool,
        /// Start time in seconds (for clips)
        #[serde(default)]
        start_time: f32,
        /// End time in seconds (0 = full duration)
        #[serde(default)]
        end_time: f32,
        /// Transparency/Opacity (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        opacity: f32,
        /// Blend mode for compositing
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        /// Color correction:  Brightness (-1.0 to 1.0)
        #[serde(default)]
        brightness: f32,
        /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_contrast")]
        contrast: f32,
        /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_saturation")]
        saturation: f32,
        /// Color correction: Hue shift (-180 to 180 degrees)
        #[serde(default)]
        hue_shift: f32,
        /// Transform:  Scale X
        #[serde(default = "default_scale")]
        scale_x: f32,
        /// Transform: Scale Y
        #[serde(default = "default_scale")]
        scale_y: f32,
        /// Transform: Rotation in degrees
        #[serde(default)]
        rotation: f32,
        /// Transform: Position offset X
        #[serde(default)]
        offset_x: f32,
        /// Transform: Position offset Y
        #[serde(default)]
        offset_y: f32,
        /// Target output width (None = use original resolution)
        #[serde(default)]
        target_width: Option<u32>,
        /// Target output height (None = use original resolution)
        #[serde(default)]
        target_height: Option<u32>,
        /// Target FPS override (None = use original FPS)
        #[serde(default)]
        target_fps: Option<f32>,
        /// Flip video horizontally
        #[serde(default)]
        flip_horizontal: bool,
        /// Flip video vertically
        #[serde(default)]
        flip_vertical: bool,
        /// Play video in reverse
        #[serde(default)]
        reverse_playback: bool,
    },
    /// Multi-instance Shared Video Source (Multi)
    /// References a shared media resource by ID
    VideoMulti {
        /// Shared Resource ID
        shared_id: String,
        /// Transparency/Opacity (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        opacity: f32,
        /// Blend mode for compositing
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        /// Color correction:  Brightness (-1.0 to 1.0)
        #[serde(default)]
        brightness: f32,
        /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_contrast")]
        contrast: f32,
        /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_saturation")]
        saturation: f32,
        /// Color correction: Hue shift (-180 to 180 degrees)
        #[serde(default)]
        hue_shift: f32,
        /// Transform:  Scale X
        #[serde(default = "default_scale")]
        scale_x: f32,
        /// Transform: Scale Y
        #[serde(default = "default_scale")]
        scale_y: f32,
        /// Transform: Rotation in degrees
        #[serde(default)]
        rotation: f32,
        /// Transform: Position offset X
        #[serde(default)]
        offset_x: f32,
        /// Transform: Position offset Y
        #[serde(default)]
        offset_y: f32,
        /// Flip video horizontally
        #[serde(default)]
        flip_horizontal: bool,
        /// Flip video vertically
        #[serde(default)]
        flip_vertical: bool,
    },
    /// Single-instance Image Source (Uni)
    ImageUni {
        /// File path
        path: String,
        /// Transparency/Opacity (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        opacity: f32,
        /// Blend mode for compositing
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        /// Color correction:  Brightness (-1.0 to 1.0)
        #[serde(default)]
        brightness: f32,
        /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_contrast")]
        contrast: f32,
        /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_saturation")]
        saturation: f32,
        /// Color correction: Hue shift (-180 to 180 degrees)
        #[serde(default)]
        hue_shift: f32,
        /// Transform:  Scale X
        #[serde(default = "default_scale")]
        scale_x: f32,
        /// Transform: Scale Y
        #[serde(default = "default_scale")]
        scale_y: f32,
        /// Transform: Rotation in degrees
        #[serde(default)]
        rotation: f32,
        /// Transform: Position offset X
        #[serde(default)]
        offset_x: f32,
        /// Transform: Position offset Y
        #[serde(default)]
        offset_y: f32,
        /// Target output width (None = use original resolution)
        #[serde(default)]
        target_width: Option<u32>,
        /// Target output height (None = use original resolution)
        #[serde(default)]
        target_height: Option<u32>,
        /// Flip video horizontally
        #[serde(default)]
        flip_horizontal: bool,
        /// Flip video vertically
        #[serde(default)]
        flip_vertical: bool,
    },
    /// Multi-instance Shared Image Source (Multi)
    ImageMulti {
        /// Shared Resource ID
        shared_id: String,
        /// Transparency/Opacity (0.0 - 1.0)
        #[serde(default = "default_opacity")]
        opacity: f32,
        /// Blend mode for compositing
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        /// Color correction:  Brightness (-1.0 to 1.0)
        #[serde(default)]
        brightness: f32,
        /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_contrast")]
        contrast: f32,
        /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
        #[serde(default = "default_saturation")]
        saturation: f32,
        /// Color correction: Hue shift (-180 to 180 degrees)
        #[serde(default)]
        hue_shift: f32,
        /// Transform:  Scale X
        #[serde(default = "default_scale")]
        scale_x: f32,
        /// Transform: Scale Y
        #[serde(default = "default_scale")]
        scale_y: f32,
        /// Transform: Rotation in degrees
        #[serde(default)]
        rotation: f32,
        /// Transform: Position offset X
        #[serde(default)]
        offset_x: f32,
        /// Transform: Position offset Y
        #[serde(default)]
        offset_y: f32,
        /// Flip video horizontally
        #[serde(default)]
        flip_horizontal: bool,
        /// Flip video vertically
        #[serde(default)]
        flip_vertical: bool,
    },
}

impl SourceType {
    /// Create a new MediaFile source with default settings
    pub fn new_media_file(path: String) -> Self {
        SourceType::MediaFile {
            path,
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
        }
    }
}

/// Types of masks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskType {
    /// Image file mask
    File {
        /// Path to file
        path: String,
    },
    /// Procedural shape mask
    Shape(MaskShape),
    /// Gradient mask
    Gradient {
        /// Angle in degrees
        angle: f32,
        /// Edge softness
        softness: f32,
    },
}

/// Procedural mask shapes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaskShape {
    /// Circle
    Circle,
    /// Rectangle
    Rectangle,
    /// Triangle
    Triangle,
    /// Star
    Star,
    /// Ellipse
    Ellipse,
}

/// Mesh types for projection mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeshType {
    /// Simple quad mesh (4 corner points)
    Quad {
        /// Top-left (x, y)
        tl: (f32, f32),
        /// Top-right (x, y)
        tr: (f32, f32),
        /// Bottom-right (x, y)
        br: (f32, f32),
        /// Bottom-left (x, y)
        bl: (f32, f32),
    },
    /// Grid mesh with configurable subdivision
    Grid {
        /// Number of rows
        rows: u32,
        /// Number of columns
        cols: u32,
    },
    /// Bezier surface with control points
    BezierSurface {
        /// Control points (x, y)
        control_points: Vec<(f32, f32)>,
    },
    /// Freeform polygon mesh
    Polygon {
        /// List of vertices (x, y)
        vertices: Vec<(f32, f32)>,
    },
    /// Triangle mesh
    TriMesh,
    /// Circle/Arc for curved surfaces
    Circle {
        /// Number of segments resolution
        segments: u32,
        /// Arc angle in radians
        arc_angle: f32,
    },
    /// Cylinder projection (for 3D surfaces)
    Cylinder {
        /// Number of segments resolution
        segments: u32,
        /// Height of the cylinder
        height: f32,
    },
    /// Sphere segment (for dome projections)
    Sphere {
        /// Latitude segments
        lat_segments: u32,
        /// Longitude segments
        lon_segments: u32,
    },
    /// Custom mesh from file
    Custom {
        /// Path to custom mesh file
        path: String,
    },
}

impl MeshType {
    fn compute_revision_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        match self {
            MeshType::Quad { tl, tr, br, bl } => {
                0u8.hash(&mut hasher); // Variant ID
                tl.0.to_bits().hash(&mut hasher);
                tl.1.to_bits().hash(&mut hasher);
                tr.0.to_bits().hash(&mut hasher);
                tr.1.to_bits().hash(&mut hasher);
                br.0.to_bits().hash(&mut hasher);
                br.1.to_bits().hash(&mut hasher);
                bl.0.to_bits().hash(&mut hasher);
                bl.1.to_bits().hash(&mut hasher);
            }
            MeshType::Grid { rows, cols } => {
                1u8.hash(&mut hasher);
                rows.hash(&mut hasher);
                cols.hash(&mut hasher);
            }
            MeshType::TriMesh => {
                2u8.hash(&mut hasher);
            }
            MeshType::Circle {
                segments,
                arc_angle,
            } => {
                3u8.hash(&mut hasher);
                segments.hash(&mut hasher);
                arc_angle.to_bits().hash(&mut hasher);
            }
            MeshType::BezierSurface { control_points } => {
                4u8.hash(&mut hasher);
                control_points.len().hash(&mut hasher);
                for (x, y) in control_points {
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                }
            }
            MeshType::Polygon { vertices } => {
                5u8.hash(&mut hasher);
                vertices.len().hash(&mut hasher);
                for (x, y) in vertices {
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                }
            }
            MeshType::Cylinder { segments, height } => {
                6u8.hash(&mut hasher);
                segments.hash(&mut hasher);
                height.to_bits().hash(&mut hasher);
            }
            MeshType::Sphere {
                lat_segments,
                lon_segments,
            } => {
                7u8.hash(&mut hasher);
                lat_segments.hash(&mut hasher);
                lon_segments.hash(&mut hasher);
            }
            MeshType::Custom { path } => {
                8u8.hash(&mut hasher);
                path.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Convert to runtime mesh
    pub fn to_mesh(&self) -> crate::mesh::Mesh {
        use crate::mesh::Mesh;
        use glam::Vec2;

        let mut mesh = match self {
            MeshType::Quad { tl, tr, br, bl } => {
                let mut mesh = Mesh::quad();
                let corners = [
                    Vec2::new(tl.0, tl.1),
                    Vec2::new(tr.0, tr.1),
                    Vec2::new(br.0, br.1),
                    Vec2::new(bl.0, bl.1),
                ];
                mesh.apply_keystone(corners);
                mesh
            }
            MeshType::Grid { rows, cols } => Mesh::create_grid(*rows, *cols),
            MeshType::TriMesh => Mesh::triangle(),
            MeshType::Circle { segments, .. } => {
                Mesh::ellipse(Vec2::new(0.5, 0.5), 0.5, 0.5, *segments)
            }
            MeshType::BezierSurface { control_points } => {
                // For Bezier surface, create a grid and warp it based on control points
                // For now, use a simple grid as a placeholder until full Bezier implementation
                if control_points.len() >= 4 {
                    // TODO: Implement proper Bezier surface interpolation
                    Mesh::create_grid(8, 8)
                } else {
                    Mesh::quad()
                }
            }
            MeshType::Polygon { vertices } => {
                // Create a triangle fan from polygon vertices
                if vertices.len() < 3 {
                    Mesh::quad()
                } else {
                    use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

                    // Calculate center point for triangle fan
                    let center = vertices
                        .iter()
                        .fold((0.0, 0.0), |acc, v| (acc.0 + v.0, acc.1 + v.1));
                    let center = (
                        center.0 / vertices.len() as f32,
                        center.1 / vertices.len() as f32,
                    );

                    let mut mesh_vertices = Vec::with_capacity(vertices.len() + 1);
                    mesh_vertices.push(MeshVertex::new(
                        Vec2::new(center.0, center.1),
                        Vec2::new(0.5, 0.5),
                    ));

                    for v in vertices {
                        mesh_vertices
                            .push(MeshVertex::new(Vec2::new(v.0, v.1), Vec2::new(v.0, v.1)));
                    }

                    // Verified: Triangle-Fan-Indices generation
                    let mut indices = Vec::with_capacity(vertices.len() * 3);
                    for i in 0..vertices.len() {
                        indices.push(0); // Center vertex
                        indices.push((i + 1) as u16); // Current outer vertex
                        indices.push(((i + 1) % vertices.len() + 1) as u16); // Next outer vertex (wrapping)
                    }

                    Mesh {
                        mesh_type: CoreMeshType::Custom,
                        vertices: mesh_vertices,
                        indices,
                        revision: 0,
                    }
                }
            }
            MeshType::Cylinder { segments, height } => {
                // Create a cylindrical mesh by wrapping a grid
                let rows = (height * 10.0).max(2.0) as u32;
                let cols = (*segments).max(3);
                Mesh::create_grid(rows, cols)
            }
            MeshType::Sphere {
                lat_segments,
                lon_segments,
            } => {
                // Create a UV sphere mesh
                use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

                let lat_segs = (*lat_segments).max(3);
                let lon_segs = (*lon_segments).max(3);

                let mut mesh_vertices = Vec::new();
                let mut indices = Vec::new();

                // Generate vertices
                for lat in 0..=lat_segs {
                    let theta = (lat as f32 / lat_segs as f32) * std::f32::consts::PI;
                    let sin_theta = theta.sin();
                    let cos_theta = theta.cos();

                    for lon in 0..=lon_segs {
                        let phi = (lon as f32 / lon_segs as f32) * std::f32::consts::TAU;
                        let _sin_phi = phi.sin();
                        let cos_phi = phi.cos();

                        let x = 0.5 + 0.5 * sin_theta * cos_phi;
                        let y = 0.5 + 0.5 * cos_theta;
                        let u = lon as f32 / lon_segs as f32;
                        let v = lat as f32 / lat_segs as f32;

                        mesh_vertices.push(MeshVertex::new(Vec2::new(x, y), Vec2::new(u, v)));
                    }
                }

                // Generate indices
                for lat in 0..lat_segs {
                    for lon in 0..lon_segs {
                        let first = (lat * (lon_segs + 1) + lon) as u16;
                        let second = first + lon_segs as u16 + 1;

                        indices.push(first);
                        indices.push(second);
                        indices.push(first + 1);

                        indices.push(second);
                        indices.push(second + 1);
                        indices.push(first + 1);
                    }
                }

                Mesh {
                    mesh_type: CoreMeshType::Custom,
                    vertices: mesh_vertices,
                    indices,
                    revision: 0,
                }
            }
            MeshType::Custom { path: _ } => {
                // TODO: Load mesh from file
                // For now, return a quad as fallback
                Mesh::quad()
            }
        };

        // Ensure revision tracks content changes (for Render Cache)
        mesh.revision = self.compute_revision_hash();
        mesh
    }
}

/// Resource definition for serialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    /// Media file
    MediaFile {
        /// File path
        path: String,
    },
    /// Shader file
    Shader {
        /// File path
        path: String,
    },
    /// Live input source
    LiveInput {
        /// Source identifier
        source: String,
    },
}

/// Types of modulizers (modifiers)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulizerType {
    /// Effect instance
    Effect {
        /// Type of effect
        effect_type: EffectType,
        /// Parameters
        #[serde(default)]
        params: HashMap<String, f32>,
    },
    /// Blend mode modifier
    BlendMode(BlendModeType),
    /// Audio reactive modifier
    AudioReactive {
        /// Audio source identifier
        source: String,
    },
}

/// Available visual effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    // Custom
    /// Custom Shader Graph
    ShaderGraph(crate::shader_graph::GraphId),
    // Basic
    /// Blur effect
    Blur,
    /// Sharpen effect
    Sharpen,
    /// Color inversion
    Invert,
    /// Luminance threshold
    Threshold,
    // Color
    /// Brightness adjustment
    Brightness,
    /// Contrast adjustment
    Contrast,
    /// Saturation adjustment
    Saturation,
    /// Hue shift
    HueShift,
    /// Color tinting
    Colorize,
    // Distortion
    /// Wave distortion
    Wave,
    /// Spiral distortion
    Spiral,
    /// Pinch distortion
    Pinch,
    /// Mirror effect
    Mirror,
    /// Kaleidoscope effect
    Kaleidoscope,
    // Stylize
    /// Pixelation effect
    Pixelate,
    /// Halftone pattern
    Halftone,
    /// Edge detection
    EdgeDetect,
    /// Color posterization
    Posterize,
    /// Digital glitch effect
    Glitch,
    // Composite
    /// RGB channel split
    RgbSplit,
    /// Chromatic aberration
    ChromaticAberration,
    /// VHS tape artifact simulation
    VHS,
    /// Film grain noise
    FilmGrain,
    /// Vignette darkening
    Vignette,
}

impl EffectType {
    /// Get all available effect types
    pub fn all() -> &'static [EffectType] {
        &[
            EffectType::Blur,
            EffectType::Sharpen,
            EffectType::Invert,
            EffectType::Threshold,
            EffectType::Brightness,
            EffectType::Contrast,
            EffectType::Saturation,
            EffectType::HueShift,
            EffectType::Colorize,
            EffectType::Wave,
            EffectType::Spiral,
            EffectType::Pinch,
            EffectType::Mirror,
            EffectType::Kaleidoscope,
            EffectType::Pixelate,
            EffectType::Halftone,
            EffectType::EdgeDetect,
            EffectType::Posterize,
            EffectType::Glitch,
            EffectType::RgbSplit,
            EffectType::ChromaticAberration,
            EffectType::VHS,
            EffectType::FilmGrain,
            EffectType::Vignette,
        ]
    }

    /// Get display name for effect
    pub fn name(&self) -> &'static str {
        match self {
            EffectType::Blur => "Blur",
            EffectType::Sharpen => "Sharpen",
            EffectType::Invert => "Invert",
            EffectType::Threshold => "Threshold",
            EffectType::Brightness => "Brightness",
            EffectType::Contrast => "Contrast",
            EffectType::Saturation => "Saturation",
            EffectType::HueShift => "Hue Shift",
            EffectType::Colorize => "Colorize",
            EffectType::Wave => "Wave",
            EffectType::Spiral => "Spiral",
            EffectType::Pinch => "Pinch",
            EffectType::Mirror => "Mirror",
            EffectType::Kaleidoscope => "Kaleidoscope",
            EffectType::Pixelate => "Pixelate",
            EffectType::Halftone => "Halftone",
            EffectType::EdgeDetect => "Edge Detect",
            EffectType::Posterize => "Posterize",
            EffectType::Glitch => "Glitch",
            EffectType::RgbSplit => "RGB Split",
            EffectType::ChromaticAberration => "Chromatic Aberration",
            EffectType::VHS => "VHS",
            EffectType::FilmGrain => "Film Grain",
            EffectType::Vignette => "Vignette",
            EffectType::ShaderGraph(_) => "Custom Shader Graph",
        }
    }
}

/// Blend mode types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlendModeType {
    /// Normal blending (no effect)
    Normal,
    /// Additive blending
    Add,
    /// Multiplicative blending
    Multiply,
    /// Screen blending
    Screen,
    /// Overlay blending
    Overlay,
    /// Difference blending
    Difference,
    /// Exclusion blending
    Exclusion,
}

impl BlendModeType {
    /// Get all available blend modes
    pub fn all() -> &'static [BlendModeType] {
        &[
            BlendModeType::Normal,
            BlendModeType::Add,
            BlendModeType::Multiply,
            BlendModeType::Screen,
            BlendModeType::Overlay,
            BlendModeType::Difference,
            BlendModeType::Exclusion,
        ]
    }

    /// Get display name of blend mode
    pub fn name(&self) -> &'static str {
        match self {
            BlendModeType::Normal => "Normal",
            BlendModeType::Add => "Add",
            BlendModeType::Multiply => "Multiply",
            BlendModeType::Screen => "Screen",
            BlendModeType::Overlay => "Overlay",
            BlendModeType::Difference => "Difference",
            BlendModeType::Exclusion => "Exclusion",
        }
    }
}

fn default_mesh_quad() -> MeshType {
    MeshType::Quad {
        tl: (0.0, 0.0),
        tr: (1.0, 0.0),
        br: (1.0, 1.0),
        bl: (0.0, 1.0),
    }
}

/// Types of compositing layers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerType {
    /// A single layer with content
    Single {
        /// Unique ID
        id: u64,
        /// Display name
        name: String,
        /// Opacity
        opacity: f32,
        /// Blend mode
        blend_mode: Option<BlendModeType>,
        /// Associated mesh geometry
        #[serde(default = "default_mesh_quad")]
        mesh: MeshType,
    },
    /// A group of layers
    Group {
        /// Display name
        name: String,
        /// Group opacity
        opacity: f32,
        /// Group blend mode
        blend_mode: Option<BlendModeType>,
        /// Associated mesh geometry
        #[serde(default = "default_mesh_quad")]
        mesh: MeshType,
    },
    /// Special layer representing "All Layers"
    All {
        /// Master opacity
        opacity: f32,
        /// Master blend mode
        blend_mode: Option<BlendModeType>,
    },
}

/// Types of final outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    /// Projector/Beamer output window
    Projector {
        /// Output ID (1-8)
        id: u64,
        /// Display name
        name: String,
        /// Hide mouse cursor on this output
        #[serde(default)]
        hide_cursor: bool,
        /// Target screen/monitor index (0 = primary, 1 = secondary, etc.)
        #[serde(default)]
        target_screen: u8,
        /// Show preview in the main UI preview panel
        #[serde(default = "default_true")]
        show_in_preview_panel: bool,
        /// Open a separate preview window for this output
        #[serde(default)]
        extra_preview_window: bool,
        /// Output resolution width (0 = use window size)
        #[serde(default)]
        output_width: u32,
        /// Output resolution height (0 = use window size)
        #[serde(default)]
        output_height: u32,
        /// Output target FPS (0.0 = unlimited/vsync)
        #[serde(default = "default_output_fps")]
        output_fps: f32,
    },
    /// NDI network video output
    NdiOutput {
        /// The broadcast name of this NDI source.
        name: String,
    },
    /// Spout shared texture output (Windows only)
    #[cfg(target_os = "windows")]
    Spout {
        /// Sender name
        name: String,
    },
    /// Philips Hue Entertainment Output
    Hue {
        /// Bridge IP address
        bridge_ip: String,
        /// Whitelisted username
        username: String,
        /// DTLS Client Key
        client_key: String,
        /// Entertainment Area ID/Name
        entertainment_area: String, // Name or ID
        /// Map of light ID (streaming ID) to normalized (X, Y) position in the virtual room (0.0-1.0)
        lamp_positions: HashMap<String, (f32, f32)>,
        /// Mapping mode
        mapping_mode: HueMappingMode,
    },
}

/// Mapping mode for Hue Entertainment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueMappingMode {
    /// Average color of whole frame
    Ambient,
    /// Spatial sampling based on lamp position
    Spatial,
    /// Strobe/Pulse on trigger
    Trigger,
}

fn default_true() -> bool {
    true
}

fn default_output_fps() -> f32 {
    60.0
}

/// Represents a connection between two modules/parts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConnection {
    /// Source part ID
    pub from_part: ModulePartId,
    /// Source socket index
    pub from_socket: usize,
    /// Target part ID
    pub to_part: ModulePartId,
    /// Target socket index
    pub to_socket: usize,
}

fn default_color_palette() -> Vec<[f32; 4]> {
    vec![
        [1.0, 0.2, 0.2, 1.0],
        [1.0, 0.5, 0.2, 1.0],
        [1.0, 1.0, 0.2, 1.0],
        [0.5, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.5, 1.0],
        [0.2, 1.0, 1.0, 1.0],
        [0.2, 0.5, 1.0, 1.0],
        [0.2, 0.2, 1.0, 1.0],
        [0.5, 0.2, 1.0, 1.0],
        [1.0, 0.2, 1.0, 1.0],
        [1.0, 0.2, 0.5, 1.0],
        [0.5, 0.5, 0.5, 1.0],
        [1.0, 0.5, 0.8, 1.0],
        [0.5, 1.0, 0.8, 1.0],
        [0.8, 0.5, 1.0, 1.0],
    ]
}

/// Type of shared media
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedMediaType {
    /// Video media
    Video,
    /// Static image media
    Image,
}

/// A shared media resource entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaItem {
    /// Unique ID
    pub id: String,
    /// File path
    pub path: String,
    /// Media Type
    pub media_type: SharedMediaType,
}

/// Registry for shared media resources
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaState {
    /// Map of ID -> Item
    pub items: HashMap<String, SharedMediaItem>,
}

impl SharedMediaState {
    /// Create a new shared media state
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Register a shared media item
    pub fn register(&mut self, id: String, path: String, media_type: SharedMediaType) {
        self.items.insert(
            id.clone(),
            SharedMediaItem {
                id,
                path,
                media_type,
            },
        );
    }

    /// Get a shared media item by ID
    pub fn get(&self, id: &str) -> Option<&SharedMediaItem> {
        self.items.get(id)
    }

    /// Unregister a shared media item
    pub fn unregister(&mut self, id: &str) {
        self.items.remove(id);
    }
}

/// Manages multiple modules (Scenes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManager {
    modules: HashMap<ModuleId, MapFlowModule>,
    next_module_id: ModuleId,
    next_part_id: ModulePartId,
    #[serde(skip, default = "default_color_palette")]
    color_palette: Vec<[f32; 4]>,
    next_color_index: usize,
    /// Shared media registry
    #[serde(default)]
    pub shared_media: SharedMediaState,
}

impl PartialEq for ModuleManager {
    fn eq(&self, other: &Self) -> bool {
        self.modules == other.modules
            && self.next_module_id == other.next_module_id
            && self.next_part_id == other.next_part_id
            && self.next_color_index == other.next_color_index
            && self.shared_media == other.shared_media
    }
}

impl ModuleManager {
    /// Create a new module manager
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            next_module_id: 1,
            next_part_id: 1,
            color_palette: default_color_palette(),
            next_color_index: 0,
            shared_media: SharedMediaState::new(),
        }
    }

    /// Add a part to a specific module
    pub fn add_part_to_module(
        &mut self,
        module_id: ModuleId,
        part_type: PartType,
        position: (f32, f32),
    ) -> Option<ModulePartId> {
        self.modules
            .get_mut(&module_id)
            .map(|module| module.add_part(part_type, position))
    }

    /// Create a new module
    pub fn create_module(&mut self, name: String) -> ModuleId {
        let id = self.next_module_id;
        self.next_module_id += 1;

        let color = self.color_palette[self.next_color_index % self.color_palette.len()];
        self.next_color_index += 1;

        let module = MapFlowModule {
            id,
            name,
            color,
            parts: Vec::new(),
            connections: Vec::new(),
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        self.modules.insert(id, module);
        id
    }

    /// Delete a module
    pub fn delete_module(&mut self, id: ModuleId) {
        self.modules.remove(&id);
    }

    /// List all modules
    pub fn list_modules(&self) -> Vec<&MapFlowModule> {
        self.modules.values().collect()
    }

    /// Set module color
    pub fn set_module_color(&mut self, id: ModuleId, color: [f32; 4]) {
        if let Some(module) = self.modules.get_mut(&id) {
            module.color = color;
        }
    }

    /// Get module by ID (mutable)
    pub fn get_module_mut(&mut self, id: ModuleId) -> Option<&mut MapFlowModule> {
        self.modules.get_mut(&id)
    }

    /// Get a module by ID (immutable)
    pub fn get_module(&self, id: ModuleId) -> Option<&MapFlowModule> {
        self.modules.get(&id)
    }

    /// Get all modules as a slice-like iterator
    pub fn modules(&self) -> Vec<&MapFlowModule> {
        self.modules.values().collect()
    }

    /// Get all modules mutably
    pub fn modules_mut(&mut self) -> Vec<&mut MapFlowModule> {
        self.modules.values_mut().collect()
    }

    /// Generate a new part ID
    pub fn next_part_id(&mut self) -> ModulePartId {
        let id = self.next_part_id;
        self.next_part_id += 1;
        id
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_trigger_output_config_defaults() {
        let config = AudioTriggerOutputConfig::default();
        let sockets = config.generate_outputs();

        // Default is just Beat Output
        assert!(sockets.iter().any(|s| s.name == "Beat Out"));
        assert!(!sockets.iter().any(|s| s.name == "BPM Out"));
        assert!(!sockets.iter().any(|s| s.name == "SubBass Out"));
    }

    #[test]
    fn test_audio_trigger_output_config_all_enabled() {
        let config = AudioTriggerOutputConfig {
            frequency_bands: true,
            volume_outputs: true,
            beat_output: true,
            bpm_output: true,
            inverted_outputs: Default::default(),
        };
        let sockets = config.generate_outputs();

        // 9 bands + 2 volume + 1 beat + 1 bpm = 13 sockets
        assert_eq!(sockets.len(), 13);
        assert!(sockets.iter().any(|s| s.name == "SubBass Out"));
        assert!(sockets.iter().any(|s| s.name == "RMS Volume"));
        assert!(sockets.iter().any(|s| s.name == "BPM Out"));
    }

    #[test]
    fn test_module_add_part_sockets() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        let part_id = module.add_part(PartType::Trigger, (0.0, 0.0));
        let part = module
            .parts
            .iter()
            .find(|p| p.id == part_id)
            .expect("Part not found");

        // Trigger (Beat) should have 1 output (Beat Out) and 0 inputs
        assert_eq!(part.outputs.len(), 1);
        assert_eq!(part.outputs[0].name, "Trigger Out");
        assert_eq!(part.inputs.len(), 0);
    }

    #[test]
    fn test_connection_management() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        let p1 = module.add_part(PartType::Trigger, (0.0, 0.0));
        let p2 = module.add_part(PartType::Layer, (100.0, 0.0));

        module.add_connection(p1, 0, p2, 1); // Connect Trigger Out to Layer Trigger In

        assert_eq!(module.connections.len(), 1);
        assert_eq!(module.connections[0].from_part, p1);
        assert_eq!(module.connections[0].to_part, p2);

        module.remove_connection(p1, 0, p2, 1);
        assert_eq!(module.connections.len(), 0);
    }

    #[test]
    fn test_socket_update_cleanup() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Create AudioFFT trigger with all bands (many outputs)
        let config = AudioTriggerOutputConfig {
            frequency_bands: true, // 9 bands
            ..Default::default()
        };
        let fft_part_type = ModulePartType::Trigger(TriggerType::AudioFFT {
            band: AudioBand::Bass,
            threshold: 0.5,
            output_config: config,
        });

        let p1 = module.add_part_with_type(fft_part_type, (0.0, 0.0));
        let p2 = module.add_part(PartType::Layer, (100.0, 0.0));

        // Connect SubBass (index 0) and Air (index 8)
        module.add_connection(p1, 0, p2, 1);
        module.add_connection(p1, 8, p2, 1);

        assert_eq!(module.connections.len(), 2);

        // Update part to disable bands (reducing outputs)
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == p1) {
            if let ModulePartType::Trigger(TriggerType::AudioFFT { output_config, .. }) =
                &mut part.part_type
            {
                output_config.frequency_bands = false;
            }
        }

        // This should trigger cleanup
        module.update_part_sockets(p1);

        assert_eq!(module.connections.len(), 1);
        assert_eq!(module.connections[0].from_socket, 0);
    }

    #[test]
    fn test_link_mode_sockets() {
        let mut part = ModulePart {
            id: 1,
            part_type: ModulePartType::Trigger(TriggerType::Beat), // Usually triggers are sources
            position: (0.0, 0.0),
            size: None,
            link_data: NodeLinkData {
                mode: LinkMode::Off,
                behavior: LinkBehavior::SameAsMaster,
                trigger_input_enabled: false,
            },
            inputs: vec![],
            outputs: vec![],
            trigger_targets: HashMap::new(),
        };

        // Case 1: Off (default)
        let (inputs, outputs) = part.compute_sockets();
        assert!(!inputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link));
        assert!(!outputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link));

        // Case 2: Master -> Should have Link Out
        part.link_data.mode = LinkMode::Master;
        let (inputs, outputs) = part.compute_sockets();
        assert!(outputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link && s.name == "Link Out"));
        assert!(!inputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link));

        // Case 3: Slave -> Should have Link In
        part.link_data.mode = LinkMode::Slave;
        let (inputs, outputs) = part.compute_sockets();
        assert!(inputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link && s.name == "Link In"));
        assert!(!outputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link));

        // Case 4: Slave with Trigger Input Enabled (should have Link In AND Trigger In)
        // Note: compute_sockets logic seems to say Trigger In is available for Master or normal,
        // but explicit check says `if self.link_data.trigger_input_enabled { inputs.push(...) }`
        // so it should be present regardless of link mode?
        // Let's verify this behavior.
        part.link_data.trigger_input_enabled = true;
        let (inputs, _) = part.compute_sockets();
        assert!(inputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Link && s.name == "Link In"));
        assert!(inputs
            .iter()
            .any(|s| s.socket_type == ModuleSocketType::Trigger && s.name == "Trigger In (Vis)"));
    }

    #[test]
    fn test_mesh_type_revision_hash() {
        let mesh1 = MeshType::Quad {
            tl: (0.0, 0.0),
            tr: (1.0, 0.0),
            br: (1.0, 1.0),
            bl: (0.0, 1.0),
        };
        let mesh2 = MeshType::Quad {
            tl: (0.0, 0.0),
            tr: (1.0, 0.0),
            br: (1.0, 1.0),
            bl: (0.0, 1.0),
        };
        let mesh3 = MeshType::Grid { rows: 10, cols: 10 };

        assert_eq!(mesh1.compute_revision_hash(), mesh2.compute_revision_hash());
        assert_ne!(mesh1.compute_revision_hash(), mesh3.compute_revision_hash());

        // Change one value
        let mesh4 = MeshType::Quad {
            tl: (0.1, 0.0),
            tr: (1.0, 0.0),
            br: (1.0, 1.0),
            bl: (0.0, 1.0),
        };
        assert_ne!(mesh1.compute_revision_hash(), mesh4.compute_revision_hash());
    }

    #[test]
    fn test_mesh_to_mesh_generation() {
        // Test Quad generation
        let quad_type = MeshType::Quad {
            tl: (0.0, 0.0),
            tr: (100.0, 0.0),
            br: (100.0, 100.0),
            bl: (0.0, 100.0),
        };
        let mesh = quad_type.to_mesh();
        assert_eq!(mesh.vertex_count(), 4);

        // Test Grid generation
        let grid_type = MeshType::Grid { rows: 2, cols: 2 };
        let grid_mesh = grid_type.to_mesh();
        // 2x2 grid has (2+1)*(2+1) = 9 vertices
        assert_eq!(grid_mesh.vertex_count(), 9);
    }

    #[test]
    fn test_module_manager_crud() {
        let mut manager = ModuleManager::new();

        // Create
        let id1 = manager.create_module("Module A".to_string());
        let id2 = manager.create_module("Module B".to_string());
        assert_ne!(id1, id2);

        // Read/List
        assert_eq!(manager.list_modules().len(), 2);
        assert_eq!(manager.get_module(id1).unwrap().name, "Module A");

        // Update (simulated via get_mut)
        if let Some(m) = manager.get_module_mut(id1) {
            m.name = "Module A Modified".to_string();
        }
        assert_eq!(manager.get_module(id1).unwrap().name, "Module A Modified");

        // Delete
        manager.delete_module(id1);
        assert_eq!(manager.list_modules().len(), 1);
        assert!(manager.get_module(id1).is_none());
    }
}

#[test]
fn test_mesh_type_polygon_indices() {
    // Create a simple square polygon
    let vertices = vec![
        (0.0, 0.0),     // Bottom-Left
        (100.0, 0.0),   // Bottom-Right
        (100.0, 100.0), // Top-Right
        (0.0, 100.0),   // Top-Left
    ];

    let polygon = MeshType::Polygon { vertices };
    let mesh = polygon.to_mesh();

    // Check vertex count: 4 original + 1 center = 5
    assert_eq!(mesh.vertices.len(), 5);

    // Check indices
    // 4 edges -> 4 triangles -> 12 indices
    assert_eq!(mesh.indices.len(), 12);

    // Verify triangle fan structure: (Center, Current, Next)
    // Center is at index 0
    // Outer vertices are at 1, 2, 3, 4

    // Triangle 1: 0, 1, 2
    assert_eq!(mesh.indices[0], 0);
    assert_eq!(mesh.indices[1], 1);
    assert_eq!(mesh.indices[2], 2);

    // Triangle 2: 0, 2, 3
    assert_eq!(mesh.indices[3], 0);
    assert_eq!(mesh.indices[4], 2);
    assert_eq!(mesh.indices[5], 3);

    // Triangle 3: 0, 3, 4
    assert_eq!(mesh.indices[6], 0);
    assert_eq!(mesh.indices[7], 3);
    assert_eq!(mesh.indices[8], 4);

    // Triangle 4: 0, 4, 1 (Closing the loop)
    assert_eq!(mesh.indices[9], 0);
    assert_eq!(mesh.indices[10], 4);
    assert_eq!(mesh.indices[11], 1);
}

#[test]
fn test_mesh_type_sphere_generation() {
    let lat = 4;
    let lon = 4;
    let sphere = MeshType::Sphere {
        lat_segments: lat,
        lon_segments: lon,
    };
    let mesh = sphere.to_mesh();

    // Vertices: (lat+1) * (lon+1) rings
    let expected_verts = (lat + 1) * (lon + 1);
    assert_eq!(mesh.vertices.len(), expected_verts as usize);

    // Indices: lat * lon * 2 triangles * 3 indices
    let expected_indices = lat * lon * 6;
    assert_eq!(mesh.indices.len(), expected_indices as usize);
}

#[test]
fn test_update_part_position() {
    let mut module = MapFlowModule {
        id: 1,
        name: "Test".to_string(),
        color: [1.0; 4],
        parts: vec![],
        connections: vec![],
        playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
        next_part_id: 1,
    };

    let pid = module.add_part(PartType::Trigger, (0.0, 0.0));
    module.update_part_position(pid, (100.0, 200.0));

    let part = module.parts.first().unwrap();
    assert_eq!(part.position, (100.0, 200.0));
}

#[test]
fn test_source_type_defaults() {
    let source = SourceType::new_media_file("video.mp4".to_string());
    if let SourceType::MediaFile {
        path,
        speed,
        loop_enabled,
        opacity,
        ..
    } = source
    {
        assert_eq!(path, "video.mp4");
        assert_eq!(speed, 1.0);
        assert!(loop_enabled);
        assert_eq!(opacity, 1.0);
    } else {
        panic!("Wrong source type created");
    }
}

#[test]
fn test_audio_trigger_output_config_fallback_enforcement() {
    let config = AudioTriggerOutputConfig {
        frequency_bands: false,
        volume_outputs: false,
        beat_output: false, // Explicitly false
        bpm_output: false,
        inverted_outputs: Default::default(),
    };
    let sockets = config.generate_outputs();

    // Should enforce at least one output (Beat Out)
    assert_eq!(sockets.len(), 1);
    assert_eq!(sockets[0].name, "Beat Out");
}

#[test]
fn test_source_type_other_variants() {
    let shader = SourceType::Shader {
        name: "Test".to_string(),
        params: vec![],
    };
    if let SourceType::Shader { name, .. } = shader {
        assert_eq!(name, "Test");
    } else {
        panic!("Wrong type");
    }

    // Just verify we can create them
    let live = SourceType::LiveInput { device_id: 1 };
    assert!(matches!(live, SourceType::LiveInput { .. }));

    let ndi = SourceType::NdiInput { source_name: None };
    assert!(matches!(ndi, SourceType::NdiInput { .. }));
}

#[test]
fn test_link_mode_sockets_with_trigger() {
    let mut part = ModulePart {
        id: 1,
        part_type: ModulePartType::Trigger(TriggerType::Beat),
        position: (0.0, 0.0),
        size: None,
        link_data: NodeLinkData {
            mode: LinkMode::Master,
            behavior: LinkBehavior::SameAsMaster,
            trigger_input_enabled: true, // ENABLED
        },
        inputs: vec![],
        outputs: vec![],
        trigger_targets: HashMap::new(),
    };

    let (inputs, outputs) = part.compute_sockets();

    // Master: Link Out + Trigger In (Vis)
    assert!(outputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Link && s.name == "Link Out"));
    assert!(inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger && s.name == "Trigger In (Vis)"));

    // Slave: Link In + Trigger In (Vis)
    part.link_data.mode = LinkMode::Slave;
    let (inputs, _) = part.compute_sockets();
    assert!(inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Link && s.name == "Link In"));
    assert!(inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger && s.name == "Trigger In (Vis)"));

    // Off: Just Trigger In (Vis) if enabled (independent of LinkMode)
    part.link_data.mode = LinkMode::Off;
    let (inputs, _) = part.compute_sockets();
    assert!(inputs
        .iter()
        .any(|s| s.socket_type == ModuleSocketType::Trigger && s.name == "Trigger In (Vis)"));
}

#[test]
fn test_effect_type_variants() {
    let all = EffectType::all();
    assert!(!all.is_empty());
    assert_eq!(all.len(), 24); // Based on the manual list

    for effect in all {
        let name = effect.name();
        assert!(!name.is_empty(), "Effect name should not be empty");

        // Check specific mappings
        match effect {
            EffectType::Blur => assert_eq!(name, "Blur"),
            EffectType::VHS => assert_eq!(name, "VHS"),
            _ => {}
        }
    }
}

#[test]
fn test_blend_mode_type_variants() {
    let all = BlendModeType::all();
    assert!(!all.is_empty());
    assert_eq!(all.len(), 7); // Based on the manual list

    for mode in all {
        let name = mode.name();
        assert!(!name.is_empty(), "Blend mode name should not be empty");

        match mode {
            BlendModeType::Normal => assert_eq!(name, "Normal"),
            BlendModeType::Add => assert_eq!(name, "Add"),
            BlendModeType::Multiply => assert_eq!(name, "Multiply"),
            BlendModeType::Screen => assert_eq!(name, "Screen"),
            BlendModeType::Overlay => assert_eq!(name, "Overlay"),
            BlendModeType::Difference => assert_eq!(name, "Difference"),
            BlendModeType::Exclusion => assert_eq!(name, "Exclusion"),
        }
    }
}

#[test]
fn test_default_hue_color_val() {
    let color = default_hue_color();
    assert_eq!(color, [1.0, 1.0, 1.0]);
}

#[test]
fn test_hue_node_serialization() {
    // Test SingleLamp
    let single = HueNodeType::SingleLamp {
        id: "1".to_string(),
        name: "Lamp 1".to_string(),
        brightness: 0.8,
        color: [1.0, 0.0, 0.0],
        effect: Some("colorloop".to_string()),
        effect_active: true,
    };

    let serialized = serde_json::to_string(&single).unwrap();
    let deserialized: HueNodeType = serde_json::from_str(&serialized).unwrap();

    if let HueNodeType::SingleLamp {
        id,
        name,
        brightness,
        color,
        effect,
        effect_active,
    } = deserialized
    {
        assert_eq!(id, "1");
        assert_eq!(name, "Lamp 1");
        assert_eq!(brightness, 0.8);
        assert_eq!(color, [1.0, 0.0, 0.0]);
        assert_eq!(effect, Some("colorloop".to_string()));
        assert!(effect_active);
    } else {
        panic!("Wrong variant deserialized");
    }

    // Test EntertainmentGroup
    let group = HueNodeType::EntertainmentGroup {
        name: "TV Area".to_string(),
        brightness: 1.0,
        color: [1.0, 1.0, 1.0],
        effect: None,
        effect_active: false,
    };

    let serialized_group = serde_json::to_string(&group).unwrap();
    let deserialized_group: HueNodeType = serde_json::from_str(&serialized_group).unwrap();

    assert!(matches!(
        deserialized_group,
        HueNodeType::EntertainmentGroup { .. }
    ));
}

#[test]
fn test_output_type_hue_serialization() {
    let mut positions = std::collections::HashMap::new();
    positions.insert("1".to_string(), (0.5, 0.5));

    let hue_output = OutputType::Hue {
        bridge_ip: "192.168.1.50".to_string(),
        username: "user123".to_string(),
        client_key: "key123".to_string(),
        entertainment_area: "area1".to_string(),
        lamp_positions: positions,
        mapping_mode: HueMappingMode::Spatial,
    };

    let serialized = serde_json::to_string(&hue_output).unwrap();
    let deserialized: OutputType = serde_json::from_str(&serialized).unwrap();

    if let OutputType::Hue {
        bridge_ip,
        lamp_positions,
        mapping_mode,
        ..
    } = deserialized
    {
        assert_eq!(bridge_ip, "192.168.1.50");
        assert_eq!(lamp_positions.get("1"), Some(&(0.5, 0.5)));
        assert_eq!(mapping_mode, HueMappingMode::Spatial);
    } else {
        panic!("Wrong output variant");
    }
}

#[cfg(test)]
mod trigger_config_tests {
    use super::*;

    #[test]
    fn test_trigger_config_direct_mapping() {
        let config = TriggerConfig {
            mode: TriggerMappingMode::Direct,
            min_value: 0.0,
            max_value: 100.0,
            invert: false,
            ..Default::default()
        };

        // 0.0 -> 0.0
        assert_eq!(config.apply(0.0), 0.0);
        // 0.5 -> 50.0
        assert_eq!(config.apply(0.5), 50.0);
        // 1.0 -> 100.0
        assert_eq!(config.apply(1.0), 100.0);
    }

    #[test]
    fn test_trigger_config_direct_inverted() {
        let config = TriggerConfig {
            mode: TriggerMappingMode::Direct,
            min_value: 0.0,
            max_value: 100.0,
            invert: true,
            ..Default::default()
        };

        // 0.0 -> 1.0 (internal) -> 100.0
        assert_eq!(config.apply(0.0), 100.0);
        // 1.0 -> 0.0 (internal) -> 0.0
        assert_eq!(config.apply(1.0), 0.0);
    }

    #[test]
    fn test_trigger_config_fixed_mode() {
        let config = TriggerConfig {
            mode: TriggerMappingMode::Fixed,
            min_value: 10.0,
            max_value: 90.0,
            threshold: 0.5,
            ..Default::default()
        };

        // Below threshold -> min_value
        assert_eq!(config.apply(0.4), 10.0);
        // Above threshold -> max_value
        assert_eq!(config.apply(0.6), 90.0);
    }

    #[test]
    fn test_trigger_config_random_mode() {
        let config = TriggerConfig {
            mode: TriggerMappingMode::RandomInRange,
            min_value: 10.0,
            max_value: 20.0,
            ..Default::default()
        };

        // If value > 0, should be random in range
        let val = config.apply(1.0);
        assert!(val >= 10.0 && val <= 20.0);

        // If value <= 0, returns min_value (implementation detail: "if value > 0.0")
        // Wait, the implementation says: if value > 0.0 { random } else { min_value }
        assert_eq!(config.apply(0.0), 10.0);
    }

    #[test]
    fn test_trigger_config_smoothed_fallback() {
        // Currently smoothed falls back to direct
        let config = TriggerConfig {
            mode: TriggerMappingMode::Smoothed {
                attack: 0.1,
                release: 0.1,
            },
            min_value: 0.0,
            max_value: 100.0,
            ..Default::default()
        };

        assert_eq!(config.apply(0.5), 50.0);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_shared_media_registry() {
        let mut state = SharedMediaState::new();

        // 1. Register
        state.register(
            "vid_1".to_string(),
            "/tmp/video.mp4".to_string(),
            SharedMediaType::Video,
        );
        assert_eq!(state.items.len(), 1);

        // 2. Get
        let item = state.get("vid_1").expect("Should find item");
        assert_eq!(item.path, "/tmp/video.mp4");
        assert_eq!(item.media_type, SharedMediaType::Video);

        // 3. Register another
        state.register(
            "img_1".to_string(),
            "/tmp/image.png".to_string(),
            SharedMediaType::Image,
        );
        assert_eq!(state.items.len(), 2);

        // 4. Unregister
        state.unregister("vid_1");
        assert_eq!(state.items.len(), 1);
        assert!(state.get("vid_1").is_none());
        assert!(state.get("img_1").is_some());
    }

    #[test]
    fn test_all_part_types_have_sockets() {
        let mut module = MapFlowModule {
            id: 1,
            name: "Test".to_string(),
            color: [1.0; 4],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::LoopUntilManualSwitch,
            next_part_id: 1,
        };

        // Iterate all PartType variants
        let part_types = [
            PartType::Trigger,
            PartType::Source,
            PartType::Mask,
            PartType::Modulator,
            PartType::Mesh,
            PartType::Layer,
            PartType::Hue,
            PartType::Output,
        ];

        for pt in part_types {
            let id = module.add_part(pt, (0.0, 0.0));
            let part = module.parts.iter().find(|p| p.id == id).unwrap();

            // Every part must have at least one socket (Input OR Output) to be useful
            let socket_count = part.inputs.len() + part.outputs.len();
            assert!(
                socket_count > 0,
                "PartType {:?} generated 0 sockets! (Inputs: {}, Outputs: {})",
                pt,
                part.inputs.len(),
                part.outputs.len()
            );
        }
    }

    #[test]
    fn test_mapflow_module_serialization_roundtrip() {
        let mut module = MapFlowModule {
            id: 42,
            name: "Complex Module".to_string(),
            color: [0.5, 0.5, 0.5, 1.0],
            parts: vec![],
            connections: vec![],
            playback_mode: ModulePlaybackMode::TimelineDuration { duration_ms: 5000 },
            next_part_id: 1,
        };

        // Add some parts
        let p1 = module.add_part(PartType::Trigger, (10.0, 10.0));
        let p2 = module.add_part(PartType::Layer, (200.0, 10.0));

        // Configure a trigger target on p2 (Layer)
        if let Some(part) = module.parts.iter_mut().find(|p| p.id == p2) {
            part.trigger_targets.insert(
                0, // Index 0
                TriggerConfig {
                    target: TriggerTarget::Opacity,
                    mode: TriggerMappingMode::Smoothed {
                        attack: 0.1,
                        release: 0.5,
                    },
                    min_value: 0.0,
                    max_value: 1.0,
                    invert: true,
                    threshold: 0.5,
                },
            );
        }

        // Add connection
        module.add_connection(p1, 0, p2, 1);

        // Serialize
        let json = serde_json::to_string(&module).expect("Serialization failed");

        // Deserialize
        let deserialized: MapFlowModule =
            serde_json::from_str(&json).expect("Deserialization failed");

        // Compare
        assert_eq!(module, deserialized);

        // Verify deep structure
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.parts.len(), 2);
        assert_eq!(deserialized.connections.len(), 1);

        // Verify Trigger Config persisted
        let p2_deser = deserialized.parts.iter().find(|p| p.id == p2).unwrap();
        let target = p2_deser.trigger_targets.get(&0).unwrap();
        match target.target {
            TriggerTarget::Opacity => {} // OK
            _ => panic!("Wrong trigger target deserialized"),
        }
        assert!(target.invert);
    }
}
