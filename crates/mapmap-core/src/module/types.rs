//! Module - Core Data Structure Types
//!
//! Defines the graph structure of a MapFlow project, including Parts (nodes),
//! Connections (edges), and their types (Source, Layer, Output, etc.).

use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a Module
pub type ModuleId = u64;
/// Unique identifier for a Part within a Module
pub type ModulePartId = u64;

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
    #[serde(default = "crate::module::config::default_next_part_id")]
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
                effect_type: EffectType::Blur,
                params: std::collections::HashMap::new(),
            }),
            PartType::Mesh => ModulePartType::Mesh(MeshType::Grid { cols: 10, rows: 10 }),
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

    pub fn update_part_position(&mut self, part_id: ModulePartId, new_position: (f32, f32)) {
        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            part.position = new_position;
        }
    }

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

/// A node in the visual graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModulePart {
    /// Unique ID
    pub id: ModulePartId,
    /// Type and configuration data
    pub part_type: ModulePartType,
    /// 2D Position on canvas
    pub position: (f32, f32),
    /// Custom size (width, height)
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

impl ModulePart {
    /// Calculate the current sockets based on configuration
    pub fn compute_sockets(&self) -> (Vec<ModuleSocket>, Vec<ModuleSocket>) {
        let (mut inputs, mut outputs) = self.part_type.get_default_sockets();

        if self.link_data.mode == LinkMode::Master {
            outputs.push(ModuleSocket {
                name: "Link Out".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        if self.link_data.mode == LinkMode::Slave {
            inputs.push(ModuleSocket {
                name: "Link In".to_string(),
                socket_type: ModuleSocketType::Link,
            });
        }

        if self.link_data.trigger_input_enabled {
            inputs.push(ModuleSocket {
                name: "Trigger In (Vis)".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        (inputs, outputs)
    }
}

/// Target parameter for a trigger input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerTarget {
    /// No target (default)
    #[default]
    None,
    Opacity,
    Brightness,
    Contrast,
    Saturation,
    HueShift,
    ScaleX,
    ScaleY,
    Rotation,
    OffsetX,
    OffsetY,
    FlipH,
    FlipV,
    /// Specific Effect Parameter (by name)
    Param(String),
}

/// Mapping mode for trigger value transformation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TriggerMappingMode {
    /// Direct mapping
    #[default]
    Direct,
    /// Fixed value when triggered
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
    /// Threshold for Fixed mode
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
    pub fn for_target(target: TriggerTarget) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

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
                    let mut rng = rand::rng();
                    rng.random_range(self.min_value..=self.max_value)
                } else {
                    self.min_value
                }
            }
            TriggerMappingMode::Smoothed { .. } => {
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
    #[default]
    Off,
    Master,
    Slave,
}

/// Behavior of a slave node relative to its master
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkBehavior {
    #[default]
    SameAsMaster,
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
    Trigger,
    Media,
    Effect,
    Layer,
    Output,
    Link,
}

/// Comprehensive enum of all node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePartType {
    Trigger(TriggerType),
    Source(SourceType),
    Mask(MaskType),
    Modulizer(ModulizerType),
    Layer(LayerType),
    Mesh(MeshType),
    Hue(HueNodeType),
    Output(OutputType),
}

impl ModulePartType {
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
                (vec![], outputs)
            }
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
            ModulePartType::Source(SourceType::BevyAtmosphere { .. })
            | ModulePartType::Source(SourceType::BevyHexGrid { .. })
            | ModulePartType::Source(SourceType::Bevy3DShape { .. })
            | ModulePartType::Source(SourceType::BevyCamera { .. }) => (
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            ModulePartType::Source(SourceType::BevyParticles { .. }) => (
                vec![ModuleSocket {
                    name: "Spawn Trigger".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
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
                    vec![],
                ),
            },
            ModulePartType::Mesh(_) => (
                vec![
                    ModuleSocket {
                        name: "Vertex In".to_string(),
                        socket_type: ModuleSocketType::Media,
                    },
                    ModuleSocket {
                        name: "Control In".to_string(),
                        socket_type: ModuleSocketType::Trigger,
                    },
                ],
                vec![ModuleSocket {
                    name: "Geometry Out".to_string(),
                    socket_type: ModuleSocketType::Media,
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
                vec![],
            ),
        }
    }
}

/// Simplified part type for UI creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    Trigger,
    Source,
    BevyParticles,
    Bevy3DShape,
    Mask,
    Modulator,
    Mesh,
    Layer,
    Hue,
    Output,
}

/// Types of Philips Hue Nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueNodeType {
    SingleLamp {
        id: String,
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        color: [f32; 3],
        #[serde(default)]
        effect: Option<String>,
        #[serde(default)]
        effect_active: bool,
    },
    MultiLamp {
        ids: Vec<String>,
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        color: [f32; 3],
        #[serde(default)]
        effect: Option<String>,
        #[serde(default)]
        effect_active: bool,
    },
    EntertainmentGroup {
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        color: [f32; 3],
        #[serde(default)]
        effect: Option<String>,
        #[serde(default)]
        effect_active: bool,
    },
}

/// Types of logic triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    AudioFFT {
        band: AudioBand,
        threshold: f32,
        output_config: AudioTriggerOutputConfig,
    },
    Random {
        min_interval_ms: u32,
        max_interval_ms: u32,
        probability: f32,
    },
    Fixed {
        interval_ms: u32,
        offset_ms: u32,
    },
    Midi {
        device: String,
        channel: u8,
        note: u8,
    },
    Osc {
        address: String,
    },
    Shortcut {
        key_code: String,
        modifiers: u8,
    },
    Beat,
}

