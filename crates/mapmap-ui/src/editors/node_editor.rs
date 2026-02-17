//! Phase 6: Node-Based Effect Editor
//!
//! Visual node graph for creating complex effects by connecting nodes.
//! Supports effect nodes, math nodes, utility nodes, and custom node API.

use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
pub use mapmap_core::shader_graph::{DataType, GraphId, NodeType, ParameterValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node graph editor
pub struct NodeEditor {
    /// ID of the graph being edited
    pub graph_id: Option<GraphId>,
    /// All nodes in the graph (shadow copy for UI)
    pub nodes: HashMap<NodeId, Node>,
    /// All connections
    pub connections: Vec<Connection>,
    /// Next node ID
    next_id: u64,
    /// Selected nodes
    selected_nodes: Vec<NodeId>,
    /// Node being dragged
    dragging_node: Option<(NodeId, Vec2)>,
    /// Connection being created
    creating_connection: Option<(NodeId, String, Pos2)>, // String for socket name
    pan_offset: Vec2,
    zoom: f32,
    node_palette: Vec<NodeType>,
    show_palette: bool,
    palette_pos: Option<Pos2>,
}

/// Unique node identifier
pub type NodeId = u64;

/// Node in the graph (UI representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: Pos2,
    pub inputs: Vec<Socket>,
    pub outputs: Vec<Socket>,
    pub parameters: HashMap<String, ParameterValue>,
    pub size: Vec2,
}

/// Socket (input or output connection point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Socket {
    pub name: String,
    pub data_type: DataType,
    pub connected: bool,
}

impl Socket {
    pub fn new(name: &str, data_type: DataType) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            connected: false,
        }
    }
}

