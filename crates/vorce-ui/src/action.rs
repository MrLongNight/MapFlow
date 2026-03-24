use crate::editors::node_editor::NodeEditorAction;
use crate::editors::timeline_v2::TimelineAction;
#[cfg(feature = "ndi")]
use vorce_io::NdiSource;

/// UI actions that can be triggered by the user interface
#[derive(Debug, Clone)]
pub enum UIAction {
    // Playback actions
    /// Start playback
    Play,
    /// Pause playback
    Pause,
    /// Stop playback
    Stop,
    /// Set global playback speed
    SetSpeed(f32),
    /// Set global loop mode
    SetLoopMode(vorce_media::LoopMode),

    // File actions
    /// Create a new project
    NewProject,
    /// Load a video file
    LoadVideo(String),
    /// Open file picker for media source
    PickMediaFile(
        vorce_core::module::ModuleId,
        vorce_core::module::ModulePartId,
        String,
    ),
    /// Set media file for source
    SetMediaFile(
        vorce_core::module::ModuleId,
        vorce_core::module::ModulePartId,
        String,
    ),

    /// Save current project
    SaveProject(String),
    /// Save project as new file
    SaveProjectAs,
    /// Load project from file
    LoadProject(String),
    /// Load project from recent list
    LoadRecentProject(String),
    /// Export project
    Export,
    /// Open settings dialog
    OpenSettings,
    /// Exit application
    Exit,

    // Edit actions
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,
    /// Cut selection
    Cut,
    /// Copy selection
    Copy,
    /// Paste from clipboard
    Paste,
    /// Delete selection
    Delete,
    /// Select all items
    SelectAll,

    // Mapping actions
    /// Add new mapping
    AddMapping,
    /// Remove mapping by ID
    RemoveMapping(u64),
    /// Toggle mapping visibility
    ToggleMappingVisibility(u64, bool),
    /// Select mapping by ID
    SelectMapping(u64),
    /// Update mapping mesh
    UpdateMappingMesh(u64, vorce_core::Mesh),
    /// Set MIDI assignment for UI element
    SetMidiAssignment(String, String), // element_id, target_id

    // Paint actions
    /// Add new paint source
    AddPaint,
    /// Remove paint source by ID
    RemovePaint(u64),

    // Layer actions (Phase 1)
    /// Add new layer
    AddLayer,
    /// Create layer group
    CreateGroup,
    /// Remove layer by ID
    RemoveLayer(u64),
    /// Duplicate layer by ID
    DuplicateLayer(u64),
    /// Reparent layer
    ReparentLayer(u64, Option<u64>),
    /// Swap layer order
    SwapLayers(u64, u64),
    /// Toggle group collapse state
    ToggleGroupCollapsed(u64),
    /// Rename layer
    RenameLayer(u64, String),
    /// Toggle layer bypass
    ToggleLayerBypass(u64),
    /// Toggle layer solo
    ToggleLayerSolo(u64),
    /// Set layer opacity
    SetLayerOpacity(u64, f32),
    /// Set layer blend mode
    SetLayerBlendMode(u64, vorce_core::BlendMode),
    /// Set layer visibility
    SetLayerVisibility(u64, bool),
    /// Remove all layers
    EjectAllLayers,

    // Transform actions (Phase 1)
    /// Set layer transform
    SetLayerTransform(u64, vorce_core::Transform),
    /// Apply resize mode to layer
    ApplyResizeMode(u64, vorce_core::ResizeMode),

    // Master controls (Phase 1)
    /// Set master opacity
    SetMasterOpacity(f32),
    /// Set master playback speed
    SetMasterSpeed(f32),
    /// Set composition name
    SetCompositionName(String),

