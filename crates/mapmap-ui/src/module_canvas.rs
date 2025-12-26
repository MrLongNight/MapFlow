use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{
    AudioBand, BlendModeType, EffectType as ModuleEffectType, LayerAssignmentType, MapFlowModule,
    MaskShape, MaskType, MeshType, ModuleManager, ModulePart, ModulePartId, ModuleSocketType,
    ModulizerType, OutputType, SourceType, TriggerType,
};

/// Information about a socket position for hit detection
#[derive(Clone)]
struct SocketInfo {
    part_id: ModulePartId,
    socket_idx: usize,
    is_output: bool,
    socket_type: ModuleSocketType,
    position: Pos2,
}

#[allow(dead_code)]
pub struct ModuleCanvas {
    /// The ID of the currently active/edited module
    active_module_id: Option<u64>,
    /// Canvas pan offset
    pan_offset: Vec2,
    /// Canvas zoom level
    zoom: f32,
    /// Part being dragged
    dragging_part: Option<(ModulePartId, Vec2)>,
    /// Part being resized: (part_id, original_size)
    resizing_part: Option<(ModulePartId, (f32, f32))>,
    /// Box selection start position (screen coords)
    box_select_start: Option<Pos2>,
    /// Connection being created: (from_part, from_socket_idx, is_output, socket_type, start_pos)
    creating_connection: Option<(ModulePartId, usize, bool, ModuleSocketType, Pos2)>,
    /// Part ID pending deletion (set when X button clicked)
    pending_delete: Option<ModulePartId>,
    /// Selected parts for multi-select and copy/paste
    selected_parts: Vec<ModulePartId>,
    /// Clipboard for copy/paste (stores part types and relative positions)
    clipboard: Vec<(mapmap_core::module::ModulePartType, (f32, f32))>,
    /// Search filter text
    search_filter: String,
    /// Whether search popup is visible
    show_search: bool,
    /// Undo history stack
    undo_stack: Vec<CanvasAction>,
    /// Redo history stack
    redo_stack: Vec<CanvasAction>,
    /// Saved module presets
    presets: Vec<ModulePreset>,
    /// Whether preset panel is visible
    show_presets: bool,
    /// New preset name input
    new_preset_name: String,
    /// Context menu position
    context_menu_pos: Option<Pos2>,
    /// Context menu target (connection index or None)
    context_menu_connection: Option<usize>,
    /// Context menu target (part ID or None)
    context_menu_part: Option<ModulePartId>,
    /// MIDI Learn mode - which part is waiting for MIDI input
    midi_learn_part_id: Option<ModulePartId>,
    /// Whether we are currently panning the canvas (started on empty area)
    panning_canvas: bool,
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
}

impl Default for ModuleCanvas {
    fn default() -> Self {
        Self {
            active_module_id: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_part: None,
            resizing_part: None,
            box_select_start: None,
            creating_connection: None,
            pending_delete: None,
            selected_parts: Vec::new(),
            clipboard: Vec::new(),
            search_filter: String::new(),
            show_search: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            presets: Self::default_presets(),
            show_presets: false,
            new_preset_name: String::new(),
            context_menu_pos: None,
            context_menu_connection: None,
            context_menu_part: None,
            midi_learn_part_id: None,
            panning_canvas: false,
        }
    }
}

impl ModuleCanvas {
    /// Set the active module ID
    pub fn set_active_module(&mut self, module_id: Option<u64>) {
        self.active_module_id = module_id;
    }

    /// Get the active module ID
    pub fn active_module_id(&self) -> Option<u64> {
        self.active_module_id
    }