/// Connection between two node sockets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_socket: String, // Output name
    pub to_node: NodeId,
    pub to_socket: String, // Input name
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeEditor {
    pub fn new() -> Self {
        Self {
            graph_id: None,
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_id: 1,
            selected_nodes: Vec::new(),
            dragging_node: None,
            creating_connection: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            node_palette: Self::create_palette(),
            show_palette: false,
            palette_pos: None,
        }
    }

    /// Load a graph from core definition
    pub fn load_graph(&mut self, graph: &mapmap_core::shader_graph::ShaderGraph) {
        self.graph_id = Some(graph.id);
        self.nodes.clear();
        self.connections.clear();
        self.next_id = 1; // todo: sync with graph?

        // Map core nodes to UI nodes
        for (id, core_node) in &graph.nodes {
            let ui_node = self.core_node_to_ui(core_node);
            self.nodes.insert(*id, ui_node);
            self.next_id = self.next_id.max(id + 1);

            // Reconstruct connections
            for input in &core_node.inputs {
                if let Some((from_node, from_output)) = &input.connected_output {
                    self.connections.push(Connection {
                        from_node: *from_node,
                        from_socket: from_output.clone(),
                        to_node: *id,
                        to_socket: input.name.clone(),
                    });
                    // Mark socket as connected
                    if let Some(node) = self.nodes.get_mut(id) {
                        if let Some(socket) = node.inputs.iter_mut().find(|s| s.name == input.name)
                        {
                            socket.connected = true;
                        }
                    }
                }
            }
        }
    }

    /// Create UI node from core node
    fn core_node_to_ui(&self, core_node: &mapmap_core::shader_graph::ShaderNode) -> Node {
        let inputs: Vec<Socket> = core_node
            .inputs
            .iter()
            .map(|s| Socket::new(&s.name, s.data_type))
            .collect();
        let outputs: Vec<Socket> = core_node
            .outputs
            .iter()
            .map(|s| Socket::new(&s.name, s.data_type))
            .collect();

        // Calculate dynamic height
        let height = 80.0 + (inputs.len().max(outputs.len()) as f32 * 24.0);

        let size = Vec2::new(180.0, height);

        Node {
            id: core_node.id,
            node_type: core_node.node_type.clone(),
            position: Pos2::new(core_node.position.0, core_node.position.1),
            inputs,
            outputs,
            parameters: core_node.parameters.clone(),
            size,
        }
    }

    /// Create the node palette with all available node types
    fn create_palette() -> Vec<NodeType> {
        vec![
            // Input
            NodeType::TextureInput,
            NodeType::TimeInput,
            NodeType::UVInput,
            NodeType::ParameterInput,
            // Math
            NodeType::Add,
            NodeType::Subtract,
            NodeType::Multiply,
            NodeType::Divide,
            NodeType::Power,
            NodeType::Sin,
            NodeType::Cos,
            NodeType::Clamp,
            NodeType::Mix,
            // Color
            NodeType::ColorRamp,
            NodeType::HSVToRGB,
            NodeType::RGBToHSV,
            NodeType::Desaturate,
            NodeType::Brightness,
            NodeType::Contrast,
            // Texture
            NodeType::TextureSample,
            NodeType::TextureCombine,
            NodeType::UVTransform,
            NodeType::UVDistort,
            // Effects
            NodeType::Blur,
            NodeType::Glow,
            NodeType::ChromaticAberration,
            NodeType::Kaleidoscope,
            NodeType::PixelSort,
            NodeType::EdgeDetect,
            // Utility
            NodeType::Split,
            NodeType::Combine,
            // Output
            NodeType::Output,
        ]
    }

    /// Add a new node to the graph
    pub fn add_node(&mut self, node_type: NodeType, position: Pos2) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;

        // Use core logic to create default sockets and parameters
        let core_node = mapmap_core::shader_graph::ShaderNode::new(id, node_type);
        let mut ui_node = self.core_node_to_ui(&core_node);
        ui_node.position = position;

        self.nodes.insert(id, ui_node);
        id
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node_id: NodeId) {
        self.nodes.remove(&node_id);
        self.connections
            .retain(|c| c.from_node != node_id && c.to_node != node_id);
        self.selected_nodes.retain(|id| *id != node_id);
    }

    /// Add a connection between two sockets
    pub fn add_connection(
        &mut self,
        from_node: NodeId,
        from_socket_name: String,
        to_node: NodeId,
        to_socket_name: String,
    ) -> bool {
        // Validate connection
        if let (Some(from), Some(to)) = (self.nodes.get(&from_node), self.nodes.get(&to_node)) {
            // Find sockets
            let out_socket = from.outputs.iter().find(|s| s.name == from_socket_name);
            let in_socket = to.inputs.iter().find(|s| s.name == to_socket_name);

            if let (Some(out_s), Some(in_s)) = (out_socket, in_socket) {
                if out_s.data_type.compatible_with(&in_s.data_type) {
                    // Remove existing connection to this input
                    // Clone to_socket_name to avoid move issues in closure
                    let target_socket = to_socket_name.clone();
                    self.connections
                        .retain(|c| c.to_node != to_node || c.to_socket != target_socket);

                    self.connections.push(Connection {
                        from_node,
                        from_socket: from_socket_name,
                        to_node,
                        to_socket: to_socket_name.clone(), // Clone here too
                    });

                    // Update socket status
                    if let Some(node) = self.nodes.get_mut(&to_node) {
                        if let Some(socket) =
                            node.inputs.iter_mut().find(|s| s.name == to_socket_name)
                        {
                            socket.connected = true;
                        }
                    }

                    return true;
                }
            }
        }
        false
    }

    /// Handle actions triggered by the UI or external events
    pub fn handle_action(&mut self, action: NodeEditorAction) {
        match action {
            NodeEditorAction::AddNode(node_type, pos) => {
                self.add_node(node_type, pos);
            }
            NodeEditorAction::RemoveNode(node_id) => {
                self.remove_node(node_id);
            }
            NodeEditorAction::SelectNode(node_id) => {
                self.selected_nodes.clear();
                self.selected_nodes.push(node_id);
            }
            NodeEditorAction::AddConnection(from, from_socket, to, to_socket) => {
                self.add_connection(from, from_socket, to, to_socket);
            }
            NodeEditorAction::RemoveConnection(from, from_socket, to, to_socket) => {
                self.connections.retain(|c| {
                    !(c.from_node == from
                        && c.from_socket == from_socket
                        && c.to_node == to
                        && c.to_socket == to_socket)
                });
            }
            NodeEditorAction::UpdateGraph(_) => {
                // Handled by main app logic usually, but here for completeness
            }
        }
    }

    // UI Helpers (Static/Associated functions to avoid &self borrows during iteration)

    fn get_socket_pos(node: &Node, socket_idx: usize, is_input: bool) -> Pos2 {
        let socket_y = node.position.y + 40.0 + (socket_idx as f32 * 24.0);
        let socket_x = if is_input {
            node.position.x
        } else {
            node.position.x + node.size.x
        };
        Pos2::new(socket_x, socket_y)
    }

    fn draw_socket(
        painter: &egui::Painter,
        pos: Pos2,
        data_type: DataType,
        _is_input: bool,
        zoom: f32,
    ) {
        let radius = 6.0 * zoom.clamp(0.1, 10.0);
        painter.circle_filled(pos, radius, data_type.color());
        painter.circle_stroke(pos, radius, Stroke::new(2.0, Color32::WHITE));
    }

    /// Draw a node
    fn draw_node(
        ui: &Ui,
        painter: &egui::Painter,
        node: &mut Node, // Mutable to potentially support parameter editing later
        rect: Rect,
        locale: &LocaleManager,
        zoom: f32,
        is_selected: bool,
    ) -> Response {
        let response = ui.interact(rect, egui::Id::new(node.id), Sense::click_and_drag());

        let bg_color = if is_selected {
            Color32::from_rgb(50, 100, 150)
        } else {
            Color32::from_rgb(40, 40, 40)
        };

        // Node background
        painter.rect_filled(rect, 4.0, bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(2.0, Color32::from_rgb(80, 80, 80)),
            egui::StrokeKind::Middle,
        );

        // Title bar
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 24.0 * zoom));
        painter.rect_filled(title_rect, 4.0, Color32::from_rgb(30, 30, 30));
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            node.node_type.ui_name(locale),
            egui::FontId::proportional(14.0 * zoom.clamp(0.1, 10.0)),
            Color32::WHITE,
        );

        // Content Area
        let content_rect = Rect::from_min_max(rect.min + Vec2::new(0.0, 24.0 * zoom), rect.max);
        Self::draw_node_content(ui, painter, node, content_rect, zoom);

        // Input sockets
        for (i, input) in node.inputs.iter().enumerate() {
            let socket_pos = Self::get_socket_pos(node, i, true);
            Self::draw_socket(painter, socket_pos, input.data_type, true, zoom);

            // Draw label
            let text_pos = socket_pos + Vec2::new(10.0 * zoom, 0.0);
            painter.text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                &input.name,
                egui::FontId::proportional(12.0 * zoom.clamp(0.1, 10.0)),
                Color32::LIGHT_GRAY,
            );
        }

        // Output sockets
        for (i, output) in node.outputs.iter().enumerate() {
            let socket_pos = Self::get_socket_pos(node, i, false);
            Self::draw_socket(painter, socket_pos, output.data_type, false, zoom);

            // Draw label
            let text_pos = socket_pos - Vec2::new(10.0 * zoom, 0.0);
            painter.text(
                text_pos,
                egui::Align2::RIGHT_CENTER,
                &output.name,
                egui::FontId::proportional(12.0 * zoom.clamp(0.1, 10.0)),
                Color32::LIGHT_GRAY,
            );
        }

        response
    }

    fn draw_node_content(ui: &Ui, painter: &egui::Painter, node: &mut Node, rect: Rect, zoom: f32) {
        if node.node_type == NodeType::TextureInput {
            Self::draw_media_node_content(ui, painter, node, rect, zoom);
        }
    }

    fn draw_media_node_content(
        _ui: &Ui,
        painter: &egui::Painter,
        node: &mut Node,
        rect: Rect,
        zoom: f32,
    ) {
        // Draw a simple media placeholder/controls
        let center = rect.center();

        // Play button placeholder (Triangle)
        let size = 20.0 * zoom;
        let p1 = center + Vec2::new(-size * 0.5, -size * 0.5);
        let p2 = center + Vec2::new(-size * 0.5, size * 0.5);
        let p3 = center + Vec2::new(size * 0.5, 0.0);

        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            Color32::from_rgb(100, 200, 100),
            Stroke::new(1.0, Color32::WHITE),
        ));

        // Filename label if parameter exists
        if let Some(ParameterValue::String(path)) = node.parameters.get("path") {
            let text_pos = center + Vec2::new(0.0, size);
            let file_name = std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or(std::borrow::Cow::Borrowed("No File"));

            painter.text(
                text_pos,
                egui::Align2::CENTER_TOP,
                file_name,
                egui::FontId::proportional(10.0 * zoom),
                Color32::LIGHT_GRAY,
            );
        }
    }

    fn draw_connection(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
        let control_offset = ((to.x - from.x) * 0.5).abs().max(50.0);
        let ctrl1 = Pos2::new(from.x + control_offset, from.y);
        let ctrl2 = Pos2::new(to.x - control_offset, to.y);

        // Draw bezier curve with multiple segments
        let segments = 20;
        let mut points = Vec::new();
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let point = cubic_bezier(from, ctrl1, ctrl2, to, t);
            points.push(point);
        }

        for i in 0..points.len() - 1 {
            painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, color));
        }
    }

    /// Render the node editor UI
    pub fn ui(&mut self, ui: &mut Ui, locale: &LocaleManager) -> Option<NodeEditorAction> {
        let mut action = None;

        // Canvas background
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // Handle canvas interactions
        if response.dragged() && self.dragging_node.is_none() && self.creating_connection.is_none()
        {
            self.pan_offset += response.drag_delta();
        }

        // Zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom *= 1.0 + scroll * 0.001;
                self.zoom = self.zoom.clamp(0.2, 3.0);
            }
        }

        // Right-click to show palette
        if response.secondary_clicked() {
            self.show_palette = true;
            self.palette_pos = response.interact_pointer_pos();
        }

        let canvas_rect = response.rect;
        let zoom = self.zoom;
        let pan_offset = self.pan_offset;

        let to_screen =
            |pos: Pos2| -> Pos2 { canvas_rect.min + (pos.to_vec2() + pan_offset) * zoom };

        // Draw grid
        self.draw_grid(&painter, canvas_rect);

        // Draw connections
        for conn in &self.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.get(&conn.from_node),
                self.nodes.get(&conn.to_node),
            ) {
                let from_idx = from_node
                    .outputs
                    .iter()
                    .position(|s| s.name == conn.from_socket);
                let to_idx = to_node.inputs.iter().position(|s| s.name == conn.to_socket);

                if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
                    let from_pos = Self::get_socket_pos(from_node, f_idx, false);
                    let to_pos = Self::get_socket_pos(to_node, t_idx, true);

                    let from_screen = to_screen(from_pos);
                    let to_screen = to_screen(to_pos);

                    let color = from_node.outputs[f_idx].data_type.color();
                    Self::draw_connection(&painter, from_screen, to_screen, color);
                }
            }
        }

        // Draw nodes
        // Using values_mut() safely
        let mut nodes_vec: Vec<_> = self.nodes.values_mut().collect();
        nodes_vec.sort_by_key(|n| n.id);

        for node in nodes_vec {
            let node_screen_pos = to_screen(node.position);
            let node_screen_rect = Rect::from_min_size(node_screen_pos, node.size * zoom);

            let is_selected = self.selected_nodes.contains(&node.id);

            let node_response = Self::draw_node(
                ui,
                &painter,
                node,
                node_screen_rect,
                locale,
                zoom,
                is_selected,
            );

            if node_response.clicked() {
                self.selected_nodes.clear();
                self.selected_nodes.push(node.id);
                action = Some(NodeEditorAction::SelectNode(node.id));
            }

            if node_response.dragged() {
                self.dragging_node = Some((node.id, response.drag_delta() / zoom));
            }
        }

        // Apply node dragging
        if let Some((node_id, delta)) = self.dragging_node {
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.position += delta;
            }
            if !response.dragged() {
                self.dragging_node = None;
            }
        }

        // Draw connection being created
        if let Some((_node_id, _socket_name, start_pos)) = &self.creating_connection {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                Self::draw_connection(
                    &painter,
                    *start_pos,
                    pointer_pos,
                    Color32::from_rgb(150, 150, 150),
                );

                if response.clicked() {
                    self.creating_connection = None;
                }
            }
        }

        // Node palette popup
        if self.show_palette {
            if let Some(pos) = self.palette_pos {
                egui::Area::new(egui::Id::new("node_palette"))
                    .fixed_pos(pos)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::default().show(ui, |ui| {
                            ui.set_min_width(200.0);
                            ui.label(locale.t("node-add"));
                            ui.separator();

                            let mut selected_type: Option<NodeType> = None;
                            let mut current_category = String::new();

                            for node_type in &self.node_palette {
                                let category = node_type.ui_category(locale);
                                if category != current_category {
                                    current_category = category.clone();
                                    ui.separator();
                                    ui.label(&current_category);
                                }

                                if ui.button(node_type.ui_name(locale)).clicked() {
                                    selected_type = Some(node_type.clone());
                                    self.show_palette = false;
                                }
                            }

                            if let Some(node_type) = selected_type {
                                let world_pos = (pos - canvas_rect.min - pan_offset) / zoom;
                                action =
                                    Some(NodeEditorAction::AddNode(node_type, world_pos.to_pos2()));
                            }
                        });
                    });
            }

            if response.clicked() {
                self.show_palette = false;
            }
        }

        action
    }

    /// Draw grid background (Uses &self, but is called before mutable iteration)
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let color = Color32::from_rgb(40, 40, 40);

        let start_x = (rect.min.x / grid_size).floor() * grid_size;
        let start_y = (rect.min.y / grid_size).floor() * grid_size;

        let mut x = start_x;
        while x < rect.max.x {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, color),
            );
            x += grid_size;
        }

        let mut y = start_y;
        while y < rect.max.y {
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, color),
            );
            y += grid_size;
        }
    }
}

