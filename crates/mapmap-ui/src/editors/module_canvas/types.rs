use crate::theme::colors;
use egui::Color32;
use egui_node_editor::*;
use mapmap_core::module::{ModulePartId, ModulePartType};
use std::borrow::Cow;

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
    fn data_type_color(&self, _user_state: &mut MyUserState) -> Color32 {
        match self {
            MyDataType::Trigger => Color32::from_rgb(180, 100, 220),
            MyDataType::Media => Color32::from_rgb(100, 180, 220),
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
