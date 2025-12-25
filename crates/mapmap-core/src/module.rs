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
    pub inputs: Vec<ModuleSocket>,
    pub outputs: Vec<ModuleSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSocket {
    pub name: String,
    pub socket_type: ModuleSocketType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Resource(ResourceType),
    Modulizer(ModulizerType),
    LayerAssignment(LayerAssignmentType),
    Output(OutputType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    Midi { channel: u8, note: u8 },
    Osc { address: String },
    Keyboard { key_code: String },
    Beat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    MediaFile { path: String },
    Shader { path: String },
    LiveInput { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModulizerType {
    Effect { name: String },
    BlendMode { mode: String },
    AudioReactive { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerAssignmentType {
    SingleLayer { id: u64 },
    Group { name: String },
    AllLayers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    Projector { id: u64, preview_disabled: bool },
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
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}