/// Helper functions
fn cubic_bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Pos2::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Actions that can be triggered by the node editor
#[derive(Debug, Clone)]
pub enum NodeEditorAction {
    AddNode(NodeType, Pos2),
    RemoveNode(NodeId),
    SelectNode(NodeId),
    AddConnection(NodeId, String, NodeId, String),
    RemoveConnection(NodeId, String, NodeId, String),
    /// Request full graph save/update
    UpdateGraph(GraphId),
}

/// Helper trait for UI names
pub trait NodeTypeUI {
    fn ui_name(&self, locale: &LocaleManager) -> String;
    fn ui_category(&self, locale: &LocaleManager) -> String;
}

impl NodeTypeUI for NodeType {
    fn ui_name(&self, _locale: &LocaleManager) -> String {
        // Fallback names for simplicity now, ideally fully localized
        match self {
            NodeType::TextureInput => "Texture Input",
            NodeType::TimeInput => "Time",
            NodeType::UVInput => "UV",
            NodeType::ParameterInput => "Param",
            NodeType::Output => "Output",
            _ => self.display_name(), // from core, returns static str
        }
        .to_string()
    }

    fn ui_category(&self, _locale: &LocaleManager) -> String {
        self.category().to_string() // from core
    }
}

/// Extension trait for DataType to get UI colors
pub trait DataTypeUI {
    fn color(&self) -> Color32;
    fn compatible_with(&self, other: &DataType) -> bool;
}

impl DataTypeUI for DataType {
    fn color(&self) -> Color32 {
        match self {
            DataType::Float => Color32::from_rgb(100, 150, 255),
            DataType::Vec2 => Color32::from_rgb(150, 100, 255),
            DataType::Vec3 => Color32::from_rgb(200, 100, 200),
            DataType::Vec4 => Color32::from_rgb(255, 100, 255),
            DataType::Color => Color32::from_rgb(255, 150, 100),
            DataType::Texture => Color32::from_rgb(255, 200, 100),
            DataType::Sampler => Color32::from_rgb(150, 150, 150),
        }
    }

    fn compatible_with(&self, other: &DataType) -> bool {
        // Simple type checking for now
        self == other
    }
}




