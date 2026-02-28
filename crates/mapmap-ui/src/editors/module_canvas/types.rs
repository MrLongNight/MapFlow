use crate::theme::colors;
use egui::Pos2;
use egui_node_editor::*;
use mapmap_core::module::{ModulePartId, ModulePartType, ModuleSocketType, TriggerType};
use std::borrow::Cow;

/// Information about a socket position for hit detection
#[derive(Clone)]
pub struct SocketInfo {
    pub part_id: ModulePartId,
    pub socket_idx: usize,
    pub is_output: bool,
    pub socket_type: ModuleSocketType,
    /// 3D position coordinates [x, y, z].
    pub position: Pos2,
}

pub type PresetPart = (
    mapmap_core::module::ModulePartType,
    (f32, f32),
    Option<(f32, f32)>,
);
pub type PresetConnection = (usize, usize, usize, usize); // from_idx, from_socket, to_idx, to_socket

/// A saved module preset/template
#[derive(Debug, Clone)]
pub struct ModulePreset {
    /// Human-readable display name.
    pub name: String,
    pub parts: Vec<PresetPart>,
    pub connections: Vec<PresetConnection>,
}

/// Actions that can be undone/redone
#[derive(Debug, Clone)]
pub enum CanvasAction {
    AddPart {
        part_id: ModulePartId,
        part_data: mapmap_core::module::ModulePart,
    },
    DeletePart {
        part_data: mapmap_core::module::ModulePart,
    },
    MovePart {
        part_id: ModulePartId,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    AddConnection {
        connection: mapmap_core::module::ModuleConnection,
    },
    DeleteConnection {
        connection: mapmap_core::module::ModuleConnection,
    },
    Batch(Vec<CanvasAction>),
}

/// Playback commands for media players
#[derive(Debug, Clone, PartialEq)]
pub enum MediaPlaybackCommand {
    Play,
    Pause,
    Stop,
    /// Reload the media from disk (used when path changes)
    Reload,
    /// Set playback speed (1.0 = normal)
    SetSpeed(f32),
    /// Set loop mode
    SetLoop(bool),
    /// Seek to position (seconds from start)
    Seek(f64),
    /// Set reverse playback
    SetReverse(bool),
}

/// Information about a media player's current state
#[derive(Debug, Clone, Default)]
pub struct MediaPlayerInfo {
    /// Current playback position in seconds
    pub current_time: f64,
    /// Total duration in seconds
    pub duration: f64,
    /// Whether the player is currently playing
    pub is_playing: bool,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum MyDataType {
    Trigger,
    Media,
    Effect,
    Layer,
    Output,
    Link,
}

#[derive(Clone, Debug)]
pub struct MyNodeData {
    pub title: String,
    pub part_type: ModulePartType,
    pub original_part_id: ModulePartId,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub struct MyValueType;

#[derive(Clone, Debug)]
pub struct MyNodeTemplate {
/// User-friendly name for identifying the element.
    pub label: String,
    pub part_type_variant: String,
}

#[derive(Clone, Debug, Default)]
pub struct MyUserState {
    pub trigger_values: std::collections::HashMap<ModulePartId, f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MyResponse {
    Connect(NodeId, usize, NodeId, usize),
    Delete(NodeId),
}

impl UserResponseTrait for MyResponse {}

impl DataTypeTrait<MyUserState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyUserState) -> egui::Color32 {
        match self {
            MyDataType::Trigger => egui::Color32::from_rgb(180, 100, 220),
            MyDataType::Media => egui::Color32::from_rgb(100, 180, 220),
            MyDataType::Effect => colors::WARN_COLOR,
            MyDataType::Layer => colors::MINT_ACCENT,
            MyDataType::Output => colors::ERROR_COLOR,
            MyDataType::Link => colors::STROKE_GREY,
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            MyDataType::Trigger => Cow::Borrowed("Trigger"),
            MyDataType::Media => Cow::Borrowed("Media"),
            MyDataType::Effect => Cow::Borrowed("Effect"),
            MyDataType::Layer => Cow::Borrowed("Layer"),
            MyDataType::Output => Cow::Borrowed("Output"),
            MyDataType::Link => Cow::Borrowed("Link"),
        }
    }
}

impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyUserState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    fn can_delete(
        &self,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut MyUserState,
    ) -> bool {
        true
    }

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut MyUserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        ui.label(format!("Type: {:?}", self.part_type));
        Vec::new()
    }
}

// Implement NodeTemplateTrait for MyNodeTemplate
impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyUserState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        Cow::Borrowed(&self.label)
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        self.label.clone()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData {
            title: self.label.clone(),
            part_type: mapmap_core::module::ModulePartType::Trigger(TriggerType::Beat), // Mock
            original_part_id: 0,
        }
    }

    fn build_node(
        &self,
        _graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        _node_id: NodeId,
    ) {
        // Mock
    }
}