/// Audio frequency bands for FFT trigger
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioBand {
    SubBass,
    Bass,
    LowMid,
    Mid,
    HighMid,
    UpperMid,
    Presence,
    Brilliance,
    Air,
    Peak,
    BPM,
}

/// Configuration for AudioFFT trigger outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioTriggerOutputConfig {
    pub frequency_bands: bool,
    pub volume_outputs: bool,
    pub beat_output: bool,
    pub bpm_output: bool,
    #[serde(default)]
    pub inverted_outputs: std::collections::HashSet<String>,
}

impl Default for AudioTriggerOutputConfig {
    fn default() -> Self {
        Self {
            frequency_bands: false,
            volume_outputs: false,
            beat_output: true,
            bpm_output: false,
            inverted_outputs: std::collections::HashSet::new(),
        }
    }
}

impl AudioTriggerOutputConfig {
    pub fn generate_outputs(&self) -> Vec<ModuleSocket> {
        let mut outputs = Vec::new();

        if self.frequency_bands {
            let bands = [
                "SubBass Out", "Bass Out", "LowMid Out", "Mid Out", 
                "HighMid Out", "UpperMid Out", "Presence Out", 
                "Brilliance Out", "Air Out"
            ];
            for b in bands {
                outputs.push(ModuleSocket {
                    name: b.to_string(),
                    socket_type: ModuleSocketType::Trigger,
                });
            }
        }

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

        if self.beat_output {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        if self.bpm_output {
            outputs.push(ModuleSocket {
                name: "BPM Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        if outputs.is_empty() {
            outputs.push(ModuleSocket {
                name: "Beat Out".to_string(),
                socket_type: ModuleSocketType::Trigger,
            });
        }

        outputs
    }
}

/// Types of 3D shapes available in Bevy nodes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum BevyShapeType {
    #[default]
    Cube,
    Sphere,
    Capsule,
    Torus,
    Cylinder,
    Plane,
}

/// Modes for Bevy Camera
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BevyCameraMode {
    Orbit {
        radius: f32,
        speed: f32,
        target: [f32; 3],
        height: f32,
    },
    Fly {
        speed: f32,
        sensitivity: f32,
    },
    Static {
        position: [f32; 3],
        look_at: [f32; 3],
    },
}

impl Default for BevyCameraMode {
    fn default() -> Self {
        BevyCameraMode::Orbit {
            radius: 10.0,
            speed: 20.0,
            target: [0.0, 0.0, 0.0],
            height: 2.0,
        }
    }
}

/// Types of media sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    MediaFile {
        path: String,
        #[serde(default = "crate::module::config::default_speed")]
        speed: f32,
        #[serde(default)]
        loop_enabled: bool,
        #[serde(default)]
        start_time: f32,
        #[serde(default)]
        end_time: f32,
        #[serde(default = "crate::module::config::default_opacity")]
        opacity: f32,
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        saturation: f32,
        #[serde(default)]
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_y: f32,
        #[serde(default)]
        rotation: f32,
        #[serde(default)]
        offset_x: f32,
        #[serde(default)]
        offset_y: f32,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        #[serde(default)]
        target_fps: Option<f32>,
        #[serde(default)]
        flip_horizontal: bool,
        #[serde(default)]
        flip_vertical: bool,
        #[serde(default)]
        reverse_playback: bool,
    },
    Shader {
        name: String,
        params: Vec<(String, f32)>,
    },
    LiveInput {
        device_id: u32,
    },
    NdiInput {
        source_name: Option<String>,
    },
    Bevy,
    BevyAtmosphere {
        turbidity: f32,
        rayleigh: f32,
        mie_coeff: f32,
        mie_directional_g: f32,
        sun_position: (f32, f32),
        exposure: f32,
    },
    BevyHexGrid {
        radius: f32,
        rings: u32,
        pointy_top: bool,
        spacing: f32,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: f32,
    },
    BevyParticles {
        rate: f32,
        lifetime: f32,
        speed: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        position: [f32; 3],
        rotation: [f32; 3],
    },
    Bevy3DShape {
        shape_type: BevyShapeType,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
        color: [f32; 4],
        unlit: bool,
        #[serde(default)]
        outline_width: f32,
        #[serde(default = "crate::module::config::default_white_rgba")]
        outline_color: [f32; 4],
    },
    Bevy3DModel {
        path: String,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
        color: [f32; 4],
        unlit: bool,
        #[serde(default)]
        outline_width: f32,
        #[serde(default = "crate::module::config::default_white_rgba")]
        outline_color: [f32; 4],
    },
    Bevy3DText {
        text: String,
        font_size: f32,
        color: [f32; 4],
        position: [f32; 3],
        rotation: [f32; 3],
        alignment: String,
    },
    BevyCamera {
        mode: BevyCameraMode,
        fov: f32,
        active: bool,
    },
    #[cfg(target_os = "windows")]
    SpoutInput {
        sender_name: String,
    },
    VideoUni {
        path: String,
        #[serde(default = "crate::module::config::default_speed")]
        speed: f32,
        #[serde(default)]
        loop_enabled: bool,
        #[serde(default)]
        start_time: f32,
        #[serde(default)]
        end_time: f32,
        #[serde(default = "crate::module::config::default_opacity")]
        opacity: f32,
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        saturation: f32,
        #[serde(default)]
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_y: f32,
        #[serde(default)]
        rotation: f32,
        #[serde(default)]
        offset_x: f32,
        #[serde(default)]
        offset_y: f32,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        #[serde(default)]
        target_fps: Option<f32>,
        #[serde(default)]
        flip_horizontal: bool,
        #[serde(default)]
        flip_vertical: bool,
        #[serde(default)]
        reverse_playback: bool,
    },
    VideoMulti {
        shared_id: String,
        #[serde(default = "crate::module::config::default_opacity")]
        opacity: f32,
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        saturation: f32,
        #[serde(default)]
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_y: f32,
        #[serde(default)]
        rotation: f32,
        #[serde(default)]
        offset_x: f32,
        #[serde(default)]
        offset_y: f32,
        #[serde(default)]
        flip_horizontal: bool,
        #[serde(default)]
        flip_vertical: bool,
    },
    ImageUni {
        path: String,
        #[serde(default = "crate::module::config::default_opacity")]
        opacity: f32,
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        saturation: f32,
        #[serde(default)]
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_y: f32,
        #[serde(default)]
        rotation: f32,
        #[serde(default)]
        offset_x: f32,
        #[serde(default)]
        offset_y: f32,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        #[serde(default)]
        flip_horizontal: bool,
        #[serde(default)]
        flip_vertical: bool,
    },
    ImageMulti {
        shared_id: String,
        #[serde(default = "crate::module::config::default_opacity")]
        opacity: f32,
        #[serde(default)]
        blend_mode: Option<BlendModeType>,
        #[serde(default)]
        brightness: f32,
        #[serde(default = "crate::module::config::default_contrast")]
        contrast: f32,
        #[serde(default = "crate::module::config::default_saturation")]
        saturation: f32,
        #[serde(default)]
        hue_shift: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_x: f32,
        #[serde(default = "crate::module::config::default_scale")]
        scale_y: f32,
        #[serde(default)]
        rotation: f32,
        #[serde(default)]
        offset_x: f32,
        #[serde(default)]
        offset_y: f32,
        #[serde(default)]
        flip_horizontal: bool,
        #[serde(default)]
        flip_vertical: bool,
    },
}