    // Phase 2: Output management
    /// Add new output
    AddOutput(String, vorce_core::CanvasRegion, (u32, u32)),
    /// Remove output by ID
    RemoveOutput(u64),
    /// Configure output settings
    ConfigureOutput(u64, vorce_core::OutputConfig),
    /// Set output edge blend configuration
    SetOutputEdgeBlend(u64, vorce_core::EdgeBlendConfig),
    /// Set output color calibration
    SetOutputColorCalibration(u64, vorce_core::ColorCalibration),
    /// Create 2x2 projector array
    CreateProjectorArray2x2((u32, u32), f32),

    // View actions
    /// Toggle fullscreen mode
    ToggleFullscreen,
    /// Reset UI layout to default
    ResetLayout,
    /// Toggle module canvas visibility
    ToggleModuleCanvas,
    /// Toggle controller overlay visibility
    ToggleControllerOverlay,
    /// Toggle media manager visibility
    ToggleMediaManager,

    // Audio actions
    /// Select audio input device
    SelectAudioDevice(String),
    /// Update audio configuration
    UpdateAudioConfig(vorce_core::audio::AudioConfig),
    /// Toggle audio panel visibility
    ToggleAudioPanel,

    // Settings
    /// Set UI language
    SetLanguage(String),
    /// Set audio meter style
    SetMeterStyle(crate::config::AudioMeterStyle),
    /// Set target FPS
    SetTargetFps(f32),
    /// Set VSync mode
    SetVsyncMode(crate::core::config::VSyncMode),
    /// Set preferred GPU
    SetPreferredGpu(Option<String>),
    /// Set master blackout
    SetMasterBlackout(bool),
    /// Connect to Philips Hue bridge
    ConnectHue,
    /// Disconnect from Philips Hue bridge
    DisconnectHue,
    /// Start Hue bridge discovery
    DiscoverHueBridges,
    /// Fetch Hue entertainment groups
    FetchHueGroups,
    /// Register app with Hue bridge
    RegisterHue,

    // Help actions
    /// Open documentation
    OpenDocs,
    /// Open "About" dialog
    OpenAbout,
    /// Open license information
    OpenLicense,

    // Module actions
    #[cfg(feature = "ndi")]
    /// Connect NDI source to module part
    ConnectNdiSource {
        /// The module part ID
        part_id: vorce_core::module::ModulePartId,
        /// The NDI source
        source: NdiSource,
    },
    #[cfg(feature = "ndi")]
    /// Disconnect NDI source from module part
    DisconnectNdiSource {
        /// The module part ID
        part_id: vorce_core::module::ModulePartId,
    },

    // Cue actions (Phase 7)
    /// Add new cue
    AddCue,
    /// Remove cue by index
    RemoveCue(u32),
    /// Update cue data
    UpdateCue(Box<vorce_control::cue::Cue>),
    /// Trigger cue execution
    GoCue(u32),
    /// Go to next cue
    NextCue,
    /// Go to previous cue
    PrevCue,
    /// Stop current cue
    StopCue,

    // Shader Graph (Phase 6b)
    /// Open shader graph editor
    OpenShaderGraph(vorce_core::GraphId),

    // MIDI
    /// Toggle MIDI learn mode
    ToggleMidiLearn,

    // Node Action (Phase 6)
    /// Execute node editor action
    NodeAction(NodeEditorAction),

    /// Execute timeline action
    TimelineAction(TimelineAction),

    // Global Fullscreen Setting
    /// Set global fullscreen state
    SetGlobalFullscreen(bool),

    // Media commands for specific module parts
    /// Send playback command to media module
    MediaCommand(
        vorce_core::module::ModulePartId,
        crate::editors::module_canvas::types::MediaPlaybackCommand,
    ),

    /// Manually fire a trigger node
    ManualTrigger(
        vorce_core::module::ModuleId,
        vorce_core::module::ModulePartId,
    ),

    // Module Connection Deletion
    /// Delete a connection between two module parts
    DeleteConnection(
        vorce_core::module::ModuleId,
        vorce_core::module::ModuleConnection,
    ),
}
