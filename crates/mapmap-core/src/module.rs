use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ModuleId = u64;
pub type ModulePartId = u64;

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

        let (module_part_type, inputs, outputs) = match part_type {
            PartType::Trigger => {
                let output_config = AudioTriggerOutputConfig::default();
                (
                    ModulePartType::Trigger(TriggerType::AudioFFT {
                        band: AudioBand::Bass,
                        threshold: 0.5,
                        output_config: output_config.clone(),
                    }),
                    vec![], // No inputs - triggers are sources
                    output_config.generate_outputs(),
                )
            }
            PartType::Source => (
                ModulePartType::Source(SourceType::MediaFile {
                    path: String::new(),
                }),
                vec![ModuleSocket {
                    name: "Trigger In".to_string(),
                    socket_type: ModuleSocketType::Trigger,
                }],
                vec![ModuleSocket {
                    name: "Media Out".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
            ),
            PartType::Mask => (
                ModulePartType::Mask(MaskType::Shape(MaskShape::Rectangle)),
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
            PartType::Modulator => (
                ModulePartType::Modulizer(ModulizerType::Effect {
                    effect_type: EffectType::Blur,
                    params: HashMap::new(),
                }),
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

            PartType::Layer => (
                ModulePartType::Layer(LayerType::Single {
                    id: 1,
                    name: "Layer 1".to_string(),
                    opacity: 1.0,
                    blend_mode: None,
                    mesh: default_mesh_quad(),
                }),
                vec![ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
                vec![ModuleSocket {
                    name: "Layer Out".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            PartType::Output => (
                ModulePartType::Output(OutputType::Projector {
                    id: 1,
                    name: "Projector 1".to_string(),
                    fullscreen: false,
                    hide_cursor: true,
                    target_screen: 0,
                    show_in_preview_panel: true,
                    extra_preview_window: false,
                }),
                vec![ModuleSocket {
                    name: "Layer In".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
                vec![], // No outputs - outputs are sinks
            ),
        };

        let part = ModulePart {
            id,
            part_type: module_part_type,
            position,
            size: None,
            inputs,
            outputs,
        };

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

        let (inputs, outputs) = match &part_type {
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
                vec![ModuleSocket {
                    name: "Input".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
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
        };

        let part = ModulePart {
            id,
            part_type,
            position,
            size: None,
            inputs,
            outputs,
        };

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

    /// Regenerate outputs for a part based on its current configuration
    /// This is called when AudioTriggerOutputConfig changes
    pub fn update_part_outputs(&mut self, part_id: ModulePartId) {
        if let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) {
            match &part.part_type {
                ModulePartType::Trigger(TriggerType::AudioFFT { output_config, .. }) => {
                    // Remove connections to outputs that no longer exist
                    let new_output_count = output_config.generate_outputs().len();
                    self.connections.retain(|c| {
                        if c.from_part == part_id {
                            c.from_socket < new_output_count
                        } else {
                            true
                        }
                    });
                    // Regenerate outputs
                    part.outputs = output_config.generate_outputs();
                }
                _ => {} // Other part types don't need dynamic output regeneration
            }
        }
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
    pub inputs: Vec<ModuleSocket>,
    pub outputs: Vec<ModuleSocket>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulePartType {
    Trigger(TriggerType),
    Source(SourceType),
    Mask(MaskType),
    Modulizer(ModulizerType),
    Layer(LayerType),
    Output(OutputType),
}

/// Simplified part type for UI creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType {
    Trigger,
    Source,
    Mask,
    Modulator,
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
}

impl Default for AudioTriggerOutputConfig {
    fn default() -> Self {
        Self {
            frequency_bands: false, // Off by default
            volume_outputs: false,  // Off by default
            beat_output: true,      // ON by default - main use case
            bpm_output: false,      // Off by default
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

        // Fallback: if nothing is enabled, add at least beat output
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
            _ => {
                255u8.hash(&mut hasher);
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
            _ => Mesh::quad(),
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