impl SourceType {
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
    File {
        path: String,
    },
    Shape(MaskShape),
    Gradient {
        angle: f32,
        softness: f32,
    },
}

/// Procedural mask shapes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaskShape {
    Circle,
    Rectangle,
    Triangle,
    Star,
    Ellipse,
}

/// Mesh types for projection mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeshType {
    Quad {
        tl: (f32, f32),
        tr: (f32, f32),
        br: (f32, f32),
        bl: (f32, f32),
    },
    Grid {
        rows: u32,
        cols: u32,
    },
    BezierSurface {
        control_points: Vec<(f32, f32)>,
    },
    Polygon {
        vertices: Vec<(f32, f32)>,
    },
    TriMesh,
    Circle {
        segments: u32,
        arc_angle: f32,
    },
    Cylinder {
        segments: u32,
        height: f32,
    },
    Sphere {
        lat_segments: u32,
        lon_segments: u32,
    },
    Custom {
        path: String,
    },
}

impl Default for MeshType {
    fn default() -> Self {
        Self::Quad {
            tl: (0.0, 0.0),
            tr: (1.0, 0.0),
            br: (1.0, 1.0),
            bl: (0.0, 1.0),
        }
    }
}

impl MeshType {
    pub fn compute_revision_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        match self {
            MeshType::Quad { tl, tr, br, bl } => {
                0u8.hash(&mut hasher);
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
                if control_points.len() == 16 {
                    let mut patch = crate::mesh::BezierPatch::new();
                    for (i, p) in control_points.iter().take(16).enumerate() {
                        let row = i / 4;
                        let col = i % 4;
                        patch.control_points[row][col] = Vec2::new(p.0, p.1);
                    }

                    let mut mesh = Mesh::create_grid(8, 8);
                    patch.apply_to_mesh(&mut mesh);
                    mesh
                } else {
                    Mesh::quad()
                }
            }
            MeshType::Polygon { vertices } => {
                if vertices.len() < 3 {
                    Mesh::quad()
                } else {
                    use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

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

                    let mut indices = Vec::with_capacity(vertices.len() * 3);
                    for i in 0..vertices.len() {
                        indices.push(0);
                        indices.push((i + 1) as u16);
                        indices.push(((i + 1) % vertices.len() + 1) as u16);
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
                let rows = (height * 10.0).max(2.0) as u32;
                let cols = (*segments).max(3);
                Mesh::create_grid(rows, cols)
            }
            MeshType::Sphere {
                lat_segments,
                lon_segments,
            } => {
                use crate::mesh::{MeshType as CoreMeshType, MeshVertex};

                let lat_segs = (*lat_segments).max(3);
                let lon_segs = (*lon_segments).max(3);

                let mut mesh_vertices = Vec::new();
                let mut indices = Vec::new();

                for lat in 0..=lat_segs {
                    let theta = (lat as f32 / lat_segs as f32) * std::f32::consts::PI;
                    let sin_theta = theta.sin();
                    let cos_theta = theta.cos();

                    for lon in 0..=lon_segs {
                        let phi = (lon as f32 / lon_segs as f32) * std::f32::consts::TAU;
                        let cos_phi = phi.cos();

                        let x = 0.5 + 0.5 * sin_theta * cos_phi;
                        let y = 0.5 + 0.5 * cos_theta;
                        let u = lon as f32 / lon_segs as f32;
                        let v = lat as f32 / lat_segs as f32;

                        mesh_vertices.push(MeshVertex::new(Vec2::new(x, y), Vec2::new(u, v)));
                    }
                }

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
                Mesh::quad()
            }
        };

        mesh.revision = self.compute_revision_hash();
        mesh
    }
}

/// Types of modulizers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulizerType {
    Effect {
        effect_type: EffectType,
        #[serde(default)]
        params: HashMap<String, f32>,
    },
    BlendMode(BlendModeType),
    AudioReactive {
        source: String,
    },
}

