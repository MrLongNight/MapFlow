//! Main application struct definition.

use crate::media_manager_ui::MediaManagerUI;
use crate::window_manager::WindowManager;
use crossbeam_channel::Receiver;
use egui_wgpu::Renderer;
use egui_winit::State;
use mapmap_control::hue::controller::HueController;
#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;
use mapmap_control::ControlManager;
use mapmap_core::{
    audio::{analyzer_v2::AudioAnalyzerV2, backend::cpal_backend::CpalBackend},
    media_library::MediaLibrary,
    module::ModulePartId,
    AppState, History, ModuleEvaluator, RenderOp,
};
use mapmap_mcp::McpAction;
use mapmap_media::player::VideoPlayer;
use mapmap_render::{
    ColorCalibrationRenderer, Compositor, EdgeBlendRenderer, EffectChainRenderer, MeshBufferCache,
    MeshRenderer, OscillatorRenderer, QuadRenderer, TexturePool, WgpuBackend,
};
use mapmap_ui::AppUI;
use std::collections::{HashMap, VecDeque};

/// The main application state.
pub struct App {
    /// Manages all application windows.
    pub window_manager: WindowManager,

    /// The UI state.
    pub ui_state: AppUI,
    /// The application's render backend.
    pub backend: WgpuBackend,
    /// Texture pool for intermediate textures.
    pub texture_pool: TexturePool,
    /// The main compositor.
    pub _compositor: Compositor,
    /// The effect chain renderer.
    pub effect_chain_renderer: EffectChainRenderer,
    /// Dedicated effect chain renderer for sidebar previews to avoid VRAM thrashing.
    pub preview_effect_chain_renderer: EffectChainRenderer,
    /// The mesh renderer.
    pub mesh_renderer: MeshRenderer,
    /// Cache for mesh GPU buffers
    pub mesh_buffer_cache: MeshBufferCache,
    /// Quad renderer for passthrough.
    pub _quad_renderer: QuadRenderer,
    /// Final composite texture before output processing.
    pub _composite_texture: String,
    /// Ping-pong textures for layer composition.
    pub layer_ping_pong: [String; 2],
    /// The application state (project data).
    pub state: AppState,
    /// Undo/Redo history
    pub history: History,
    /// The audio backend.
    pub audio_backend: Option<CpalBackend>,
    /// The audio analyzer.
    pub audio_analyzer: AudioAnalyzerV2,
    /// List of available audio devices.
    pub audio_devices: Vec<String>,
    /// The egui context.
    pub egui_context: egui::Context,
    /// The egui state.
    pub egui_state: State,
    /// The egui renderer.
    pub egui_renderer: Renderer,
    /// Last autosave timestamp.
    pub last_autosave: std::time::Instant,
    /// Last update timestamp for delta time calculation.
    pub last_update: std::time::Instant,
    /// Application start time.
    pub start_time: std::time::Instant,
    /// Receiver for MCP commands
    pub mcp_receiver: Receiver<McpAction>,
    /// Sender for internal actions (async -> sync)
    pub action_sender: crossbeam_channel::Sender<McpAction>,
    /// Unified control manager
    pub control_manager: ControlManager,
    /// Flag to track if exit was requested
    pub exit_requested: bool,
    /// Flag to track if restart was requested
    pub restart_requested: bool,
    /// The oscillator distortion renderer.
    pub oscillator_renderer: Option<OscillatorRenderer>,
    /// A dummy texture used as input for effects when no other source is available.
    pub dummy_texture: Option<wgpu::Texture>,
    /// A view of the dummy texture.
    pub dummy_view: Option<std::sync::Arc<wgpu::TextureView>>,
    /// Module evaluator
    pub module_evaluator: ModuleEvaluator,
    /// Active media players for source nodes ((ModuleID, PartID) -> Player)
    pub media_players: HashMap<(ModulePartId, ModulePartId), (String, VideoPlayer)>,
    /// FPS calculation: accumulated frame times
    pub fps_samples: VecDeque<f32>,
    /// Current calculated FPS
    pub current_fps: f32,
    /// Current frame time in ms
    pub current_frame_time_ms: f32,
    /// System info for CPU/RAM monitoring
    pub sys_info: sysinfo::System,
    /// Last system refresh time
    pub last_sysinfo_refresh: std::time::Instant,
    /// MIDI Input Handler
    #[cfg(feature = "midi")]
    pub midi_handler: Option<MidiInputHandler>,
    /// Available MIDI ports
    #[cfg(feature = "midi")]
    pub midi_ports: Vec<String>,
    /// Selected MIDI port index
    #[cfg(feature = "midi")]
    pub selected_midi_port: Option<usize>,
    /// NDI Receivers for module sources
    #[cfg(feature = "ndi")]
    pub ndi_receivers:
        std::collections::HashMap<mapmap_core::module::ModulePartId, mapmap_io::ndi::NdiReceiver>,
    /// NDI Senders for module outputs
    #[cfg(feature = "ndi")]
    pub ndi_senders:
        std::collections::HashMap<mapmap_core::module::ModulePartId, mapmap_io::ndi::NdiSender>,

    /// Shader Graph Manager (Runtime)
    #[allow(dead_code)]
    pub shader_graph_manager: mapmap_render::ShaderGraphManager,
    /// Output assignments (OutputID -> List of Texture Names)
    pub output_assignments: std::collections::HashMap<u64, Vec<String>>,
    /// Recent Effect Configurations (User Prefs)
    pub recent_effect_configs: mapmap_core::RecentEffectConfigs,
    /// Render Operations from Module Evaluator ((ModuleID, RenderOp))
    pub render_ops: Vec<(ModulePartId, RenderOp)>,
    /// Edge blend renderer for output windows
    pub edge_blend_renderer: Option<EdgeBlendRenderer>,
    /// Color calibration renderer for output windows
    pub color_calibration_renderer: Option<ColorCalibrationRenderer>,
    /// Temporary textures for output rendering (OutputID -> Texture)
    pub output_temp_textures: std::collections::HashMap<u64, wgpu::Texture>,
    /// Cache for egui textures to avoid re-registering every frame ((ModuleId, PartId) -> (EguiId, View))
    pub preview_texture_cache:
        HashMap<(u64, u64), (egui::TextureId, std::sync::Arc<wgpu::TextureView>)>,
    /// Cache for output preview textures (OutputID -> (EguiTextureId, View))
    pub output_preview_cache: HashMap<u64, (egui::TextureId, std::sync::Arc<wgpu::TextureView>)>,
    /// Unit Quad buffers for preview rendering (Vertex, Index, IndexCount)
    pub preview_quad_buffers: (wgpu::Buffer, wgpu::Buffer, u32),
    /// Philips Hue Controller
    pub hue_controller: HueController,
    /// Tokio runtime for async operations
    pub tokio_runtime: tokio::runtime::Runtime,
    /// Media Manager UI
    pub media_manager_ui: MediaManagerUI,
    /// Media Library
    /// Media Library
    pub media_library: MediaLibrary,
    /// Bevy runner for 3D/Particles
    pub bevy_runner: Option<mapmap_bevy::BevyRunner>,
}
