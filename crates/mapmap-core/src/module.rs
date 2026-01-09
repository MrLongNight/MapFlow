use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ModuleId = u64;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapFlowModule {
    pub id: ModuleId,
    pub name: String,
    pub color: [f32; 4],
    pub parts: Vec<ModulePart>,
    pub connections: Vec<ModuleConnection>,
    pub playback_mode: ModulePlaybackMode,
}

impl MapFlowModule {
    /// Add a part to this module with proper socket configuration
    pub fn add_part(&mut self, part_type: PartType, position: (f32, f32)) -> ModulePartId {
        static NEXT_PART_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_PART_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

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
            PartType::Output => ModulePartType::Output(OutputType::Projector {
                id: 0,
                name: "Output".to_string(),
                fullscreen: false,
                hide_cursor: true,
                target_screen: 0,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 0,
                output_height: 0,
                output_fps: 60.0,
            }),
        };

        let mut part = ModulePart {
            id,
            part_type: module_part_type,
            position,
            size: None,
            link_data: NodeLinkData::default(),
            inputs: vec![],
            outputs: vec![],
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
        static NEXT_PART_ID: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(10000);
        let id = NEXT_PART_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut part = ModulePart {
            id,
            part_type,
            position,
            size: None,
            link_data: NodeLinkData::default(),
            inputs: vec![],
            outputs: vec![],
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePlaybackMode {
    TimelineDuration { duration_ms: u64 },
    LoopUntilManualSwitch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModulePart {
    pub id: ModulePartId,
    pub part_type: ModulePartType,
    pub position: (f32, f32),
    /// Custom size (width, height). If None, uses default size.
    #[serde(default)]
    pub size: Option<(f32, f32)>,
    #[serde(default)]
    pub link_data: NodeLinkData,
    pub inputs: Vec<ModuleSocket>,
    pub outputs: Vec<ModuleSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeLinkData {
    pub mode: LinkMode,
    pub behavior: LinkBehavior,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkMode {
    #[default]
    Off,
    Master,
    Slave,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkBehavior {
    #[default]
    SameAsMaster,
    Inverted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSocket {
    pub name: String,
    pub socket_type: ModuleSocketType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleSocketType {
    Trigger,
    Media,
    Effect,
    Layer,
    Output,
    Link,
}

impl ModulePart {
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
            ModulePartType::Output(_) => (
                vec![ModuleSocket {
                    name: "Layer In".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
                vec![], // No outputs - outputs are sinks
            ),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePartType {
    Trigger(TriggerType),
    Source(SourceType),
    Mask(MaskType),
    Modulizer(ModulizerType),
    Layer(LayerType),
    Mesh(MeshType),
    Output(OutputType),
}

/// Simplified part type for UI creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    Trigger,
    Source,
    Mask,
    Modulator,
    Mesh,
    Layer,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Audio FFT analysis with configurable outputs
    AudioFFT {
        band: AudioBand,
        threshold: f32,
        /// Which outputs are enabled
        output_config: AudioTriggerOutputConfig,
    },
    /// Random trigger with configurable interval and probability
    Random {
        min_interval_ms: u32,
        max_interval_ms: u32,
        probability: f32,
    },
    /// Fixed time-based trigger
    Fixed { interval_ms: u32, offset_ms: u32 },
    /// MIDI note/CC trigger
    Midi {
        device: String,
        channel: u8,
        note: u8,
    },
    /// OSC message trigger
    Osc { address: String },
    /// Keyboard shortcut trigger
    Shortcut {
        key_code: String,
        modifiers: u8, // Ctrl=1, Shift=2, Alt=4
    },
    /// Beat detection (legacy)
    Beat,
}

/// Audio frequency bands for FFT trigger
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioBand {
    SubBass,    // 20-60Hz
    Bass,       // 60-250Hz
    LowMid,     // 250-500Hz
    Mid,        // 500-2kHz
    HighMid,    // 2-4kHz
    Presence,   // 4-6kHz
    Brilliance, // 6-20kHz
    Peak,       // Peak detection
    BPM,        // Beat per minute
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    MediaFile {
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
    Shader {
        name: String,
        params: Vec<(String, f32)>,
    },
    LiveInput {
        device_id: u32,
    },
    /// NDI network video source
    NdiInput {
        /// The name of the NDI source to connect to.
        /// If None, the first available source will be used.
        source_name: Option<String>,
    },
    #[cfg(target_os = "windows")]
    SpoutInput {
        sender_name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskType {
    File { path: String },
    Shape(MaskShape),
    Gradient { angle: f32, softness: f32 },
}

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
    /// Simple quad mesh (4 corner points)
    Quad {
        tl: (f32, f32),
        tr: (f32, f32),
        br: (f32, f32),
        bl: (f32, f32),
    },
    /// Grid mesh with configurable subdivision
    Grid { rows: u32, cols: u32 },
    /// Bezier surface with control points
    BezierSurface { control_points: Vec<(f32, f32)> },
    /// Freeform polygon mesh
    Polygon { vertices: Vec<(f32, f32)> },
    /// Triangle mesh
    TriMesh,
    /// Circle/Arc for curved surfaces
    Circle { segments: u32, arc_angle: f32 },
    /// Cylinder projection (for 3D surfaces)
    Cylinder { segments: u32, height: f32 },
    /// Sphere segment (for dome projections)
    Sphere {
        lat_segments: u32,
        lon_segments: u32,
    },
    /// Custom mesh from file
    Custom { path: String },
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    MediaFile { path: String },
    Shader { path: String },
    LiveInput { source: String },
}

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
    // Basic
    Blur,
    Sharpen,
    Invert,
    Threshold,
    // Color
    Brightness,
    Contrast,
    Saturation,
    HueShift,
    Colorize,
    // Distortion
    Wave,
    Spiral,
    Pinch,
    Mirror,
    Kaleidoscope,
    // Stylize
    Pixelate,
    Halftone,
    EdgeDetect,
    Posterize,
    Glitch,
    // Composite
    RgbSplit,
    ChromaticAberration,
    VHS,
    FilmGrain,
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

fn default_mesh_quad() -> MeshType {
    MeshType::Quad {
        tl: (0.0, 0.0),
        tr: (1.0, 0.0),
        br: (1.0, 1.0),
        bl: (0.0, 1.0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerType {
    Single {
        id: u64,
        name: String,
        opacity: f32,
        blend_mode: Option<BlendModeType>,
        #[serde(default = "default_mesh_quad")]
        mesh: MeshType,
    },
    Group {
        name: String,
        opacity: f32,
        blend_mode: Option<BlendModeType>,
        #[serde(default = "default_mesh_quad")]
        mesh: MeshType,
    },
    All {
        opacity: f32,
        blend_mode: Option<BlendModeType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    /// Projector/Beamer output window
    Projector {
        /// Output ID (1-8)
        id: u64,
        /// Display name
        name: String,
        /// Enable fullscreen mode
        #[serde(default)]
        fullscreen: bool,
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
    #[cfg(target_os = "windows")]
    Spout { name: String },
}

fn default_true() -> bool {
    true
}

fn default_output_fps() -> f32 {
    60.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConnection {
    pub from_part: ModulePartId,
    pub from_socket: usize,
    pub to_part: ModulePartId,
    pub to_socket: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManager {
    modules: HashMap<ModuleId, MapFlowModule>,
    next_module_id: ModuleId,
    next_part_id: ModulePartId,
    #[serde(skip)]
    color_palette: Vec<[f32; 4]>,
    next_color_index: usize,
}

impl PartialEq for ModuleManager {
    fn eq(&self, other: &Self) -> bool {
        self.modules == other.modules
            && self.next_module_id == other.next_module_id
            && self.next_part_id == other.next_part_id
            && self.next_color_index == other.next_color_index
    }
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            next_module_id: 1,
            next_part_id: 1,
            color_palette: vec![
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
            ],
            next_color_index: 0,
        }
    }

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
        };

        self.modules.insert(id, module);
        id
    }

    pub fn delete_module(&mut self, id: ModuleId) {
        self.modules.remove(&id);
    }

    pub fn list_modules(&self) -> Vec<&MapFlowModule> {
        self.modules.values().collect()
    }

    pub fn set_module_color(&mut self, id: ModuleId, color: [f32; 4]) {
        if let Some(module) = self.modules.get_mut(&id) {
            module.color = color;
        }
    }

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