/// Available visual effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    ShaderGraph(crate::shader_graph::GraphId),
    Blur,
    Sharpen,
    Invert,
    Threshold,
    Brightness,
    Contrast,
    Saturation,
    HueShift,
    Colorize,
    Wave,
    Spiral,
    Pinch,
    Mirror,
    Kaleidoscope,
    Pixelate,
    Halftone,
    EdgeDetect,
    Posterize,
    Glitch,
    RgbSplit,
    ChromaticAberration,
    VHS,
    FilmGrain,
    Vignette,
}

impl EffectType {
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
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Difference,
    Exclusion,
}

impl BlendModeType {
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

/// Types of compositing layers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerType {
    Single {
        id: u64,
        name: String,
        opacity: f32,
        blend_mode: Option<BlendModeType>,
        #[serde(default = "crate::module::config::default_mesh_quad")]
        mesh: MeshType,
        #[serde(default)]
        mapping_mode: bool,
    },
    Group {
        name: String,
        opacity: f32,
        blend_mode: Option<BlendModeType>,
        #[serde(default = "crate::module::config::default_mesh_quad")]
        mesh: MeshType,
        #[serde(default)]
        mapping_mode: bool,
    },
    All {
        opacity: f32,
        blend_mode: Option<BlendModeType>,
    },
}