    /// Add a Trigger node with specified type
    fn add_trigger_node(&mut self, manager: &mut ModuleManager, trigger_type: TriggerType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (100.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Trigger(trigger_type),
                    pos,
                );
            }
        }
    }

    /// Add a Source node with specified type
    fn add_source_node(&mut self, manager: &mut ModuleManager, source_type: SourceType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (200.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Source(source_type),
                    pos,
                );
            }
        }
    }

    /// Add a Mask node with specified type
    fn add_mask_node(&mut self, manager: &mut ModuleManager, mask_type: MaskType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (300.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Mask(mask_type),
                    pos,
                );
            }
        }
    }

    /// Add a Modulator node with specified type
    fn add_modulator_node(&mut self, manager: &mut ModuleManager, mod_type: ModulizerType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (400.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Modulizer(mod_type),
                    pos,
                );
            }
        }
    }

    /// Add a Layer node with specified type
    fn add_layer_node(&mut self, manager: &mut ModuleManager, layer_type: LayerAssignmentType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (500.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::LayerAssignment(layer_type),
                    pos,
                );
            }
        }
    }

    /// Add a Mesh node with specified type
    fn add_mesh_node(&mut self, manager: &mut ModuleManager, mesh_type: MeshType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (450.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Mesh(mesh_type),
                    pos,
                );
            }
        }
    }

    /// Add an Output node with specified type
    fn add_output_node(&mut self, manager: &mut ModuleManager, output_type: OutputType) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let pos = Self::find_free_position(&module.parts, (600.0, 100.0));
                module.add_part_with_type(
                    mapmap_core::module::ModulePartType::Output(output_type),
                    pos,
                );
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui, manager: &mut ModuleManager, locale: &LocaleManager) {
        // === CANVAS TOOLBAR ===
        ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Create New Module
            if ui
                .button("➕ New Module")
                .on_hover_text("Create a new module")
                .clicked()
            {
                let new_module_id = manager.create_module("New Module".to_string());
                self.active_module_id = Some(new_module_id);
            }

            ui.separator();

            // Part creation tools (only enabled when module is active)
            let has_module = self.active_module_id.is_some();

            ui.add_enabled_ui(has_module, |ui| {
                // === SIGNAL FLOW ORDER: Trigger → Source → Mask → Modulator → Layer → Output ===
                
                // TRIGGER DROPDOWN
                egui::menu::menu_button(ui, "⚡ Trigger", |ui| {
                    ui.set_min_width(180.0);
                    ui.label("Audio Analysis");
                    if ui.button("🎵 Audio FFT").clicked() {
                        self.add_trigger_node(manager, TriggerType::AudioFFT { band: AudioBand::Bass, threshold: 0.5 });
                        ui.close_menu();
                    }
                    if ui.button("🥁 Beat Detection").clicked() {
                        self.add_trigger_node(manager, TriggerType::Beat);
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("Control");
                    if ui.button("🎹 MIDI").clicked() {
                        self.add_trigger_node(manager, TriggerType::Midi { channel: 1, note: 60 });
                        ui.close_menu();
                    }
                    if ui.button("📡 OSC").clicked() {
                        self.add_trigger_node(manager, TriggerType::Osc { address: "/trigger".to_string() });
                        ui.close_menu();
                    }
                    if ui.button("⌨️ Keyboard Shortcut").clicked() {
                        self.add_trigger_node(manager, TriggerType::Shortcut { key_code: "Space".to_string(), modifiers: 0 });
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("Time-based");
                    if ui.button("🎲 Random").clicked() {
                        self.add_trigger_node(manager, TriggerType::Random { min_interval_ms: 500, max_interval_ms: 2000, probability: 0.8 });
                        ui.close_menu();
                    }
                    if ui.button("⏱️ Fixed Timer").clicked() {
                        self.add_trigger_node(manager, TriggerType::Fixed { interval_ms: 1000, offset_ms: 0 });
                        ui.close_menu();
                    }
                });

                // SOURCE DROPDOWN
                egui::menu::menu_button(ui, "🎬 Source", |ui| {
                    ui.set_min_width(180.0);
                    if ui.button("📁 Media File").clicked() {
                        self.add_source_node(manager, SourceType::MediaFile { path: String::new() });
                        ui.close_menu();
                    }
                    if ui.button("🎨 Shader").clicked() {
                        self.add_source_node(manager, SourceType::Shader { name: "Default".to_string(), params: vec![] });
                        ui.close_menu();
                    }
                    if ui.button("📹 Live Input").clicked() {
                        self.add_source_node(manager, SourceType::LiveInput { device_id: 0 });
                        ui.close_menu();
                    }
                });

                // MASK DROPDOWN
                egui::menu::menu_button(ui, "🎭 Mask", |ui| {
                    ui.set_min_width(180.0);
                    ui.label("Shapes");
                    if ui.button("⬜ Rectangle").clicked() {
                        self.add_mask_node(manager, MaskType::Shape(MaskShape::Rectangle));
                        ui.close_menu();
                    }
                    if ui.button("⭕ Circle").clicked() {
                        self.add_mask_node(manager, MaskType::Shape(MaskShape::Circle));
                        ui.close_menu();
                    }
                    if ui.button("🔺 Triangle").clicked() {
                        self.add_mask_node(manager, MaskType::Shape(MaskShape::Triangle));
                        ui.close_menu();
                    }
                    if ui.button("⭐ Star").clicked() {
                        self.add_mask_node(manager, MaskType::Shape(MaskShape::Star));
                        ui.close_menu();
                    }
                    if ui.button("⬭ Ellipse").clicked() {
                        self.add_mask_node(manager, MaskType::Shape(MaskShape::Ellipse));
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📁 File Mask").clicked() {
                        self.add_mask_node(manager, MaskType::File { path: String::new() });
                        ui.close_menu();
                    }
                    if ui.button("🌈 Gradient").clicked() {
                        self.add_mask_node(manager, MaskType::Gradient { angle: 0.0, softness: 0.5 });
                        ui.close_menu();
                    }
                });

                // MODULATOR DROPDOWN
                egui::menu::menu_button(ui, "〰️ Modulator", |ui| {
                    ui.set_min_width(200.0);
                    ui.label("--- Basic ---");
                    if ui.button("Blur").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Blur)); ui.close_menu(); }
                    if ui.button("Sharpen").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Sharpen)); ui.close_menu(); }
                    if ui.button("Invert").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Invert)); ui.close_menu(); }
                    ui.separator();
                    ui.label("--- Color ---");
                    if ui.button("Brightness").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Brightness)); ui.close_menu(); }
                    if ui.button("Contrast").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Contrast)); ui.close_menu(); }
                    if ui.button("Saturation").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Saturation)); ui.close_menu(); }
                    if ui.button("Hue Shift").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::HueShift)); ui.close_menu(); }
                    ui.separator();
                    ui.label("--- Distort ---");
                    if ui.button("Kaleidoscope").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Kaleidoscope)); ui.close_menu(); }
                    if ui.button("Mirror").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Mirror)); ui.close_menu(); }
                    if ui.button("Wave").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Wave)); ui.close_menu(); }
                    ui.separator();
                    ui.label("--- Stylize ---");
                    if ui.button("Glitch").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Glitch)); ui.close_menu(); }
                    if ui.button("VHS").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::VHS)); ui.close_menu(); }
                    if ui.button("Pixelate").clicked() { self.add_modulator_node(manager, ModulizerType::Effect(ModuleEffectType::Pixelate)); ui.close_menu(); }
                    ui.separator();
                    ui.label("--- Blend Modes ---");
                    if ui.button("Add").clicked() { self.add_modulator_node(manager, ModulizerType::BlendMode(BlendModeType::Add)); ui.close_menu(); }
                    if ui.button("Multiply").clicked() { self.add_modulator_node(manager, ModulizerType::BlendMode(BlendModeType::Multiply)); ui.close_menu(); }
                    if ui.button("Screen").clicked() { self.add_modulator_node(manager, ModulizerType::BlendMode(BlendModeType::Screen)); ui.close_menu(); }
                    ui.separator();
                    ui.label("--- Audio Reactive ---");
                    if ui.button("🔊 Audio Reactive").clicked() { self.add_modulator_node(manager, ModulizerType::AudioReactive { source: "Bass".to_string() }); ui.close_menu(); }
                });

                // LAYER DROPDOWN
                egui::menu::menu_button(ui, "📑 Layer", |ui| {
                    ui.set_min_width(180.0);
                    if ui.button("🔲 Single Layer").clicked() {
                        self.add_layer_node(manager, LayerAssignmentType::SingleLayer { id: 0, name: "Layer 1".to_string(), opacity: 1.0, blend_mode: None });
                        ui.close_menu();
                    }
                    if ui.button("📂 Layer Group").clicked() {
                        self.add_layer_node(manager, LayerAssignmentType::Group { name: "Group 1".to_string(), opacity: 1.0, blend_mode: None });
                        ui.close_menu();
                    }
                    if ui.button("🎚️ All Layers (Master)").clicked() {
                        self.add_layer_node(manager, LayerAssignmentType::AllLayers { opacity: 1.0, blend_mode: None });
                        ui.close_menu();
                    }
                });

                // MESH DROPDOWN
                egui::menu::menu_button(ui, "🔷 Mesh", |ui| {
                    ui.set_min_width(200.0);
                    ui.label("--- Basic Shapes ---");
                    if ui.button("⬜ Quad").clicked() {
                        self.add_mesh_node(manager, MeshType::Quad { 
                            tl: (0.0, 0.0), tr: (1.0, 0.0), br: (1.0, 1.0), bl: (0.0, 1.0) 
                        });
                        ui.close_menu();
                    }
                    if ui.button("🔺 Triangle").clicked() {
                        self.add_mesh_node(manager, MeshType::TriMesh);
                        ui.close_menu();
                    }
                    if ui.button("⭕ Circle/Arc").clicked() {
                        self.add_mesh_node(manager, MeshType::Circle { segments: 32, arc_angle: 360.0 });
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("--- Subdivided ---");
                    if ui.button("▦ Grid (4x4)").clicked() {
                        self.add_mesh_node(manager, MeshType::Grid { rows: 4, cols: 4 });
                        ui.close_menu();
                    }
                    if ui.button("▦ Grid (8x8)").clicked() {
                        self.add_mesh_node(manager, MeshType::Grid { rows: 8, cols: 8 });
                        ui.close_menu();
                    }
                    if ui.button("〰️ Bezier Surface").clicked() {
                        self.add_mesh_node(manager, MeshType::BezierSurface { control_points: vec![] });
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("--- 3D Mapping ---");
                    if ui.button("🌐 Cylinder").clicked() {
                        self.add_mesh_node(manager, MeshType::Cylinder { segments: 16, height: 1.0 });
                        ui.close_menu();
                    }
                    if ui.button("🌍 Sphere (Dome)").clicked() {
                        self.add_mesh_node(manager, MeshType::Sphere { lat_segments: 8, lon_segments: 16 });
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📁 Custom Mesh...").clicked() {
                        self.add_mesh_node(manager, MeshType::Custom { path: String::new() });
                        ui.close_menu();
                    }
                });

                egui::menu::menu_button(ui, "📺 Output", |ui| {
                    ui.set_min_width(180.0);
                    if ui.button("📽️ Projector").clicked() {
                        self.add_output_node(manager, OutputType::Projector { id: 0, name: "Projector 1".to_string() });
                        ui.close_menu();
                    }
                    if ui.button("👁️ Preview Window").clicked() {
                        self.add_output_node(manager, OutputType::Preview { window_id: 0 });
                        ui.close_menu();
                    }
                });
            });

            ui.separator();

            // Module selector dropdown
            ui.label("Module:");
            let module_names: Vec<_> = manager
                .list_modules()
                .iter()
                .map(|m| (m.id, m.name.clone()))
                .collect();
            let current_name = self
                .active_module_id
                .and_then(|id| manager.get_module_mut(id))
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "None".to_string());

            egui::ComboBox::from_id_source("module_selector")
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.active_module_id, None, "None");
                    for (id, name) in module_names {
                        ui.selectable_value(&mut self.active_module_id, Some(id), name);
                    }
                });

            ui.separator();

            // === ZOOM CONTROLS ===
            ui.label("Zoom:");

            // Zoom out button
            if ui.button("−").on_hover_text("Zoom out").clicked() {
                self.zoom = (self.zoom - 0.1).clamp(0.2, 3.0);
            }

            // Zoom slider
            ui.add(
                egui::Slider::new(&mut self.zoom, 0.2..=3.0)
                    .show_value(false)
                    .clamp_to_range(true),
            );

            // Zoom in button
            if ui.button("+").on_hover_text("Zoom in").clicked() {
                self.zoom = (self.zoom + 0.1).clamp(0.2, 3.0);
            }

            // Zoom percentage display
            ui.label(format!("{:.0}%", self.zoom * 100.0));

            // Fit to view button
            if ui
                .button("⊡")
                .on_hover_text("Fit to view / Reset zoom")
                .clicked()
            {
                self.zoom = 1.0;
                self.pan_offset = Vec2::ZERO;
            }

            // === MODULE MANAGEMENT (only when module selected) ===
            if let Some(module_id) = self.active_module_id {
                ui.separator();

                // Get module for editing
                if let Some(module) = manager.get_module_mut(module_id) {
                    // Module name editor
                    ui.label("Name:");
                    let mut name = module.name.clone();
                    let name_response = ui.add(
                        egui::TextEdit::singleline(&mut name)
                            .desired_width(100.0)
                            .hint_text("Module name"),
                    );
                    if name_response.changed() {
                        module.name = name;
                    }

                    // Color picker button (shows current color)
                    let color = Color32::from_rgba_unmultiplied(
                        (module.color[0] * 255.0) as u8,
                        (module.color[1] * 255.0) as u8,
                        (module.color[2] * 255.0) as u8,
                        (module.color[3] * 255.0) as u8,
                    );
                    let color_btn = ui
                        .add(
                            egui::Button::new("🎨")
                                .fill(color)
                                .min_size(Vec2::splat(20.0)),
                        )
                        .on_hover_text("Module timeline color");

                    if color_btn.clicked() {
                        // Cycle through preset colors
                        let presets = [
                            [0.8, 0.3, 0.3, 1.0], // Red
                            [0.3, 0.8, 0.3, 1.0], // Green
                            [0.3, 0.3, 0.8, 1.0], // Blue
                            [0.8, 0.8, 0.3, 1.0], // Yellow
                            [0.8, 0.3, 0.8, 1.0], // Magenta
                            [0.3, 0.8, 0.8, 1.0], // Cyan
                            [0.8, 0.5, 0.2, 1.0], // Orange
                        ];
                        let current_idx =
                            presets.iter().position(|c| *c == module.color).unwrap_or(0);
                        module.color = presets[(current_idx + 1) % presets.len()];
                    }
                }

                // Delete module button
                if ui.button("🗑").on_hover_text("Delete this module").clicked() {
                    manager.delete_module(module_id);
                    self.active_module_id = None;
                }

                ui.separator();

                // Search button
                if ui
                    .button("🔍")
                    .on_hover_text("Search nodes (Ctrl+F)")
                    .clicked()
                {
                    self.show_search = !self.show_search;
                }

                // Auto-layout button
                if ui.button("⊞").on_hover_text("Auto-layout nodes").clicked() {
                    if let Some(module) = manager.get_module_mut(module_id) {
                        Self::auto_layout_parts(&mut module.parts);
                    }
                }

                // Presets button
                if ui
                    .button("📋")
                    .on_hover_text("Load preset template")
                    .clicked()
                {
                    self.show_presets = !self.show_presets;
                }
            }

            ui.add_space(4.0);
        });

        ui.separator();

        // Find the active module
        let active_module = self
            .active_module_id
            .and_then(|id| manager.get_module_mut(id));

        if let Some(module) = active_module {
            // Split view: canvas on left, inspector on right if node selected
            if !self.selected_parts.is_empty() {
                ui.horizontal(|ui| {
                    // Canvas area (left side - takes most space)
                    let canvas_width = ui.available_width() - 300.0;
                    ui.allocate_ui(Vec2::new(canvas_width, ui.available_height()), |ui| {
                        self.render_canvas(ui, module, locale);
                    });

                    ui.separator();

                    // Node Control panel (right side - 280px width)
                    ui.vertical(|ui| {
                        ui.set_min_width(280.0);
                        ui.heading("🎛️ Node Control");
                        ui.separator();

                        // Get first selected part
                        if let Some(part_id) = self.selected_parts.first().copied() {
                            if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                                use mapmap_core::module::*;

                                let (_, _, icon, type_name) = Self::get_part_style(&part.part_type);
                                ui.label(format!("{} {}", icon, type_name));
                                ui.add_space(8.0);

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        match &mut part.part_type {
                                            ModulePartType::Trigger(trigger) => {
                                                ui.label("Trigger Type:");
                                                match trigger {
                                                    TriggerType::Beat => {
                                                        ui.label("🥁 Beat Sync");
                                                        ui.label("Triggers on BPM beat.");
                                                    }
                                                    TriggerType::AudioFFT { band, threshold } => {
                                                        ui.label("🔊 Audio FFT");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Band:");
                                                            egui::ComboBox::from_id_source(
                                                                "audio_band",
                                                            )
                                                            .selected_text(format!("{:?}", band))
                                                            .show_ui(ui, |ui| {
                                                                let bands = [
                                                                    (
                                                                        AudioBand::SubBass,
                                                                        "SubBass (20-60Hz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::Bass,
                                                                        "Bass (60-250Hz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::LowMid,
                                                                        "LowMid (250-500Hz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::Mid,
                                                                        "Mid (500-2kHz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::HighMid,
                                                                        "HighMid (2-4kHz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::Presence,
                                                                        "Presence (4-6kHz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::Brilliance,
                                                                        "Brilliance (6-20kHz)",
                                                                    ),
                                                                    (
                                                                        AudioBand::Peak,
                                                                        "Peak Detection",
                                                                    ),
                                                                    (AudioBand::BPM, "BPM"),
                                                                ];
                                                                for (b, label) in bands {
                                                                    if ui
                                                                        .selectable_label(
                                                                            *band == b,
                                                                            label,
                                                                        )
                                                                        .clicked()
                                                                    {
                                                                        *band = b;
                                                                    }
                                                                }
                                                            });
                                                        });
                                                        ui.add(
                                                            egui::Slider::new(threshold, 0.0..=1.0)
                                                                .text("Threshold"),
                                                        );
                                                    }
                                                    TriggerType::Random {
                                                        min_interval_ms,
                                                        max_interval_ms,
                                                        probability,
                                                    } => {
                                                        ui.label("🎲 Random");
                                                        ui.add(
                                                            egui::Slider::new(
                                                                min_interval_ms,
                                                                50..=5000,
                                                            )
                                                            .text("Min (ms)"),
                                                        );
                                                        ui.add(
                                                            egui::Slider::new(
                                                                max_interval_ms,
                                                                100..=10000,
                                                            )
                                                            .text("Max (ms)"),
                                                        );
                                                        ui.add(
                                                            egui::Slider::new(
                                                                probability,
                                                                0.0..=1.0,
                                                            )
                                                            .text("Probability"),
                                                        );
                                                    }
                                                    TriggerType::Fixed {
                                                        interval_ms,
                                                        offset_ms,
                                                    } => {
                                                        ui.label("⏱️ Fixed Timer");
                                                        ui.add(
                                                            egui::Slider::new(
                                                                interval_ms,
                                                                16..=10000,
                                                            )
                                                            .text("Interval (ms)"),
                                                        );
                                                        ui.add(
                                                            egui::Slider::new(offset_ms, 0..=5000)
                                                                .text("Offset (ms)"),
                                                        );
                                                    }
                                                    TriggerType::Midi { channel, note } => {
                                                        ui.label("🎹 MIDI Trigger");

                                                        // Available MIDI ports dropdown
                                                        ui.horizontal(|ui| {
                                                            ui.label("Device:");
                                                            #[cfg(feature = "midi")]
                                                            {
                                                                if let Ok(ports) = mapmap_control::midi::MidiInputHandler::list_ports() {
                                                                    if ports.is_empty() {
                                                                        ui.label("No MIDI devices");
                                                                    } else {
                                                                        egui::ComboBox::from_id_source("midi_device")
                                                                            .selected_text(ports.first().cloned().unwrap_or_default())
                                                                            .show_ui(ui, |ui| {
                                                                                for port in &ports {
                                                                                    let _ = ui.selectable_label(false, port);
                                                                                }
                                                                            });
                                                                    }
                                                                } else {
                                                                    ui.label("MIDI unavailable");
                                                                }
                                                            }
                                                            #[cfg(not(feature = "midi"))]
                                                            {
                                                                ui.label("(MIDI disabled)");
                                                            }
                                                        });

                                                        ui.add(
                                                            egui::Slider::new(channel, 1..=16)
                                                                .text("Channel"),
                                                        );
                                                        ui.add(
                                                            egui::Slider::new(note, 0..=127)
                                                                .text("Note"),
                                                        );

                                                        // MIDI Learn button
                                                        let is_learning = self.midi_learn_part_id == Some(part_id);
                                                        let learn_text = if is_learning {
                                                            "⏳ Waiting for MIDI..."
                                                        } else {
                                                            "🎯 MIDI Learn"
                                                        };
                                                        if ui.button(learn_text).clicked() {
                                                            if is_learning {
                                                                self.midi_learn_part_id = None;
                                                            } else {
                                                                self.midi_learn_part_id = Some(part_id);
                                                            }
                                                        }
                                                        if is_learning {
                                                            ui.label("Press any MIDI key/knob...");
                                                        }
                                                    }
                                                    TriggerType::Osc { address } => {
                                                        ui.label("📡 OSC Trigger");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Address:");
                                                            ui.add(egui::TextEdit::singleline(address).desired_width(150.0));
                                                        });
                                                        ui.label("Format: /path/to/trigger");
                                                        ui.label("Default port: 8000");
                                                    }
                                                    TriggerType::Shortcut {
                                                        key_code,
                                                        modifiers,
                                                    } => {
                                                        ui.label("⌨️ Shortcut");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Key:");
                                                            ui.text_edit_singleline(key_code);
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Mods:");
                                                            ui.label(format!(
                                                                "Ctrl={} Shift={} Alt={}",
                                                                *modifiers & 1 != 0,
                                                                *modifiers & 2 != 0,
                                                                *modifiers & 4 != 0
                                                            ));
                                                        });
                                                    }
                                                }
                                            }
                                            ModulePartType::Source(source) => {
                                                ui.label("Source Type:");
                                                match source {
                                                    SourceType::MediaFile { path } => {
                                                        ui.label("📁 Media File");
                                                        ui.horizontal(|ui| {
                                                            ui.add(
                                                                egui::TextEdit::singleline(path)
                                                                    .desired_width(120.0),
                                                            );
                                                            if ui.button("📂").clicked() {
                                                                if let Some(picked) =
                                                                    rfd::FileDialog::new()
                                                                        .add_filter(
                                                                            "Media",
                                                                            &[
                                                                                "mp4", "mov",
                                                                                "avi", "mkv",
                                                                                "webm", "gif",
                                                                                "png", "jpg",
                                                                                "jpeg",
                                                                            ],
                                                                        )
                                                                        .pick_file()
                                                                {
                                                                    *path = picked
                                                                        .display()
                                                                        .to_string();
                                                                }
                                                            }
                                                        });
                                                    }
                                                    SourceType::Shader { name, params: _ } => {
                                                        ui.label("🎨 Shader");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.text_edit_singleline(name);
                                                        });
                                                    }
                                                    SourceType::LiveInput { device_id } => {
                                                        ui.label("📹 Live Input");
                                                        ui.add(
                                                            egui::Slider::new(device_id, 0..=10)
                                                                .text("Device ID"),
                                                        );
                                                    }
                                                }
                                            }
                                            ModulePartType::Mask(mask) => {
                                                ui.label("Mask Type:");
                                                match mask {
                                                    MaskType::File { path } => {
                                                        ui.label("📁 Mask File");
                                                        ui.horizontal(|ui| {
                                                            ui.add(
                                                                egui::TextEdit::singleline(path)
                                                                    .desired_width(120.0),
                                                            );
                                                            if ui.button("📂").clicked() {
                                                                if let Some(picked) =
                                                                    rfd::FileDialog::new()
                                                                        .add_filter(
                                                                            "Image",
                                                                            &[
                                                                                "png", "jpg",
                                                                                "jpeg", "webp",
                                                                                "bmp",
                                                                            ],
                                                                        )
                                                                        .pick_file()
                                                                {
                                                                    *path = picked
                                                                        .display()
                                                                        .to_string();
                                                                }
                                                            }
                                                        });
                                                    }
                                                    MaskType::Shape(shape) => {
                                                        ui.label("🔷 Shape Mask");
                                                        egui::ComboBox::from_id_source(
                                                            "mask_shape",
                                                        )
                                                        .selected_text(format!("{:?}", shape))
                                                        .show_ui(ui, |ui| {
                                                            if ui
                                                                .selectable_label(
                                                                    matches!(
                                                                        shape,
                                                                        MaskShape::Circle
                                                                    ),
                                                                    "Circle",
                                                                )
                                                                .clicked()
                                                            {
                                                                *shape = MaskShape::Circle;
                                                            }
                                                            if ui
                                                                .selectable_label(
                                                                    matches!(
                                                                        shape,
                                                                        MaskShape::Rectangle
                                                                    ),
                                                                    "Rectangle",
                                                                )
                                                                .clicked()
                                                            {
                                                                *shape = MaskShape::Rectangle;
                                                            }
                                                            if ui
                                                                .selectable_label(
                                                                    matches!(
                                                                        shape,
                                                                        MaskShape::Triangle
                                                                    ),
                                                                    "Triangle",
                                                                )
                                                                .clicked()
                                                            {
                                                                *shape = MaskShape::Triangle;
                                                            }
                                                            if ui
                                                                .selectable_label(
                                                                    matches!(
                                                                        shape,
                                                                        MaskShape::Star
                                                                    ),
                                                                    "Star",
                                                                )
                                                                .clicked()
                                                            {
                                                                *shape = MaskShape::Star;
                                                            }
                                                            if ui
                                                                .selectable_label(
                                                                    matches!(
                                                                        shape,
                                                                        MaskShape::Ellipse
                                                                    ),
                                                                    "Ellipse",
                                                                )
                                                                .clicked()
                                                            {
                                                                *shape = MaskShape::Ellipse;
                                                            }
                                                        });
                                                    }
                                                    MaskType::Gradient { angle, softness } => {
                                                        ui.label("🌈 Gradient Mask");
                                                        ui.add(
                                                            egui::Slider::new(angle, 0.0..=360.0)
                                                                .text("Angle °"),
                                                        );
                                                        ui.add(
                                                            egui::Slider::new(softness, 0.0..=1.0)
                                                                .text("Softness"),
                                                        );
                                                    }
                                                }
                                            }
                                            ModulePartType::Modulizer(mod_type) => {
                                                ui.label("Modulator:");
                                                match mod_type {
                                                    ModulizerType::Effect(effect) => {
                                                        ui.label("✨ Effect");
                                                        egui::ComboBox::from_id_source("effect_type")
                                                            .selected_text(format!("{:?}", effect))
                                                            .show_ui(ui, |ui| {
                                                                // Basic
                                                                ui.label("--- Basic ---");
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Blur), "Blur").clicked() { *effect = ModuleEffectType::Blur; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Sharpen), "Sharpen").clicked() { *effect = ModuleEffectType::Sharpen; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Invert), "Invert").clicked() { *effect = ModuleEffectType::Invert; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Threshold), "Threshold").clicked() { *effect = ModuleEffectType::Threshold; }
                                                                // Color
                                                                ui.label("--- Color ---");
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Brightness), "Brightness").clicked() { *effect = ModuleEffectType::Brightness; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Contrast), "Contrast").clicked() { *effect = ModuleEffectType::Contrast; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Saturation), "Saturation").clicked() { *effect = ModuleEffectType::Saturation; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::HueShift), "Hue Shift").clicked() { *effect = ModuleEffectType::HueShift; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Colorize), "Colorize").clicked() { *effect = ModuleEffectType::Colorize; }
                                                                // Distortion
                                                                ui.label("--- Distort ---");
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Wave), "Wave").clicked() { *effect = ModuleEffectType::Wave; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Spiral), "Spiral").clicked() { *effect = ModuleEffectType::Spiral; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Pinch), "Pinch").clicked() { *effect = ModuleEffectType::Pinch; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Mirror), "Mirror").clicked() { *effect = ModuleEffectType::Mirror; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Kaleidoscope), "Kaleidoscope").clicked() { *effect = ModuleEffectType::Kaleidoscope; }
                                                                // Stylize
                                                                ui.label("--- Stylize ---");
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Pixelate), "Pixelate").clicked() { *effect = ModuleEffectType::Pixelate; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Halftone), "Halftone").clicked() { *effect = ModuleEffectType::Halftone; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::EdgeDetect), "Edge Detect").clicked() { *effect = ModuleEffectType::EdgeDetect; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Posterize), "Posterize").clicked() { *effect = ModuleEffectType::Posterize; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::Glitch), "Glitch").clicked() { *effect = ModuleEffectType::Glitch; }
                                                                // Composite
                                                                ui.label("--- Composite ---");
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::RgbSplit), "RGB Split").clicked() { *effect = ModuleEffectType::RgbSplit; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::ChromaticAberration), "Chromatic").clicked() { *effect = ModuleEffectType::ChromaticAberration; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::VHS), "VHS").clicked() { *effect = ModuleEffectType::VHS; }
                                                                if ui.selectable_label(matches!(effect, ModuleEffectType::FilmGrain), "Film Grain").clicked() { *effect = ModuleEffectType::FilmGrain; }
                                                            });
                                                        // TODO: Add effect-specific parameter sliders
                                                        ui.add(egui::Slider::new(&mut 0.5_f32, 0.0..=1.0).text("Intensity"));
                                                    }
                                                    ModulizerType::BlendMode(blend) => {
                                                        ui.label("🎨 Blend Mode");
                                                        egui::ComboBox::from_id_source("blend_mode")
                                                            .selected_text(format!("{:?}", blend))
                                                            .show_ui(ui, |ui| {
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Normal), "Normal").clicked() { *blend = BlendModeType::Normal; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Add), "Add").clicked() { *blend = BlendModeType::Add; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Multiply), "Multiply").clicked() { *blend = BlendModeType::Multiply; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Screen), "Screen").clicked() { *blend = BlendModeType::Screen; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Overlay), "Overlay").clicked() { *blend = BlendModeType::Overlay; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Difference), "Difference").clicked() { *blend = BlendModeType::Difference; }
                                                                if ui.selectable_label(matches!(blend, BlendModeType::Exclusion), "Exclusion").clicked() { *blend = BlendModeType::Exclusion; }
                                                            });
                                                        ui.add(egui::Slider::new(&mut 1.0_f32, 0.0..=1.0).text("Opacity"));
                                                    }
                                                    ModulizerType::AudioReactive { source } => {
                                                        ui.label("🔊 Audio Reactive");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Source:");
                                                            egui::ComboBox::from_id_source("audio_source")
                                                                .selected_text(source.as_str())
                                                                .show_ui(ui, |ui| {
                                                                    if ui.selectable_label(source == "SubBass", "SubBass").clicked() { *source = "SubBass".to_string(); }
                                                                    if ui.selectable_label(source == "Bass", "Bass").clicked() { *source = "Bass".to_string(); }
                                                                    if ui.selectable_label(source == "LowMid", "LowMid").clicked() { *source = "LowMid".to_string(); }
                                                                    if ui.selectable_label(source == "Mid", "Mid").clicked() { *source = "Mid".to_string(); }
                                                                    if ui.selectable_label(source == "HighMid", "HighMid").clicked() { *source = "HighMid".to_string(); }
                                                                    if ui.selectable_label(source == "Presence", "Presence").clicked() { *source = "Presence".to_string(); }
                                                                    if ui.selectable_label(source == "Brilliance", "Brilliance").clicked() { *source = "Brilliance".to_string(); }
                                                                    if ui.selectable_label(source == "RMS", "RMS Volume").clicked() { *source = "RMS".to_string(); }
                                                                    if ui.selectable_label(source == "Peak", "Peak").clicked() { *source = "Peak".to_string(); }
                                                                    if ui.selectable_label(source == "BPM", "BPM").clicked() { *source = "BPM".to_string(); }
                                                                });
                                                        });
                                                        ui.add(egui::Slider::new(&mut 1.0_f32, 0.0..=2.0).text("Sensitivity"));
                                                        ui.add(egui::Slider::new(&mut 0.1_f32, 0.0..=1.0).text("Smoothing"));
                                                    }
                                                }
                                            }
                                            ModulePartType::LayerAssignment(layer) => {
                                                ui.label("📋 Layer Assignment:");
                                                match layer {
                                                    LayerAssignmentType::SingleLayer { id, name, opacity, blend_mode } => {
                                                        ui.label("🔲 Single Layer");
                                                        ui.horizontal(|ui| {
                                                            ui.label("ID:");
                                                            let mut id_u32 = *id as u32;
                                                            if ui.add(egui::Slider::new(&mut id_u32, 0..=99).text("")).changed() {
                                                                *id = id_u32 as u64;
                                                            }
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.text_edit_singleline(name);
                                                        });
                                                        ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                                                        // Blend Mode selector
                                                        let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or("None".to_string());
                                                        egui::ComboBox::from_id_source("layer_blend")
                                                            .selected_text(blend_text)
                                                            .show_ui(ui, |ui| {
                                                                if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Multiply)), "Multiply").clicked() { *blend_mode = Some(BlendModeType::Multiply); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Screen)), "Screen").clicked() { *blend_mode = Some(BlendModeType::Screen); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Overlay)), "Overlay").clicked() { *blend_mode = Some(BlendModeType::Overlay); }
                                                            });
                                                    }
                                                    LayerAssignmentType::Group { name, opacity, blend_mode } => {
                                                        ui.label("📂 Layer Group");
                                                        ui.horizontal(|ui| {
                                                            ui.label("Group Name:");
                                                            ui.text_edit_singleline(name);
                                                        });
                                                        ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Group Opacity"));
                                                        let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or("None".to_string());
                                                        egui::ComboBox::from_id_source("group_blend")
                                                            .selected_text(blend_text)
                                                            .show_ui(ui, |ui| {
                                                                if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Multiply)), "Multiply").clicked() { *blend_mode = Some(BlendModeType::Multiply); }
                                                            });
                                                    }
                                                    LayerAssignmentType::AllLayers { opacity, blend_mode } => {
                                                        ui.label("🎚️ All Layers (Master)");
                                                        ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Master Opacity"));
                                                        let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or("None".to_string());
                                                        egui::ComboBox::from_id_source("master_blend")
                                                            .selected_text(blend_text)
                                                            .show_ui(ui, |ui| {
                                                                if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                                                                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                                                            });
                                                    }
                                                }
                                            }
                                            ModulePartType::Mesh(mesh_type) => {
                                                ui.label("Mesh:");
                                                match mesh_type {
                                                    MeshType::Quad { tl, tr, br, bl } => {
                                                        ui.label("⬜ Quad Mesh");
                                                        ui.separator();
                                                        ui.label("Corner Mapping:");
                                                        
                                                        let mut coord_ui = |name: &str, coord: &mut (f32, f32)| {
                                                            ui.horizontal(|ui| {
                                                                ui.label(name);
                                                                ui.add(egui::DragValue::new(&mut coord.0).speed(0.01).clamp_range(0.0..=1.0).prefix("X: "));
                                                                ui.add(egui::DragValue::new(&mut coord.1).speed(0.01).clamp_range(0.0..=1.0).prefix("Y: "));
                                                            });
                                                        };
                                                        
                                                        coord_ui("Top Left:", tl);
                                                        coord_ui("Top Right:", tr);
                                                        coord_ui("Bottom Right:", br);
                                                        coord_ui("Bottom Left:", bl);
                                                        
                                                        ui.separator();
                                                        ui.label("Visual Editor:");
                                                        
                                                        let (response, painter) = ui.allocate_painter(Vec2::new(240.0, 180.0), Sense::click_and_drag());
                                                        let rect = response.rect;
                                                        
                                                        // Draw background
                                                        painter.rect_filled(rect, 0.0, Color32::from_gray(30));
                                                        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));
                                                        
                                                        let to_screen = |norm: (f32, f32)| -> Pos2 {
                                                            Pos2::new(
                                                                rect.min.x + norm.0 * rect.width(),
                                                                rect.min.y + norm.1 * rect.height()
                                                            )
                                                        };
                                                        
                                                        let from_screen = |pos: Pos2| -> (f32, f32) {
                                                            (
                                                                ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
                                                                ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0)
                                                            )
                                                        };

                                                        // Draw Quad Lines
                                                        let p_tl = to_screen(*tl);
                                                        let p_tr = to_screen(*tr);
                                                        let p_br = to_screen(*br);
                                                        let p_bl = to_screen(*bl);
                                                        
                                                        painter.add(egui::Shape::convex_polygon(
                                                            vec![p_tl, p_tr, p_br, p_bl],
                                                            Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                                                            Stroke::new(1.0, Color32::LIGHT_BLUE),
                                                        ));
                                                        
                                                        // Handles
                                                        let draw_handle = |coord: &mut (f32, f32), name: &str| {
                                                            let pos = to_screen(*coord);
                                                            let handle_radius = 6.0;
                                                            let handle_rect = Rect::from_center_size(pos, Vec2::splat(handle_radius * 2.0));
                                                            
                                                            // Interaction
                                                            let handle_id = response.id.with(name);
                                                            let handle_response = ui.interact(handle_rect, handle_id, Sense::drag());
                                                            
                                                            if handle_response.dragged() {
                                                                if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                                                    *coord = from_screen(mouse_pos);
                                                                }
                                                            }
                                                            
                                                            let color = if handle_response.hovered() || handle_response.dragged() {
                                                                Color32::WHITE
                                                            } else {
                                                                Color32::LIGHT_BLUE
                                                            };
                                                            
                                                            painter.circle_filled(pos, handle_radius, color);
                                                        };
                                                        
                                                        draw_handle(tl, "tl");
                                                        draw_handle(tr, "tr");
                                                        draw_handle(br, "br");
                                                        draw_handle(bl, "bl");
                                                    }
                                                    MeshType::Grid { rows, cols } => {
                                                        ui.label("▦ Grid Mesh");
                                                        ui.add(egui::Slider::new(rows, 1..=16).text("Rows"));
                                                        ui.add(egui::Slider::new(cols, 1..=16).text("Cols"));
                                                    }
                                                    MeshType::BezierSurface { .. } => {
                                                        ui.label("〰️ Bezier Surface");
                                                        ui.label("Curved surface with control points");
                                                    }
                                                    MeshType::Polygon { .. } => {
                                                        ui.label("⬡ Polygon Mesh");
                                                        ui.label("Freeform polygon vertices");
                                                    }
                                                    MeshType::TriMesh => {
                                                        ui.label("🔺 Triangle Mesh");
                                                    }
                                                    MeshType::Circle { segments, arc_angle } => {
                                                        ui.label("⭕ Circle/Arc");
                                                        ui.add(egui::Slider::new(segments, 8..=64).text("Segments"));
                                                        ui.add(egui::Slider::new(arc_angle, 0.0..=360.0).text("Arc °"));
                                                    }
                                                    MeshType::Cylinder { segments, height } => {
                                                        ui.label("🌐 Cylinder");
                                                        ui.add(egui::Slider::new(segments, 8..=64).text("Segments"));
                                                        ui.add(egui::Slider::new(height, 0.1..=10.0).text("Height"));
                                                    }
                                                    MeshType::Sphere { lat_segments, lon_segments } => {
                                                        ui.label("🌍 Sphere/Dome");
                                                        ui.add(egui::Slider::new(lat_segments, 4..=32).text("Lat Segs"));
                                                        ui.add(egui::Slider::new(lon_segments, 4..=64).text("Lon Segs"));
                                                    }
                                                    MeshType::Custom { path } => {
                                                        ui.label("📁 Custom Mesh");
                                                        ui.horizontal(|ui| {
                                                            ui.label("File:");
                                                            ui.text_edit_singleline(path);
                                                        });
                                                    }
                                                }
                                            }
                                            ModulePartType::Output(output) => {
                                                ui.label("Output:");
                                                match output {
                                                    OutputType::Projector { id, name } => {
                                                        ui.label("📽️ Projector");
                                                        ui.add(
                                                            egui::Slider::new(id, 0..=8).text("ID"),
                                                        );
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.text_edit_singleline(name);
                                                        });
                                                    }
                                                    OutputType::Preview { window_id: _ } => {
                                                        ui.label("👁️ Preview Window");
                                                    }
                                                }
                                            }
                                        }

                                        ui.add_space(16.0);
                                        ui.separator();

                                        // Node position info
                                        ui.label(format!(
                                            "Position: ({:.0}, {:.0})",
                                            part.position.0, part.position.1
                                        ));
                                        if let Some((w, h)) = part.size {
                                            ui.label(format!("Size: {:.0} × {:.0}", w, h));
                                        }
                                        ui.label(format!("Inputs: {}", part.inputs.len()));
                                        ui.label(format!("Outputs: {}", part.outputs.len()));
                                    });
                            }
                        }
                    });
                });
            } else {
                self.render_canvas(ui, module, locale);

                // Context Menu Logic
                if let Some(menu_pos) = self.context_menu_pos {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default);
                    let menu_rect = Rect::from_min_size(menu_pos, Vec2::new(140.0, 100.0));

                    // Check for click outside to close
                    if ui.input(|i| i.pointer.any_pressed()) {
                        let pointer = ui.input(|i| i.pointer.hover_pos());
                        if let Some(pos) = pointer {
                            if !menu_rect.contains(pos) {
                                self.context_menu_pos = None;
                                self.context_menu_connection = None;
                                self.context_menu_part = None;
                            }
                        }
                    }

                    if self.context_menu_pos.is_some() {
                        egui::Window::new("Context Menu")
                            .fixed_pos(menu_pos)
                            .collapsible(false)
                            .resizable(false)
                            .title_bar(false)
                            .frame(egui::Frame::popup(ui.style()))
                            .show(ui.ctx(), |ui| {
                                if let Some(conn_idx) = self.context_menu_connection {
                                    if ui.button("🗑 Delete Connection").clicked() {
                                        if conn_idx < module.connections.len() {
                                            module.connections.remove(conn_idx);
                                        }
                                        self.context_menu_pos = None;
                                        self.context_menu_connection = None;
                                    }
                                }
                                if let Some(part_id) = self.context_menu_part {
                                    if ui.button("🗑 Delete Node").clicked() {
                                        // Remove connections
                                        module.connections.retain(|c| {
                                            c.from_part != part_id && c.to_part != part_id
                                        });
                                        // Remove part
                                        module.parts.retain(|p| p.id != part_id);
                                        self.context_menu_pos = None;
                                        self.context_menu_part = None;
                                    }
                                    if ui.button("📄 Duplicate Node").clicked() {
                                        // Find part to duplicate
                                        if let Some(part) =
                                            module.parts.iter().find(|p| p.id == part_id).cloned()
                                        {
                                            // Generate new ID locally to avoid borrowing manager/module conflict
                                            let new_id = module
                                                .parts
                                                .iter()
                                                .map(|p| p.id)
                                                .max()
                                                .unwrap_or(0)
                                                + 1;
                                            let mut new_part = part.clone();
                                            new_part.id = new_id;
                                            new_part.position.0 += 20.0;
                                            new_part.position.1 += 20.0;
                                            module.parts.push(new_part);
                                        }
                                        self.context_menu_pos = None;
                                        self.context_menu_part = None;
                                    }
                                }
                            });
                    }
                }
            }
        } else {
            // Show a message if no module is selected
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("🔧 Module Canvas");
                    ui.add_space(10.0);
                    ui.label("Click '➕ New Module' to create a module.");
                    ui.label("Or select an existing module from the dropdown above.");
                });
            });
        }
    }

    fn render_canvas(&mut self, ui: &mut Ui, module: &mut MapFlowModule, _locale: &LocaleManager) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Store drag_started state before we check parts
        let drag_started_on_empty = response.drag_started() && self.dragging_part.is_none();
        
        // Handle canvas pan (only when not dragging a part and not creating connection)
        // We also need middle mouse button for panning to avoid conflicts
        let middle_button = ui.input(|i| i.pointer.middle_down());
        if response.dragged() && self.dragging_part.is_none() && self.creating_connection.is_none() {
            // Only pan with middle mouse or when not over a part
            if middle_button || self.panning_canvas {
                self.pan_offset += response.drag_delta();
            }
        }
        
        // Track if we started panning (for continuing the pan)
        if drag_started_on_empty && !middle_button {
            // Will be set to true if click was on empty canvas
            self.panning_canvas = false;
        }
        if !response.dragged() {
            self.panning_canvas = false;
        }

        // Handle keyboard shortcuts
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
        let shift_held = ui.input(|i| i.modifiers.shift);

        // Ctrl+C: Copy selected parts
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::C)) && !self.selected_parts.is_empty()
        {
            self.clipboard.clear();
            // Find center of selection for relative positioning
            let center = if !self.selected_parts.is_empty() {
                let sum: (f32, f32) = module
                    .parts
                    .iter()
                    .filter(|p| self.selected_parts.contains(&p.id))
                    .map(|p| p.position)
                    .fold((0.0, 0.0), |acc, pos| (acc.0 + pos.0, acc.1 + pos.1));
                let count = self.selected_parts.len() as f32;
                (sum.0 / count, sum.1 / count)
            } else {
                (0.0, 0.0)
            };

            for part in module
                .parts
                .iter()
                .filter(|p| self.selected_parts.contains(&p.id))
            {
                let relative_pos = (part.position.0 - center.0, part.position.1 - center.1);
                self.clipboard.push((part.part_type.clone(), relative_pos));
            }
        }

        // Ctrl+V: Paste from clipboard
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::V)) && !self.clipboard.is_empty() {
            let paste_offset = (50.0, 50.0); // Offset from original position
            self.selected_parts.clear();

            for (part_type, rel_pos) in self.clipboard.clone() {
                let new_pos = (
                    rel_pos.0 + paste_offset.0 + 100.0,
                    rel_pos.1 + paste_offset.1 + 100.0,
                );
                let part_type_variant = Self::part_type_from_module_part_type(&part_type);
                let new_id = module.add_part(part_type_variant, new_pos);
                self.selected_parts.push(new_id);
            }
        }

        // Ctrl+A: Select all
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::A)) {
            self.selected_parts = module.parts.iter().map(|p| p.id).collect();
        }

        // Delete: Delete selected parts
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) && !self.selected_parts.is_empty() {
            for part_id in self.selected_parts.clone() {
                module
                    .connections
                    .retain(|c| c.from_part != part_id && c.to_part != part_id);
                module.parts.retain(|p| p.id != part_id);
            }
            self.selected_parts.clear();
        }

        // Escape: Deselect all or close search
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_search {
                self.show_search = false;
            } else {
                self.selected_parts.clear();
            }
        }

        // Ctrl+F: Toggle search popup
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::F)) {
            self.show_search = !self.show_search;
            if self.show_search {
                self.search_filter.clear();
            }
        }

        // Ctrl+Z: Undo
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Z)) && !self.undo_stack.is_empty() {
            if let Some(action) = self.undo_stack.pop() {
                match &action {
                    CanvasAction::AddPart { part_id, .. } => {
                        // Undo add = delete
                        module.parts.retain(|p| p.id != *part_id);
                    }
                    CanvasAction::DeletePart { part_data } => {
                        // Undo delete = restore
                        module.parts.push(part_data.clone());
                    }
                    CanvasAction::MovePart {
                        part_id, old_pos, ..
                    } => {
                        // Undo move = restore old position
                        if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                            part.position = *old_pos;
                        }
                    }
                    CanvasAction::AddConnection { connection } => {
                        // Undo add connection = delete
                        module.connections.retain(|c| {
                            !(c.from_part == connection.from_part
                                && c.to_part == connection.to_part
                                && c.from_socket == connection.from_socket
                                && c.to_socket == connection.to_socket)
                        });
                    }
                    CanvasAction::DeleteConnection { connection } => {
                        // Undo delete connection = restore
                        module.connections.push(connection.clone());
                    }
                }
                self.redo_stack.push(action);
            }
        }

        // Ctrl+Y: Redo
        if ctrl_held && ui.input(|i| i.key_pressed(egui::Key::Y)) && !self.redo_stack.is_empty() {
            if let Some(action) = self.redo_stack.pop() {
                match &action {
                    CanvasAction::AddPart { part_data, .. } => {
                        // Redo add = add again
                        module.parts.push(part_data.clone());
                    }
                    CanvasAction::DeletePart { part_data } => {
                        // Redo delete = delete again
                        module.parts.retain(|p| p.id != part_data.id);
                    }
                    CanvasAction::MovePart {
                        part_id, new_pos, ..
                    } => {
                        // Redo move = apply new position
                        if let Some(part) = module.parts.iter_mut().find(|p| p.id == *part_id) {
                            part.position = *new_pos;
                        }
                    }
                    CanvasAction::AddConnection { connection } => {
                        // Redo add connection = add again
                        module.connections.push(connection.clone());
                    }
                    CanvasAction::DeleteConnection { connection } => {
                        // Redo delete connection = delete again
                        module.connections.retain(|c| {
                            !(c.from_part == connection.from_part
                                && c.to_part == connection.to_part
                                && c.from_socket == connection.from_socket
                                && c.to_socket == connection.to_socket)
                        });
                    }
                }
                self.undo_stack.push(action);
            }
        }

        // For shift_held - used in click handling below
        let _ = shift_held;

        // Handle zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom *= 1.0 + scroll * 0.001;
                self.zoom = self.zoom.clamp(0.2, 3.0);
            }
        }

        let to_screen =
            |pos: Pos2| -> Pos2 { canvas_rect.min + (pos.to_vec2() + self.pan_offset) * self.zoom };

        let _from_screen = |screen_pos: Pos2| -> Pos2 {
            let v = (screen_pos - canvas_rect.min) / self.zoom - self.pan_offset;
            Pos2::new(v.x, v.y)
        };

        // Draw grid
        self.draw_grid(&painter, canvas_rect);

        // Draw connections first (behind nodes)
        self.draw_connections(&painter, module, &to_screen);

        // Collect socket positions for hit detection
        let mut all_sockets: Vec<SocketInfo> = Vec::new();

        // Collect part info and socket positions
        let part_rects: Vec<_> = module
            .parts
            .iter()
            .map(|part| {
                let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_size = Vec2::new(200.0, part_height);
                let rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

                // Calculate socket positions
                let title_height = 28.0 * self.zoom;
                let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;

                // Input sockets (left side)
                for (i, socket) in part.inputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: false,
                        socket_type: socket.socket_type.clone(),
                        position: Pos2::new(rect.min.x, socket_y),
                    });
                }

                // Output sockets (right side)
                for (i, socket) in part.outputs.iter().enumerate() {
                    let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
                    all_sockets.push(SocketInfo {
                        part_id: part.id,
                        socket_idx: i,
                        is_output: true,
                        socket_type: socket.socket_type.clone(),
                        position: Pos2::new(rect.max.x, socket_y),
                    });
                }

                (part.id, rect)
            })
            .collect();

        // Handle socket clicks for creating connections
        let socket_radius = 8.0 * self.zoom;
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let primary_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let primary_released = ui.input(|i| i.pointer.any_released());
        let clicked = ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
        let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));

        // Start connection on mouse down over socket
        if let Some(pos) = pointer_pos {
            if primary_down && self.creating_connection.is_none() && self.dragging_part.is_none() {
                for socket in &all_sockets {
                    if socket.position.distance(pos) < socket_radius {
                        // Start creating a connection
                        self.creating_connection = Some((
                            socket.part_id,
                            socket.socket_idx,
                            socket.is_output,
                            socket.socket_type.clone(),
                            socket.position,
                        ));
                        break;
                    }
                }
            }

            // Complete connection on release over compatible socket
            if primary_released && self.creating_connection.is_some() {
                if let Some((from_part, from_socket, from_is_output, ref _from_type, _)) =
                    self.creating_connection.clone()
                {
                    for socket in &all_sockets {
                        if socket.position.distance(pos) < socket_radius * 1.5 {
                            // Validate connection: must be different parts, opposite directions
                            // Type check relaxed for now - allow any connection for testing
                            if socket.part_id != from_part
                                && socket.is_output != from_is_output
                            {
                                // Create connection (from output to input)
                                if from_is_output {
                                    module.add_connection(
                                        from_part,
                                        from_socket,
                                        socket.part_id,
                                        socket.socket_idx,
                                    );
                                } else {
                                    module.add_connection(
                                        socket.part_id,
                                        socket.socket_idx,
                                        from_part,
                                        from_socket,
                                    );
                                }
                            }
                            break;
                        }
                    }
                }
                self.creating_connection = None;
            }
        }

        // Clear connection if mouse released without hitting a socket
        if primary_released && self.creating_connection.is_some() {
            self.creating_connection = None;
        }

        // Draw wire preview while dragging (visual feedback)
        if let Some((_, _, is_output, ref socket_type, start_pos)) = self.creating_connection.clone() {
            if let Some(mouse_pos) = pointer_pos {
                // Draw bezier curve from start to mouse
                let wire_color = Self::get_socket_color(&socket_type);
                let control_offset = 50.0 * self.zoom;
                
                // Calculate control points for smooth curve
                let (ctrl1, ctrl2) = if is_output {
                    // Dragging from output (right side) - curve goes right then to mouse
                    (
                        Pos2::new(start_pos.x + control_offset, start_pos.y),
                        Pos2::new(mouse_pos.x - control_offset, mouse_pos.y),
                    )
                } else {
                    // Dragging from input (left side) - curve goes left then to mouse
                    (
                        Pos2::new(start_pos.x - control_offset, start_pos.y),
                        Pos2::new(mouse_pos.x + control_offset, mouse_pos.y),
                    )
                };

                // Draw bezier path
                let segments = 20;
                for i in 0..segments {
                    let t0 = i as f32 / segments as f32;
                    let t1 = (i + 1) as f32 / segments as f32;
                    let p0 = Self::bezier_point(start_pos, ctrl1, ctrl2, mouse_pos, t0);
                    let p1 = Self::bezier_point(start_pos, ctrl1, ctrl2, mouse_pos, t1);
                    painter.line_segment([p0, p1], Stroke::new(3.0 * self.zoom, wire_color));
                }

                // Draw endpoint circle at mouse
                painter.circle_filled(mouse_pos, 6.0 * self.zoom, wire_color);
            }
        }

        // Handle right-click for context menu
        let right_clicked = ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary));
        if right_clicked {
            if let Some(pos) = pointer_pos {
                // Check if clicking near a connection line
                for (conn_idx, conn) in module.connections.iter().enumerate() {
                    // Find positions of connected sockets
                    if let (Some(from_part), Some(to_part)) = (
                        module.parts.iter().find(|p| p.id == conn.from_part),
                        module.parts.iter().find(|p| p.id == conn.to_part),
                    ) {
                        // Adjust for socket offset (approximation)
                        let from_socket_y = 50.0 + conn.from_socket as f32 * 20.0;
                        let from_screen_socket = to_screen(Pos2::new(
                            from_part.position.0 + 180.0,
                            from_part.position.1 + from_socket_y,
                        ));

                        let to_socket_y = 50.0 + conn.to_socket as f32 * 20.0;
                        let to_screen_socket = to_screen(Pos2::new(
                            to_part.position.0,
                            to_part.position.1 + to_socket_y,
                        ));

                        // Simple distance check to bezier curve (approximate with line)
                        let mid = Pos2::new(
                            (from_screen_socket.x + to_screen_socket.x) / 2.0,
                            (from_screen_socket.y + to_screen_socket.y) / 2.0,
                        );
                        if pos.distance(mid) < 20.0 * self.zoom {
                            self.context_menu_pos = Some(pos);
                            self.context_menu_connection = Some(conn_idx);
                            break;
                        }
                    }
                }
            }
        }

        // Handle box selection start (on empty canvas)
        if clicked && self.creating_connection.is_none() && self.dragging_part.is_none() {
            if let Some(pos) = pointer_pos {
                // Check if not clicking on any part
                let on_part = part_rects.iter().any(|(_, rect)| rect.contains(pos));
                if !on_part && canvas_rect.contains(pos) {
                    self.box_select_start = Some(pos);
                }
            }
        }

        // Handle box selection drag
        if let Some(start_pos) = self.box_select_start {
            if let Some(current_pos) = pointer_pos {
                // Draw selection rectangle
                let select_rect = Rect::from_two_pos(start_pos, current_pos);
                painter.rect_stroke(
                    select_rect,
                    0.0,
                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                );
                painter.rect_filled(
                    select_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 200, 255, 30),
                );
            }

            if released {
                // Select all parts within the box
                if let Some(current_pos) = pointer_pos {
                    let select_rect = Rect::from_two_pos(start_pos, current_pos);
                    if !shift_held {
                        self.selected_parts.clear();
                    }
                    for (part_id, part_rect) in &part_rects {
                        if select_rect.intersects(*part_rect)
                            && !self.selected_parts.contains(part_id)
                        {
                            self.selected_parts.push(*part_id);
                        }
                    }
                }
                self.box_select_start = None;
            }
        }

        // Handle part dragging and delete buttons
        let mut delete_part_id: Option<ModulePartId> = None;

        for (part_id, rect) in &part_rects {
            let part_response =
                ui.interact(*rect, egui::Id::new(*part_id), Sense::click_and_drag());

            // Handle click for selection
            if part_response.clicked() && self.creating_connection.is_none() {
                if shift_held {
                    // Shift+Click: toggle selection
                    if self.selected_parts.contains(part_id) {
                        self.selected_parts.retain(|id| id != part_id);
                    } else {
                        self.selected_parts.push(*part_id);
                    }
                } else {
                    // Normal click: replace selection
                    self.selected_parts.clear();
                    self.selected_parts.push(*part_id);
                }
            }

            if part_response.drag_started() && self.creating_connection.is_none() {
                self.dragging_part = Some((*part_id, Vec2::ZERO));
                // If dragging a non-selected part, select only it
                if !self.selected_parts.contains(part_id) {
                    self.selected_parts.clear();
                    self.selected_parts.push(*part_id);
                }
            }

            if part_response.dragged() {
                if let Some((dragged_id, _)) = self.dragging_part {
                    if dragged_id == *part_id {
                        let delta = part_response.drag_delta() / self.zoom;

                        // Calculate new position
                        if let Some(part) = module.parts.iter().find(|p| p.id == *part_id) {
                            let new_x = part.position.0 + delta.x;
                            let new_y = part.position.1 + delta.y;
                            let part_height =
                                80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                            let new_rect = Rect::from_min_size(
                                Pos2::new(new_x, new_y),
                                Vec2::new(200.0, part_height),
                            );

                            // Check collision with other parts
                            let has_collision = module.parts.iter().any(|other| {
                                if other.id == *part_id {
                                    return false;
                                }
                                let other_height = 80.0
                                    + (other.inputs.len().max(other.outputs.len()) as f32) * 20.0;
                                let other_rect = Rect::from_min_size(
                                    Pos2::new(other.position.0, other.position.1),
                                    Vec2::new(200.0, other_height),
                                );
                                new_rect.intersects(other_rect)
                            });

                            // Only move if no collision
                            if !has_collision {
                                if let Some(part_mut) =
                                    module.parts.iter_mut().find(|p| p.id == *part_id)
                                {
                                    part_mut.position.0 = new_x;
                                    part_mut.position.1 = new_y;
                                }
                            }
                        }
                    }
                }
            }

            if part_response.drag_stopped() {
                self.dragging_part = None;
            }

            // Check for delete button click (× in top-right corner of title bar)
            let delete_button_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 20.0 * self.zoom, rect.min.y),
                Vec2::splat(20.0 * self.zoom),
            );
            let delete_response = ui.interact(
                delete_button_rect,
                egui::Id::new((*part_id, "delete")),
                Sense::click(),
            );
            if delete_response.clicked() {
                delete_part_id = Some(*part_id);
            }
        }

        // Process pending deletion
        if let Some(part_id) = delete_part_id {
            // Remove all connections involving this part
            module
                .connections
                .retain(|c| c.from_part != part_id && c.to_part != part_id);
            // Remove the part
            module.parts.retain(|p| p.id != part_id);
        }

        // Resize operations to apply after the loop
        let mut resize_ops = Vec::new();

        // Draw parts (nodes) with delete buttons and selection highlight
        for part in &module.parts {
            let part_screen_pos = to_screen(Pos2::new(part.position.0, part.position.1));

            // Use custom size or calculate default
            let (part_width, part_height) = part.size.unwrap_or_else(|| {
                let default_height =
                    80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                (200.0, default_height)
            });
            let part_size = Vec2::new(part_width, part_height);
            let part_screen_rect = Rect::from_min_size(part_screen_pos, part_size * self.zoom);

            // Draw selection highlight if selected
            if self.selected_parts.contains(&part.id) {
                let highlight_rect = part_screen_rect.expand(4.0 * self.zoom);
                painter.rect_stroke(
                    highlight_rect,
                    8.0 * self.zoom,
                    Stroke::new(3.0 * self.zoom, Color32::from_rgb(100, 200, 255)),
                );

                // Draw resize handle at bottom-right corner
                let handle_size = 12.0 * self.zoom;
                let handle_rect = Rect::from_min_size(
                    Pos2::new(
                        part_screen_rect.max.x - handle_size,
                        part_screen_rect.max.y - handle_size,
                    ),
                    Vec2::splat(handle_size),
                );
                painter.rect_filled(handle_rect, 2.0, Color32::from_rgb(100, 200, 255));
                // Draw diagonal lines for resize indicator
                painter.line_segment(
                    [
                        handle_rect.min + Vec2::new(3.0, handle_size - 3.0),
                        handle_rect.min + Vec2::new(handle_size - 3.0, 3.0),
                    ],
                    Stroke::new(1.5, Color32::from_gray(40)),
                );

                // Handle resize drag interaction
                let resize_response = ui.interact(
                    handle_rect,
                    egui::Id::new((part.id, "resize")),
                    Sense::drag(),
                );

                if resize_response.drag_started() {
                    self.resizing_part = Some((part.id, (part_width, part_height)));
                }

                if resize_response.dragged() {
                    if let Some((id, _original_size)) = self.resizing_part {
                        if id == part.id {
                            let delta = resize_response.drag_delta() / self.zoom;
                            resize_ops.push((part.id, delta));
                        }
                    }
                }

                if resize_response.drag_stopped() {
                    self.resizing_part = None;
                }
            }

            self.draw_part_with_delete(&painter, part, part_screen_rect);
        }

        // Apply resize operations
        for (part_id, delta) in resize_ops {
             if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                 // Initialize size if None
                 let current_size = part.size.unwrap_or_else(|| {
                     let h = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                     (200.0, h)
                 });
                 let new_w = (current_size.0 + delta.x).max(100.0);
                 let new_h = (current_size.1 + delta.y).max(50.0);
                 part.size = Some((new_w, new_h));
             }
        }

        // Draw connection being created with visual feedback
        if let Some((from_part_id, _from_socket_idx, from_is_output, ref from_type, start_pos)) =
            self.creating_connection.clone()
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // Check if hovering over a compatible socket
                let socket_radius = 8.0 * self.zoom;
                let mut is_valid_target = false;
                let mut near_socket = false;

                for socket in &all_sockets {
                    if socket.position.distance(pointer_pos) < socket_radius * 2.0 {
                        near_socket = true;
                        // Valid if: different part, opposite direction, same type
                        if socket.part_id != from_part_id
                            && socket.is_output != from_is_output
                            && socket.socket_type == *from_type
                        {
                            is_valid_target = true;
                        }
                        break;
                    }
                }

                // Color based on validity
                let color = if near_socket {
                    if is_valid_target {
                        Color32::from_rgb(50, 255, 100) // Green = valid
                    } else {
                        Color32::from_rgb(255, 80, 80) // Red = invalid
                    }
                } else {
                    Self::get_socket_color(from_type) // Default socket color
                };

                // Draw the connection line
                painter.line_segment([start_pos, pointer_pos], Stroke::new(3.0, color));

                // Draw a circle at the end point
                painter.circle_filled(pointer_pos, 5.0, color);
            }
        }

        // Draw mini-map in bottom-right corner
        self.draw_mini_map(&painter, canvas_rect, module);

        // Draw search popup if visible
        if self.show_search {
            self.draw_search_popup(ui, canvas_rect, module);
        }

        // Draw presets popup if visible
        if self.show_presets {
            self.draw_presets_popup(ui, canvas_rect, module);
        }
    }

    fn draw_search_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Search popup in top-center
        let popup_width = 300.0;
        let popup_height = 200.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            8.0,
            Color32::from_rgba_unmultiplied(30, 30, 40, 240),
        );
        painter.rect_stroke(
            popup_rect,
            8.0,
            Stroke::new(2.0, Color32::from_rgb(80, 120, 200)),
        );

        // Popup content
        let inner_rect = popup_rect.shrink(10.0);
        ui.allocate_ui_at_rect(inner_rect, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.search_filter);
                });
                ui.add_space(8.0);

                // Filter and show matching nodes
                let filter_lower = self.search_filter.to_lowercase();
                let matching_parts: Vec<_> = module
                    .parts
                    .iter()
                    .filter(|p| {
                        if filter_lower.is_empty() {
                            return true;
                        }
                        let name = Self::get_part_property_text(&p.part_type).to_lowercase();
                        let (_, _, _, type_name) = Self::get_part_style(&p.part_type);
                        name.contains(&filter_lower)
                            || type_name.to_lowercase().contains(&filter_lower)
                    })
                    .take(6)
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for part in matching_parts {
                            let (_, _, icon, type_name) = Self::get_part_style(&part.part_type);
                            let label = format!(
                                "{} {} - {}",
                                icon,
                                type_name,
                                Self::get_part_property_text(&part.part_type)
                            );
                            if ui
                                .selectable_label(self.selected_parts.contains(&part.id), &label)
                                .clicked()
                            {
                                self.selected_parts.clear();
                                self.selected_parts.push(part.id);
                                // Center view on selected node
                                self.pan_offset =
                                    Vec2::new(-part.position.0 + 200.0, -part.position.1 + 150.0);
                                self.show_search = false;
                            }
                        }
                    });
            });
        });
    }

    fn draw_presets_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Presets popup in top-center
        let popup_width = 280.0;
        let popup_height = 220.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            8.0,
            Color32::from_rgba_unmultiplied(30, 35, 45, 245),
        );
        painter.rect_stroke(
            popup_rect,
            8.0,
            Stroke::new(2.0, Color32::from_rgb(100, 180, 80)),
        );

        // Popup content
        let inner_rect = popup_rect.shrink(12.0);
        ui.allocate_ui_at_rect(inner_rect, |ui| {
            ui.vertical(|ui| {
                ui.heading("📋 Presets / Templates");
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        let presets = self.presets.clone();
                        for preset in &presets {
                            ui.horizontal(|ui| {
                                if ui.button(&preset.name).clicked() {
                                    // Clear current and load preset
                                    module.parts.clear();
                                    module.connections.clear();

                                    // Add parts from preset
                                    let mut part_ids = Vec::new();
                                    let mut next_id =
                                        module.parts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                                    for (part_type, position, size) in &preset.parts {
                                        let id = next_id;
                                        next_id += 1;

                                        let (inputs, outputs) =
                                            Self::get_sockets_for_part_type(part_type);

                                        module.parts.push(mapmap_core::module::ModulePart {
                                            id,
                                            part_type: part_type.clone(),
                                            position: *position,
                                            size: *size,
                                            inputs,
                                            outputs,
                                        });
                                        part_ids.push(id);
                                    }

                                    // Add connections
                                    for (from_idx, from_socket, to_idx, to_socket) in
                                        &preset.connections
                                    {
                                        if *from_idx < part_ids.len() && *to_idx < part_ids.len() {
                                            module.connections.push(
                                                mapmap_core::module::ModuleConnection {
                                                    from_part: part_ids[*from_idx],
                                                    from_socket: *from_socket,
                                                    to_part: part_ids[*to_idx],
                                                    to_socket: *to_socket,
                                                },
                                            );
                                        }
                                    }

                                    self.show_presets = false;
                                }
                                ui.label(format!("({} nodes)", preset.parts.len()));
                            });
                        }
                    });

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_presets = false;
                }
            });
        });
    }

    /// Get default sockets for a part type
    fn get_sockets_for_part_type(
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (
        Vec<mapmap_core::module::ModuleSocket>,
        Vec<mapmap_core::module::ModuleSocket>,
    ) {
        use mapmap_core::module::{ModulePartType, ModuleSocket, ModuleSocketType};

        match part_type {
            ModulePartType::Trigger(_) => (
                vec![],
                vec![ModuleSocket {
                    name: "Trigger Out".to_string(),
                    socket_type: ModuleSocketType::Trigger,
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
            ModulePartType::LayerAssignment(_) => (
                vec![ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
                vec![ModuleSocket {
                    name: "Layer Out".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Mesh(_) => (
                vec![ModuleSocket {
                    name: "Media In".to_string(),
                    socket_type: ModuleSocketType::Media,
                }],
                vec![ModuleSocket {
                    name: "Mesh Out".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
            ),
            ModulePartType::Output(_) => (
                vec![ModuleSocket {
                    name: "Layer In".to_string(),
                    socket_type: ModuleSocketType::Layer,
                }],
                vec![],
            ),
        }
    }

    fn draw_mini_map(&self, painter: &egui::Painter, canvas_rect: Rect, module: &MapFlowModule) {
        if module.parts.is_empty() {
            return;
        }

        // Mini-map size and position
        let map_size = Vec2::new(150.0, 100.0);
        let map_margin = 10.0;
        let map_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.max.x - map_size.x - map_margin,
                canvas_rect.max.y - map_size.y - map_margin,
            ),
            map_size,
        );

        // Background
        painter.rect_filled(
            map_rect,
            4.0,
            Color32::from_rgba_unmultiplied(30, 30, 40, 200),
        );
        painter.rect_stroke(map_rect, 4.0, Stroke::new(1.0, Color32::from_gray(80)));

        // Calculate bounds of all parts
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for part in &module.parts {
            let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            min_x = min_x.min(part.position.0);
            min_y = min_y.min(part.position.1);
            max_x = max_x.max(part.position.0 + 200.0);
            max_y = max_y.max(part.position.1 + height);
        }

        // Add padding
        let padding = 50.0;
        min_x -= padding;
        min_y -= padding;
        max_x += padding;
        max_y += padding;

        let world_width = (max_x - min_x).max(1.0);
        let world_height = (max_y - min_y).max(1.0);

        // Scale to fit in mini-map
        let scale_x = (map_size.x - 8.0) / world_width;
        let scale_y = (map_size.y - 8.0) / world_height;
        let scale = scale_x.min(scale_y);

        let to_map = |pos: Pos2| -> Pos2 {
            Pos2::new(
                map_rect.min.x + 4.0 + (pos.x - min_x) * scale,
                map_rect.min.y + 4.0 + (pos.y - min_y) * scale,
            )
        };

        // Draw parts as small rectangles
        for part in &module.parts {
            let height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
            let part_min = to_map(Pos2::new(part.position.0, part.position.1));
            let part_max = to_map(Pos2::new(part.position.0 + 200.0, part.position.1 + height));
            let part_rect = Rect::from_min_max(part_min, part_max);

            let (_, title_color, _, _) = Self::get_part_style(&part.part_type);
            painter.rect_filled(part_rect, 1.0, title_color);
        }

        // Draw viewport rectangle
        let viewport_min = to_map(Pos2::new(
            -self.pan_offset.x / self.zoom,
            -self.pan_offset.y / self.zoom,
        ));
        let viewport_max = to_map(Pos2::new(
            (-self.pan_offset.x + canvas_rect.width()) / self.zoom,
            (-self.pan_offset.y + canvas_rect.height()) / self.zoom,
        ));
        let viewport_rect = Rect::from_min_max(viewport_min, viewport_max).intersect(map_rect);
        painter.rect_stroke(viewport_rect, 0.0, Stroke::new(1.5, Color32::WHITE));
    }

    #[allow(dead_code)]
    fn render_node_inspector(ui: &mut Ui, part: &mut mapmap_core::module::ModulePart) {
        use mapmap_core::module::{
            BlendModeType, EffectType, LayerAssignmentType, MaskShape, MaskType, ModulePartType,
            ModulizerType, OutputType, SourceType, TriggerType,
        };

        let (_, _, icon, type_name) = Self::get_part_style(&part.part_type);
        ui.label(format!("{} {} Node", icon, type_name));
        ui.add_space(8.0);

        match &mut part.part_type {
            ModulePartType::Trigger(trigger_type) => {
                ui.label("Trigger Type:");
                let current = match trigger_type {
                    TriggerType::Beat => "Beat",
                    TriggerType::AudioFFT { .. } => "Audio FFT",
                    TriggerType::Random { .. } => "Random",
                    TriggerType::Fixed { .. } => "Fixed Timer",
                    TriggerType::Midi { .. } => "MIDI",
                    TriggerType::Osc { .. } => "OSC",
                    TriggerType::Shortcut { .. } => "Shortcut",
                };
                egui::ComboBox::from_id_source("trigger_type")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(matches!(trigger_type, TriggerType::Beat), "Beat")
                            .clicked()
                        {
                            *trigger_type = TriggerType::Beat;
                        }
                        if ui
                            .selectable_label(
                                matches!(trigger_type, TriggerType::AudioFFT { .. }),
                                "Audio FFT",
                            )
                            .clicked()
                        {
                            *trigger_type = TriggerType::AudioFFT {
                                band: mapmap_core::module::AudioBand::Bass,
                                threshold: 0.5,
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(trigger_type, TriggerType::Random { .. }),
                                "Random",
                            )
                            .clicked()
                        {
                            *trigger_type = TriggerType::Random {
                                min_interval_ms: 500,
                                max_interval_ms: 2000,
                                probability: 0.5,
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(trigger_type, TriggerType::Fixed { .. }),
                                "Fixed Timer",
                            )
                            .clicked()
                        {
                            *trigger_type = TriggerType::Fixed {
                                interval_ms: 1000,
                                offset_ms: 0,
                            };
                        }
                    });
            }
            ModulePartType::Source(source_type) => {
                ui.label("Source Type:");
                let current = match source_type {
                    SourceType::MediaFile { .. } => "Media File",
                    SourceType::Shader { .. } => "Shader",
                    SourceType::LiveInput { .. } => "Live Input",
                };
                egui::ComboBox::from_id_source("source_type")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(source_type, SourceType::MediaFile { .. }),
                                "Media File",
                            )
                            .clicked()
                        {
                            *source_type = SourceType::MediaFile {
                                path: String::new(),
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(source_type, SourceType::Shader { .. }),
                                "Shader",
                            )
                            .clicked()
                        {
                            *source_type = SourceType::Shader {
                                name: "Default".to_string(),
                                params: vec![],
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(source_type, SourceType::LiveInput { .. }),
                                "Live Input",
                            )
                            .clicked()
                        {
                            *source_type = SourceType::LiveInput { device_id: 0 };
                        }
                    });

                // Properties for SourceType
                if let SourceType::MediaFile { path } = source_type {
                    ui.add_space(4.0);
                    ui.label("Media Path:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(path);
                        if ui.button("📂").on_hover_text("Select File").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                                *path = file_path.to_string_lossy().to_string();
                            }
                        }
                    });
                }
            }
            ModulePartType::Mask(mask_type) => {
                ui.label("Mask Type:");
                let current = match mask_type {
                    MaskType::File { .. } => "File",
                    MaskType::Shape(_) => "Shape",
                    MaskType::Gradient { .. } => "Gradient",
                };
                egui::ComboBox::from_id_source("mask_type")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(matches!(mask_type, MaskType::File { .. }), "File")
                            .clicked()
                        {
                            *mask_type = MaskType::File {
                                path: String::new(),
                            };
                        }
                        if ui
                            .selectable_label(matches!(mask_type, MaskType::Shape(_)), "Shape")
                            .clicked()
                        {
                            *mask_type = MaskType::Shape(MaskShape::Rectangle);
                        }
                        if ui
                            .selectable_label(
                                matches!(mask_type, MaskType::Gradient { .. }),
                                "Gradient",
                            )
                            .clicked()
                        {
                            *mask_type = MaskType::Gradient {
                                angle: 0.0,
                                softness: 0.5,
                            };
                        }
                    });

                // Properties for MaskType
                if let MaskType::File { path } = mask_type {
                    ui.add_space(4.0);
                    ui.label("Mask Image Path:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(path);
                        if ui.button("📂").on_hover_text("Select File").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                                *path = file_path.to_string_lossy().to_string();
                            }
                        }
                    });
                }

                // Shape sub-selector
                if let MaskType::Shape(shape) = mask_type {
                    ui.add_space(4.0);
                    ui.label("Shape:");
                    egui::ComboBox::from_id_source("shape_type")
                        .selected_text(format!("{:?}", shape))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(matches!(shape, MaskShape::Circle), "Circle")
                                .clicked()
                            {
                                *shape = MaskShape::Circle;
                            }
                            if ui
                                .selectable_label(
                                    matches!(shape, MaskShape::Rectangle),
                                    "Rectangle",
                                )
                                .clicked()
                            {
                                *shape = MaskShape::Rectangle;
                            }
                            if ui
                                .selectable_label(matches!(shape, MaskShape::Triangle), "Triangle")
                                .clicked()
                            {
                                *shape = MaskShape::Triangle;
                            }
                            if ui
                                .selectable_label(matches!(shape, MaskShape::Star), "Star")
                                .clicked()
                            {
                                *shape = MaskShape::Star;
                            }
                            if ui
                                .selectable_label(matches!(shape, MaskShape::Ellipse), "Ellipse")
                                .clicked()
                            {
                                *shape = MaskShape::Ellipse;
                            }
                        });
                }
            }
            ModulePartType::Modulizer(modulizer_type) => {
                ui.label("Modulator Type:");
                let current = match modulizer_type {
                    ModulizerType::Effect(_) => "Effect",
                    ModulizerType::BlendMode(_) => "Blend Mode",
                    ModulizerType::AudioReactive { .. } => "Audio Reactive",
                };
                egui::ComboBox::from_id_source("modulator_type")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(modulizer_type, ModulizerType::Effect(_)),
                                "Effect",
                            )
                            .clicked()
                        {
                            *modulizer_type = ModulizerType::Effect(EffectType::Blur);
                        }
                        if ui
                            .selectable_label(
                                matches!(modulizer_type, ModulizerType::BlendMode(_)),
                                "Blend Mode",
                            )
                            .clicked()
                        {
                            *modulizer_type = ModulizerType::BlendMode(BlendModeType::Normal);
                        }
                    });

                // Effect sub-selector
                if let ModulizerType::Effect(effect) = modulizer_type {
                    ui.add_space(4.0);
                    ui.label("Effect:");
                    egui::ComboBox::from_id_source("effect_type")
                        .selected_text(effect.name())
                        .show_ui(ui, |ui| {
                            for e in EffectType::all() {
                                if ui.selectable_label(*effect == *e, e.name()).clicked() {
                                    *effect = *e;
                                }
                            }
                        });
                }

                // Blend mode sub-selector
                if let ModulizerType::BlendMode(blend) = modulizer_type {
                    ui.add_space(4.0);
                    ui.label("Blend Mode:");
                    egui::ComboBox::from_id_source("blend_type")
                        .selected_text(blend.name())
                        .show_ui(ui, |ui| {
                            for b in BlendModeType::all() {
                                if ui.selectable_label(*blend == *b, b.name()).clicked() {
                                    *blend = *b;
                                }
                            }
                        });
                }
            }
            ModulePartType::LayerAssignment(layer_type) => {
                ui.label("Layer Type:");
                let current_type_name = match layer_type {
                    LayerAssignmentType::SingleLayer { .. } => "Single Layer",
                    LayerAssignmentType::Group { .. } => "Group",
                    LayerAssignmentType::AllLayers { .. } => "All Layers",
                };

                // Type Selector
                egui::ComboBox::from_id_source("layer_type")
                    .selected_text(current_type_name)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(layer_type, LayerAssignmentType::SingleLayer { .. }),
                                "Single Layer",
                            )
                            .clicked()
                        {
                            *layer_type = LayerAssignmentType::SingleLayer {
                                id: 0,
                                name: "Layer 1".to_string(),
                                opacity: 1.0,
                                blend_mode: None,
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(layer_type, LayerAssignmentType::Group { .. }),
                                "Group",
                            )
                            .clicked()
                        {
                            *layer_type = LayerAssignmentType::Group {
                                name: "Group 1".to_string(),
                                opacity: 1.0,
                                blend_mode: None,
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(layer_type, LayerAssignmentType::AllLayers { .. }),
                                "All Layers",
                            )
                            .clicked()
                        {
                            *layer_type = LayerAssignmentType::AllLayers {
                                opacity: 1.0,
                                blend_mode: None,
                            };
                        }
                    });

                ui.separator();

                // Common Properties access
                let (opacity, blend_mode) = match layer_type {
                    LayerAssignmentType::SingleLayer {
                        opacity,
                        blend_mode,
                        ..
                    } => (opacity, blend_mode),
                    LayerAssignmentType::Group {
                        opacity,
                        blend_mode,
                        ..
                    } => (opacity, blend_mode),
                    LayerAssignmentType::AllLayers {
                        opacity,
                        blend_mode,
                    } => (opacity, blend_mode),
                };

                // Opacity Slider
                ui.label("Opacity:");
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Value"));

                // Blend Mode Selector
                ui.label("Blend Mode:");
                let current_blend = blend_mode.map(|b| b.name()).unwrap_or("Keep Original");
                egui::ComboBox::from_id_source("layer_blend")
                    .selected_text(current_blend)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(blend_mode.is_none(), "Keep Original")
                            .clicked()
                        {
                            *blend_mode = None;
                        }
                        ui.separator();
                        for b in BlendModeType::all() {
                            if ui
                                .selectable_label(
                                    blend_mode.as_ref().map_or(false, |current| *current == *b),
                                    b.name(),
                                )
                                .clicked()
                            {
                                *blend_mode = Some(*b);
                            }
                        }
                    });
            }
            ModulePartType::Mesh(_) => {
                ui.label("Mesh Type:");
                ui.label("Configure mesh in Node Control panel.");
            }
            ModulePartType::Output(output_type) => {
                ui.label("Output Type:");
                let current = match output_type {
                    OutputType::Projector { .. } => "Projector",
                    OutputType::Preview { .. } => "Preview",
                };
                egui::ComboBox::from_id_source("output_type")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(output_type, OutputType::Projector { .. }),
                                "Projector",
                            )
                            .clicked()
                        {
                            *output_type = OutputType::Projector {
                                id: 0,
                                name: "Projector 1".to_string(),
                            };
                        }
                        if ui
                            .selectable_label(
                                matches!(output_type, OutputType::Preview { .. }),
                                "Preview",
                            )
                            .clicked()
                        {
                            *output_type = OutputType::Preview { window_id: 0 };
                        }
                    });
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.label(format!(
            "Position: ({:.0}, {:.0})",
            part.position.0, part.position.1
        ));
        ui.label(format!("Inputs: {}", part.inputs.len()));
        ui.label(format!("Outputs: {}", part.outputs.len()));
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let color = Color32::from_rgb(40, 40, 40);
        let mut x = rect.left() - self.pan_offset.x % grid_size;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
            x += grid_size;
        }
        let mut y = rect.top() - self.pan_offset.y % grid_size;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, color),
            );
            y += grid_size;
        }
    }

    fn draw_connections<F>(&self, painter: &egui::Painter, module: &MapFlowModule, to_screen: &F)
    where
        F: Fn(Pos2) -> Pos2,
    {
        let node_width = 200.0;
        let title_height = 28.0;
        let socket_offset_y = 10.0;
        let socket_spacing = 22.0;
        // let socket_radius = 6.0; // Not used directly, we draw plugs from center

        for conn in &module.connections {
            // Find source and target parts
            let from_part = module.parts.iter().find(|p| p.id == conn.from_part);
            let to_part = module.parts.iter().find(|p| p.id == conn.to_part);

            if let (Some(from), Some(to)) = (from_part, to_part) {
                // Determine cable color based on socket type
                let socket_type = if let Some(socket) = from.outputs.get(conn.from_socket) {
                    &socket.socket_type
                } else if let Some(socket) = to.inputs.get(conn.to_socket) {
                     &socket.socket_type
                } else {
                    &mapmap_core::module::ModuleSocketType::Media // Fallback
                };
                let cable_color = Self::get_socket_color(socket_type);
                let shadow_color = Color32::from_black_alpha(100);

                // Calculate WORLD positions
                // Output: Right side + center of socket height
                let from_local_y = title_height + socket_offset_y + conn.from_socket as f32 * socket_spacing + socket_spacing / 2.0;
                let from_socket_world = Pos2::new(
                    from.position.0 + node_width,
                    from.position.1 + from_local_y,
                );
                
                // Input: Left side + center of socket height
                let to_local_y = title_height + socket_offset_y + conn.to_socket as f32 * socket_spacing + socket_spacing / 2.0;
                let to_socket_world = Pos2::new(
                    to.position.0,
                    to.position.1 + to_local_y,
                );

                // Convert to SCREEN positions
                let start_pos = to_screen(from_socket_world);
                let end_pos = to_screen(to_socket_world);

                // Draw Plugs
                let plug_len = 15.0 * self.zoom;
                let plug_width = 8.0 * self.zoom;
                
                // Source Plug (Right facing)
                let start_plug_rect = Rect::from_min_max(
                    start_pos,
                    Pos2::new(start_pos.x + plug_len, start_pos.y + plug_width / 2.0)
                ).translate(Vec2::new(0.0, -plug_width / 2.0));
                
                painter.rect_filled(start_plug_rect, 2.0, cable_color);
                painter.rect_stroke(start_plug_rect, 2.0, Stroke::new(1.0, Color32::BLACK));

                // Target Plug (Left facing)
                let end_plug_rect = Rect::from_min_max(
                    Pos2::new(end_pos.x - plug_len, end_pos.y - plug_width / 2.0),
                    Pos2::new(end_pos.x, end_pos.y + plug_width / 2.0)
                );
                
                painter.rect_filled(end_plug_rect, 2.0, cable_color);
                painter.rect_stroke(end_plug_rect, 2.0, Stroke::new(1.0, Color32::BLACK));

                // Draw Cable (Bezier)
                // Start cable from end of plugs
                let cable_start = Pos2::new(start_pos.x + plug_len, start_pos.y);
                let cable_end = Pos2::new(end_pos.x - plug_len, end_pos.y);

                let control_offset = (cable_end.x - cable_start.x).abs() * 0.5;
                let control_offset = control_offset.max(50.0 * self.zoom); // Minimum curve

                let ctrl1 = Pos2::new(cable_start.x + control_offset, cable_start.y);
                let ctrl2 = Pos2::new(cable_end.x - control_offset, cable_end.y);

                // Draw shadow/outline first
                 let steps = 40;
                for i in 0..steps {
                    let t1 = i as f32 / steps as f32;
                    let t2 = (i + 1) as f32 / steps as f32;
                    let p1 = Self::bezier_point(cable_start, ctrl1, ctrl2, cable_end, t1);
                    let p2 = Self::bezier_point(cable_start, ctrl1, ctrl2, cable_end, t2);
                    // Shadow/Outline
                    painter.line_segment([p1, p2], Stroke::new(5.0 * self.zoom, shadow_color));
                    // Inner Cable
                    painter.line_segment([p1, p2], Stroke::new(3.0 * self.zoom, cable_color));
                }
            }
        }
    }

    fn bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        Pos2::new(
            uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
            uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
        )
    }

    fn draw_part_with_delete(&self, painter: &egui::Painter, part: &ModulePart, rect: Rect) {
        // Get part color and name based on type
        let (bg_color, title_color, icon, name) = Self::get_part_style(&part.part_type);

        // Draw background
        painter.rect_filled(rect, 6.0 * self.zoom, bg_color);
        painter.rect_stroke(
            rect,
            6.0 * self.zoom,
            Stroke::new(2.0, Color32::from_rgb(80, 80, 90)),
        );

        // Title bar
        let title_height = 28.0 * self.zoom;
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));
        painter.rect_filled(
            title_rect,
            egui::Rounding {
                nw: 6.0 * self.zoom,
                ne: 6.0 * self.zoom,
                sw: 0.0,
                se: 0.0,
            },
            title_color,
        );

        // Title text with icon (offset slightly left to make room for × button)
        let title_text = format!("{} {}", icon, name);
        painter.text(
            Pos2::new(
                title_rect.center().x - 8.0 * self.zoom,
                title_rect.center().y,
            ),
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(13.0 * self.zoom),
            Color32::WHITE,
        );

        // Delete button (× in top-right corner)
        let delete_button_pos = Pos2::new(
            rect.max.x - 12.0 * self.zoom,
            rect.min.y + title_height * 0.5,
        );
        painter.text(
            delete_button_pos,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::from_rgba_unmultiplied(255, 100, 100, 200),
        );

        // Draw property display based on part type
        let property_y = rect.min.y + title_height + 8.0 * self.zoom;
        let property_text = Self::get_part_property_text(&part.part_type);
        if !property_text.is_empty() {
            painter.text(
                Pos2::new(rect.center().x, property_y),
                egui::Align2::CENTER_TOP,
                property_text,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(160),
            );
        }

        // Draw input sockets (left side)
        let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;
        for (i, socket) in part.inputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.min.x, socket_y);
            let socket_radius = 6.0 * self.zoom;

            // Socket circle
            let socket_color = Self::get_socket_color(&socket.socket_type);
            painter.circle_filled(socket_pos, socket_radius, socket_color);
            painter.circle_stroke(socket_pos, socket_radius, Stroke::new(1.5, Color32::WHITE));

            // Socket label
            painter.text(
                Pos2::new(rect.min.x + 12.0 * self.zoom, socket_y),
                egui::Align2::LEFT_CENTER,
                &socket.name,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(200),
            );
        }

        // Draw output sockets (right side)
        for (i, socket) in part.outputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.max.x, socket_y);
            let socket_radius = 6.0 * self.zoom;

            // Socket circle
            let socket_color = Self::get_socket_color(&socket.socket_type);
            painter.circle_filled(socket_pos, socket_radius, socket_color);
            painter.circle_stroke(socket_pos, socket_radius, Stroke::new(1.5, Color32::WHITE));

            // Socket label
            painter.text(
                Pos2::new(rect.max.x - 12.0 * self.zoom, socket_y),
                egui::Align2::RIGHT_CENTER,
                &socket.name,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(200),
            );
        }
    }

    fn get_part_style(
        part_type: &mapmap_core::module::ModulePartType,
    ) -> (Color32, Color32, &'static str, &'static str) {
        use mapmap_core::module::{ModulePartType, TriggerType, SourceType, MaskType, MaskShape, ModulizerType, EffectType, BlendModeType, LayerAssignmentType, OutputType};
        match part_type {
            ModulePartType::Trigger(trigger) => {
                let name = match trigger {
                    TriggerType::AudioFFT { .. } => "Audio FFT",
                    TriggerType::Beat => "Beat",
                    TriggerType::Midi { .. } => "MIDI",
                    TriggerType::Osc { .. } => "OSC",
                    TriggerType::Shortcut { .. } => "Shortcut",
                    TriggerType::Random { .. } => "Random",
                    TriggerType::Fixed { .. } => "Fixed Timer",
                };
                (Color32::from_rgb(60, 50, 70), Color32::from_rgb(130, 80, 180), "⚡", name)
            },
            ModulePartType::Source(source) => {
                let name = match source {
                    SourceType::MediaFile { .. } => "Media File",
                    SourceType::Shader { .. } => "Shader",
                    SourceType::LiveInput { .. } => "Live Input",
                };
                (Color32::from_rgb(50, 60, 70), Color32::from_rgb(80, 140, 180), "🎬", name)
            },
            ModulePartType::Mask(mask) => {
                let name = match mask {
                    MaskType::File { .. } => "File Mask",
                    MaskType::Shape(shape) => match shape {
                        MaskShape::Circle => "Circle",
                        MaskShape::Rectangle => "Rectangle",
                        MaskShape::Triangle => "Triangle",
                        MaskShape::Star => "Star",
                        MaskShape::Ellipse => "Ellipse",
                    },
                    MaskType::Gradient { .. } => "Gradient",
                };
                (Color32::from_rgb(60, 55, 70), Color32::from_rgb(160, 100, 180), "🎭", name)
            },
            ModulePartType::Modulizer(mod_type) => {
                let name = match mod_type {
                    ModulizerType::Effect(effect) => match effect {
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
                        EffectType::ChromaticAberration => "Chromatic",
                        EffectType::VHS => "VHS",
                        EffectType::FilmGrain => "Film Grain",
                    },
                    ModulizerType::BlendMode(blend) => match blend {
                        BlendModeType::Normal => "Normal",
                        BlendModeType::Add => "Add",
                        BlendModeType::Multiply => "Multiply",
                        BlendModeType::Screen => "Screen",
                        BlendModeType::Overlay => "Overlay",
                        BlendModeType::Difference => "Difference",
                        BlendModeType::Exclusion => "Exclusion",
                    },
                    ModulizerType::AudioReactive { .. } => "Audio Reactive",
                };
                (Color32::from_rgb(60, 60, 50), Color32::from_rgb(180, 140, 60), "〰️", name)
            },
            ModulePartType::LayerAssignment(layer) => {
                let name = match layer {
                    LayerAssignmentType::SingleLayer { .. } => "Single Layer",
                    LayerAssignmentType::Group { .. } => "Layer Group",
                    LayerAssignmentType::AllLayers { .. } => "All Layers",
                };
                (Color32::from_rgb(50, 70, 60), Color32::from_rgb(80, 180, 120), "📑", name)
            },
            ModulePartType::Mesh(mesh) => {
                let name = match mesh {
                    MeshType::Quad { .. } => "Quad",
                    MeshType::Grid { .. } => "Grid",
                    MeshType::BezierSurface { .. } => "Bezier",
                    MeshType::Polygon { .. } => "Polygon",
                    MeshType::TriMesh => "Triangle",
                    MeshType::Circle { .. } => "Circle",
                    MeshType::Cylinder { .. } => "Cylinder",
                    MeshType::Sphere { .. } => "Sphere",
                    MeshType::Custom { .. } => "Custom",
                };
                (Color32::from_rgb(55, 60, 70), Color32::from_rgb(100, 150, 200), "🔷", name)
            },
            ModulePartType::Output(output) => {
                let name = match output {
                    OutputType::Projector { .. } => "Projector",
                    OutputType::Preview { .. } => "Preview",
                };
                (Color32::from_rgb(70, 50, 50), Color32::from_rgb(180, 80, 80), "📺", name)
            },
        }
    }

    fn get_socket_color(socket_type: &mapmap_core::module::ModuleSocketType) -> Color32 {
        use mapmap_core::module::ModuleSocketType;
        match socket_type {
            ModuleSocketType::Trigger => Color32::from_rgb(180, 100, 220),
            ModuleSocketType::Media => Color32::from_rgb(100, 180, 220),
            ModuleSocketType::Effect => Color32::from_rgb(220, 180, 100),
            ModuleSocketType::Layer => Color32::from_rgb(100, 220, 140),
            ModuleSocketType::Output => Color32::from_rgb(220, 100, 100),
        }
    }

    fn get_part_property_text(part_type: &mapmap_core::module::ModulePartType) -> String {
        use mapmap_core::module::{
            MaskType, ModulePartType, ModulizerType, OutputType, SourceType, TriggerType,
        };
        match part_type {
            ModulePartType::Trigger(trigger_type) => match trigger_type {
                TriggerType::AudioFFT { band, .. } => format!("🔊 Audio: {:?}", band),
                TriggerType::Random { .. } => "🎲 Random".to_string(),
                TriggerType::Fixed { interval_ms, .. } => format!("⏱️ {}ms", interval_ms),
                TriggerType::Midi { channel, note } => format!("🎹 Ch{} N{}", channel, note),
                TriggerType::Osc { address } => format!("📡 {}", address),
                TriggerType::Shortcut { key_code, .. } => format!("⌨️ {}", key_code),
                TriggerType::Beat => "🥁 Beat".to_string(),
            },
            ModulePartType::Source(source_type) => match source_type {
                SourceType::MediaFile { path } => {
                    if path.is_empty() {
                        "📁 Select file...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                SourceType::Shader { name, .. } => format!("🎨 {}", name),
                SourceType::LiveInput { device_id } => format!("📹 Device {}", device_id),
            },
            ModulePartType::Mask(mask_type) => match mask_type {
                MaskType::File { path } => {
                    if path.is_empty() {
                        "📁 Select mask...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                MaskType::Shape(shape) => format!("🔷 {:?}", shape),
                MaskType::Gradient { angle, .. } => format!("🌈 Gradient {}°", *angle as i32),
            },
            ModulePartType::Modulizer(modulizer_type) => match modulizer_type {
                ModulizerType::Effect(effect) => format!("✨ {}", effect.name()),
                ModulizerType::BlendMode(blend) => format!("🔀 {}", blend.name()),
                ModulizerType::AudioReactive { source } => format!("🔊 {}", source),
            },
            ModulePartType::LayerAssignment(layer_type) => {
                use mapmap_core::module::LayerAssignmentType;
                match layer_type {
                    LayerAssignmentType::SingleLayer { name, .. } => format!("📑 {}", name),
                    LayerAssignmentType::Group { name, .. } => format!("📁 {}", name),
                    LayerAssignmentType::AllLayers { .. } => "📑 All Layers".to_string(),
                }
            }
            ModulePartType::Mesh(mesh_type) => {
                use mapmap_core::module::MeshType;
                match mesh_type {
                    MeshType::Quad { .. } => "⬜ Quad".to_string(),
                    MeshType::Grid { rows, cols } => format!("▦ Grid {}x{}", rows, cols),
                    MeshType::BezierSurface { .. } => "〰️ Bezier".to_string(),
                    MeshType::Polygon { .. } => "⬡ Polygon".to_string(),
                    MeshType::TriMesh => "🔺 Triangle".to_string(),
                    MeshType::Circle { segments, .. } => format!("⭕ Circle ({})", segments),
                    MeshType::Cylinder { segments, .. } => format!("🌐 Cylinder ({})", segments),
                    MeshType::Sphere { .. } => "🌍 Sphere".to_string(),
                    MeshType::Custom { path } => {
                        if path.is_empty() {
                            "📁 Custom...".to_string()
                        } else {
                            format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                        }
                    }
                }
            }
            ModulePartType::Output(output_type) => match output_type {
                OutputType::Projector { name, .. } => format!("📺 {}", name),
                OutputType::Preview { window_id } => format!("👁 Preview {}", window_id),
            },
        }
    }

    /// Convert ModulePartType back to PartType for add_part
    fn part_type_from_module_part_type(
        mpt: &mapmap_core::module::ModulePartType,
    ) -> mapmap_core::module::PartType {
        use mapmap_core::module::{ModulePartType, PartType};
        match mpt {
            ModulePartType::Trigger(_) => PartType::Trigger,
            ModulePartType::Source(_) => PartType::Source,
            ModulePartType::Mask(_) => PartType::Mask,
            ModulePartType::Modulizer(_) => PartType::Modulator,
            ModulePartType::Mesh(_) => PartType::Mesh,
            ModulePartType::LayerAssignment(_) => PartType::Layer,
            ModulePartType::Output(_) => PartType::Output,
        }
    }

    /// Auto-layout parts in a grid by type (left to right: Trigger → Source → Mask → Modulator → Layer → Output)
    fn auto_layout_parts(parts: &mut [mapmap_core::module::ModulePart]) {
        use mapmap_core::module::ModulePartType;

        // Sort parts by type category for left-to-right flow
        let type_order = |pt: &ModulePartType| -> usize {
            match pt {
                ModulePartType::Trigger(_) => 0,
                ModulePartType::Source(_) => 1,
                ModulePartType::Mask(_) => 2,
                ModulePartType::Modulizer(_) => 3,
                ModulePartType::Mesh(_) => 4,
                ModulePartType::LayerAssignment(_) => 5,
                ModulePartType::Output(_) => 6,
            }
        };

        // Group parts by type
        let mut columns: [Vec<usize>; 7] = Default::default();
        for (i, part) in parts.iter().enumerate() {
            let col = type_order(&part.part_type);
            columns[col].push(i);
        }

        // Layout parameters
        let node_width = 200.0;
        let node_height = 120.0;
        let h_spacing = 50.0;
        let v_spacing = 30.0;
        let start_x = 50.0;
        let start_y = 50.0;

        // Position each column
        let mut x = start_x;
        for col in &columns {
            if col.is_empty() {
                continue;
            }

            let mut y = start_y;
            for &part_idx in col {
                parts[part_idx].position = (x, y);
                y += node_height + v_spacing;
            }

            x += node_width + h_spacing;
        }
    }

    /// Find a free position for a new node, avoiding overlaps with existing nodes
    fn find_free_position(
        parts: &[mapmap_core::module::ModulePart],
        preferred: (f32, f32),
    ) -> (f32, f32) {
        let node_width = 200.0;
        let node_height = 130.0;
        let grid_step = 30.0;

        let mut pos = preferred;
        let mut attempts = 0;

        loop {
            let new_rect =
                Rect::from_min_size(Pos2::new(pos.0, pos.1), Vec2::new(node_width, node_height));

            let has_collision = parts.iter().any(|part| {
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_rect = Rect::from_min_size(
                    Pos2::new(part.position.0, part.position.1),
                    Vec2::new(node_width, part_height),
                );
                new_rect.intersects(part_rect)
            });

            if !has_collision {
                return pos;
            }

            // Try different positions in a spiral pattern
            attempts += 1;
            if attempts > 100 {
                // Give up after 100 attempts, just offset significantly
                return (preferred.0, preferred.1 + (parts.len() as f32) * 150.0);
            }

            // Move down first, then right
            pos.1 += grid_step;
            if pos.1 > preferred.1 + 500.0 {
                pos.1 = preferred.1;
                pos.0 += node_width + 20.0;
            }
        }
    }

    /// Create default presets/templates
    fn default_presets() -> Vec<ModulePreset> {
        use mapmap_core::module::*;

        vec![
            ModulePreset {
                name: "Simple Media Chain".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::MediaFile {
                            path: String::new(),
                        }),
                        (250.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 0,
                            name: "Projector 1".to_string(),
                        }),
                        (450.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                ],
            },
            ModulePreset {
                name: "Effect Chain".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::MediaFile {
                            path: String::new(),
                        }),
                        (250.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect(EffectType::Blur)),
                        (450.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 0,
                            name: "Projector 1".to_string(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Effect
                ],
            },
            ModulePreset {
                name: "Audio Reactive".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::AudioFFT {
                            band: AudioBand::Bass,
                            threshold: 0.5,
                        }),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::MediaFile {
                            path: String::new(),
                        }),
                        (250.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Modulizer(ModulizerType::Effect(EffectType::Glitch)),
                        (450.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::LayerAssignment(LayerAssignmentType::AllLayers {
                            opacity: 1.0,
                            blend_mode: None,
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 0,
                            name: "Projector 1".to_string(),
                        }),
                        (850.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Audio -> Source
                    (1, 0, 2, 0), // Source -> Effect
                    (2, 0, 3, 0), // Effect -> Layer
                ],
            },
            ModulePreset {
                name: "Masked Media".to_string(),
                parts: vec![
                    (
                        ModulePartType::Trigger(TriggerType::Beat),
                        (50.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Source(SourceType::MediaFile {
                            path: String::new(),
                        }),
                        (250.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Mask(MaskType::Shape(MaskShape::Circle)),
                        (450.0, 100.0),
                        None,
                    ),
                    (
                        ModulePartType::Output(OutputType::Projector {
                            id: 0,
                            name: "Projector 1".to_string(),
                        }),
                        (650.0, 100.0),
                        None,
                    ),
                ],
                connections: vec![
                    (0, 0, 1, 0), // Trigger -> Source
                    (1, 0, 2, 0), // Source -> Mask
                ],
            },
        ]
    }
}