/// Types of final outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    Projector {
        id: u64,
        name: String,
        #[serde(default)]
        hide_cursor: bool,
        #[serde(default)]
        target_screen: u8,
        #[serde(default = "crate::module::config::default_true")]
        show_in_preview_panel: bool,
        #[serde(default)]
        extra_preview_window: bool,
        #[serde(default)]
        output_width: u32,
        #[serde(default)]
        output_height: u32,
        #[serde(default = "crate::module::config::default_output_fps")]
        output_fps: f32,
        #[serde(default)]
        ndi_enabled: bool,
        #[serde(default)]
        ndi_stream_name: String,
    },
    NdiOutput {
        name: String,
    },
    #[cfg(target_os = "windows")]
    Spout {
        name: String,
    },
    Hue {
        bridge_ip: String,
        username: String,
        client_key: String,
        entertainment_area: String,
        lamp_positions: HashMap<String, (f32, f32)>,
        mapping_mode: HueMappingMode,
    },
}

/// Mapping mode for Hue Entertainment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueMappingMode {
    Ambient,
    Spatial,
    Trigger,
}

/// Represents a connection between two modules/parts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConnection {
    pub from_part: ModulePartId,
    pub from_socket: usize,
    pub to_part: ModulePartId,
    pub to_socket: usize,
}

/// Type of shared media
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedMediaType {
    Video,
    Image,
}

/// A shared media resource entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaItem {
    pub id: String,
    pub path: String,
    pub media_type: SharedMediaType,
}

/// Registry for shared media resources
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SharedMediaState {
    pub items: HashMap<String, SharedMediaItem>,
}

impl SharedMediaState {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

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

    pub fn get(&self, id: &str) -> Option<&SharedMediaItem> {
        self.items.get(id)
    }

    pub fn unregister(&mut self, id: &str) {
        self.items.remove(id);
    }
}
