//! Controller Overlay Panel
//!
//! Visual representation of the Ecler NUO 4 (or other MIDI controllers)
//! with live state visualization and MIDI Learn functionality.

#[cfg(feature = "midi")]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[cfg(feature = "midi")]
use mapmap_control::midi::{
    ControllerElement, ControllerElements, ElementState, ElementStateManager, ElementType,
    MidiConfig, MidiLearnManager, MidiMessage,
};
use mapmap_control::target::ControlTarget;
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
fn get_mock_targets() -> Vec<ControlTarget> {
    let mut targets = vec![ControlTarget::MasterOpacity, ControlTarget::MasterBlackout];
    for i in 0..4 {
        targets.push(ControlTarget::LayerOpacity(i));
        targets.push(ControlTarget::LayerPosition(i));
        targets.push(ControlTarget::LayerScale(i));
        targets.push(ControlTarget::LayerRotation(i));
        targets.push(ControlTarget::LayerVisibility(i));
    }
    targets
}

/// Maximum size of the overlay (matches mixer photo resolution)
const MAX_WIDTH: f32 = 841.0;
const MAX_HEIGHT: f32 = 1024.0;
const MIN_SCALE: f32 = 0.3;

/// MIDI Learn target type
#[derive(Debug, Clone, PartialEq)]
pub enum MidiLearnTarget {
    MapFlow,
    StreamerBot(String), // Function name
    Mixxx(String),       // Function name
}

/// Controller Overlay Panel for visualizing MIDI controller state
pub struct ControllerOverlayPanel {
    /// Currently loaded controller elements
    #[cfg(feature = "midi")]
    elements: Option<ControllerElements>,

    /// Runtime state for each element
    #[cfg(feature = "midi")]
    state_manager: ElementStateManager,

    /// MIDI Learn manager
    #[cfg(feature = "midi")]
    learn_manager: MidiLearnManager,

    /// Current MIDI learn target type
    pub learn_target: Option<MidiLearnTarget>,

    /// Last active element from MIDI input (for global learn Way 1)
    pub last_active_element: Option<String>,
    pub last_active_time: Option<std::time::Instant>,

    /// Input field for Streamer.bot function
    streamerbot_function: String,

    /// Input field for Mixxx function
    mixxx_function: String,

    /// Show element labels
    show_labels: bool,

    /// Show element values
    show_values: bool,

    /// Show MIDI info on hover
    #[allow(dead_code)]
    show_midi_info: bool,

    /// Selected element for editing
    selected_element: Option<String>,

    /// Hovered element
    hovered_element: Option<String>,

    /// Panel is expanded
    pub is_expanded: bool,

    /// Current scale factor (0.3 - 1.0)
    scale: f32,

    /// Background texture
    background_texture: Option<TextureHandle>,

    /// Show element list view
    show_element_list: bool,

    /// Filter for element list
    element_filter: ElementFilter,

    /// Show assignment colors mode (highlights all elements by their assignment type)
    show_assignment_colors: bool,

    /// Edit mode for moving elements
    pub is_edit_mode: bool,

    /// Clipboard for element size (width, height)
    clipboard_size: Option<[f32; 2]>,

    /// Loaded assets
    assets: HashMap<String, TextureHandle>,
}

/// Filter for element list view
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ElementFilter {
    #[default]
    All,
    MapFlow,
    StreamerBot,
    Mixxx,
    Unassigned,
}

impl Default for ControllerOverlayPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ControllerOverlayPanel {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "midi")]
            elements: None,
            #[cfg(feature = "midi")]
            state_manager: ElementStateManager::new(),
            #[cfg(feature = "midi")]
            learn_manager: MidiLearnManager::new(),
            learn_target: None,
            last_active_element: None,
            last_active_time: None,
            streamerbot_function: String::new(),
            mixxx_function: String::new(),

            show_labels: true,
            show_values: true,
            show_midi_info: true,
            selected_element: None,
            hovered_element: None,
            is_expanded: true,
            scale: 0.6, // Start at 60% size
            background_texture: None,
            show_element_list: false,
            element_filter: ElementFilter::All,
            show_assignment_colors: false,
            is_edit_mode: false,
            clipboard_size: None,
            assets: HashMap::new(),
        }
    }

    /// Load resources (background and assets)
    fn ensure_resources_loaded(&mut self, ctx: &egui::Context) {
        // Load Background
        if self.background_texture.is_none() {
            let bg_paths = [
                "resources/controllers/ecler_nuo4/background.png",
                "resources/controllers/ecler_nuo4/background.jpg",
                "../resources/controllers/ecler_nuo4/background.png",
                "../resources/controllers/ecler_nuo4/background.jpg",
                r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\background.png",
                r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\background.jpg",
            ];
            if let Some(tex) = self.load_texture_from_candidates(ctx, &bg_paths, "mixer_background")
            {
                self.background_texture = Some(tex);
                tracing::info!("Loaded mixer background");
            }
        }

        // Load Assets
        #[cfg(feature = "midi")]
        if let Some(elements) = &self.elements {
            let mut needed = HashSet::new();
            for el in &elements.elements {
                if let Some(asset) = &el.asset {
                    needed.insert(asset.clone());
                }
            }

            for asset_name in needed {
                if !self.assets.contains_key(&asset_name) {
                    let paths = [
                        format!("resources/controllers/ecler_nuo4/{}", asset_name),
                        format!("../resources/controllers/ecler_nuo4/{}", asset_name),
                        format!(
                            r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\{}",
                            asset_name
                        ),
                    ];
                    if let Some(tex) = self.load_texture_from_candidates(ctx, &paths, &asset_name) {
                        self.assets.insert(asset_name, tex);
                    }
                }
            }
        }
    }

    fn load_texture_from_candidates<S: AsRef<str>>(
        &self,
        ctx: &egui::Context,
        paths: &[S],
        name: &str,
    ) -> Option<TextureHandle> {
        for path_str in paths {
            let path = std::path::Path::new(path_str.as_ref());
            if path.exists() {
                if let Ok(image_data) = std::fs::read(path) {
                    if let Ok(img) = image::load_from_memory(&image_data) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        return Some(ctx.load_texture(
                            name,
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                }
            }
        }
        None
    }

    /// Helper to draw asset for an element
    #[cfg(feature = "midi")]
    fn draw_asset(
        painter: &egui::Painter,
        assets: &HashMap<String, TextureHandle>,
        scale: f32,
        elem_rect: Rect,
        element: &ControllerElement,
        value: f32,
    ) {
        if let Some(asset_name) = &element.asset {
            if let Some(texture) = assets.get(asset_name) {
                match element.element_type {
                    ElementType::Fader => {
                        let asset_size = texture.size_vec2() * scale;
                        let travel = elem_rect.height() - asset_size.y;
                        // Avoid negative travel
                        let travel = travel.max(0.0);
                        let y = elem_rect.max.y - asset_size.y - (value * travel);

                        let cap_rect = Rect::from_min_size(
                            Pos2::new(elem_rect.center().x - asset_size.x * 0.5, y),
                            asset_size,
                        );

                        painter.image(
                            texture.id(),
                            cap_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ElementType::Knob | ElementType::Encoder => {
                        let asset_size = texture.size_vec2() * scale;
                        let knob_rect = Rect::from_center_size(elem_rect.center(), asset_size);
                        painter.image(
                            texture.id(),
                            knob_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    /// Load controller elements from JSON
    #[cfg(feature = "midi")]
    pub fn load_elements(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let mut elements = ControllerElements::from_json(json)?;
        if elements.controller == "Ecler NUO 4" {
            self.expand_nuo4_elements(&mut elements);
        }
        self.elements = Some(elements);
        Ok(())
    }

    #[cfg(feature = "midi")]
    fn expand_nuo4_elements(&self, elements: &mut ControllerElements) {
        let target_ids = [
            "encoder_1",
            "encoder_2",
            "encoder_3",
            "encoder_4",
            "push_enc_1",
            "push_enc_2",
            "push_enc_3",
            "push_enc_4",
            "switch_1",
            "switch_2",
            "switch_3",
            "switch_4",
        ];

        let mut new_list = Vec::new();
        let mut variants = Vec::new();

        for el in &elements.elements {
            if target_ids.contains(&el.id.as_str()) {
                let base_id = el.id.clone();
                let base_label = el.label.clone();

                let is_encoder = base_id.starts_with("encoder");
                let is_switch = base_id.starts_with("switch");
                // Get index (1-4)
                let num_char = base_id.chars().last().unwrap();
                let index = num_char.to_digit(10).unwrap() as u8 - 1;

                struct Variant {
                    suffix: &'static str,
                    label: &'static str,
                    enc_cc_base: u8,
                    push_note_base: u8,
                }
                let variant_defs = [
                    Variant {
                        suffix: "a1",
                        label: "A/L1",
                        enc_cc_base: 20,
                        push_note_base: 0,
                    },
                    Variant {
                        suffix: "b1",
                        label: "B/L1",
                        enc_cc_base: 24,
                        push_note_base: 8,
                    },
                    Variant {
                        suffix: "a2",
                        label: "A/L2",
                        enc_cc_base: 28,
                        push_note_base: 16,
                    },
                    Variant {
                        suffix: "b2",
                        label: "B/L2",
                        enc_cc_base: 102,
                        push_note_base: 24,
                    },
                    Variant {
                        suffix: "a3",
                        label: "A/L3",
                        enc_cc_base: 106,
                        push_note_base: 32,
                    },
                    Variant {
                        suffix: "b3",
                        label: "B/L3",
                        enc_cc_base: 110,
                        push_note_base: 40,
                    },
                ];

                for v in variant_defs {
                    let mut var_el = el.clone();
                    var_el.id = format!("{}_{}", base_id, v.suffix);
                    var_el.label = format!("{} ({})", base_label, v.label);

                    let ch = 15;
                    let note_offset = if is_switch { 4 } else { 0 };

                    if let Some(midi) = &mut var_el.midi {
                        match midi {
                            MidiConfig::CcRelative { .. } | MidiConfig::Cc { .. } => {
                                if is_encoder {
                                    *midi = MidiConfig::CcRelative {
                                        channel: ch,
                                        controller: v.enc_cc_base + index,
                                    };
                                }
                            }
                            MidiConfig::Note { .. } => {
                                // Push or Switch
                                let note = v.push_note_base + index + note_offset;
                                *midi = MidiConfig::Note { channel: ch, note };
                            }
                        }
                    }
                    variants.push(var_el);
                }
            } else {
                new_list.push(el.clone());
            }
        }

        new_list.append(&mut variants);
        elements.elements = new_list;
    }

    /// Process incoming MIDI message
    #[cfg(feature = "midi")]
    pub fn process_midi(&mut self, message: MidiMessage) {
        // Check if in learn mode
        if self.learn_manager.process(message) {
            return; // Message was consumed by learn mode
        }

        // Update element states based on message
        if let Some(elements) = &self.elements {
            for element in &elements.elements {
                if let Some(midi_config) = &element.midi {
                    if Self::message_matches_config(&message, midi_config) {
                        // Track activity for global learn
                        self.last_active_element = Some(element.id.clone());
                        self.last_active_time = Some(std::time::Instant::now());

                        match message {
                            MidiMessage::ControlChange { value, .. } => {
                                self.state_manager.update_cc(&element.id, value);
                            }
                            MidiMessage::NoteOn { velocity, .. } => {
                                self.state_manager.update_note_on(&element.id, velocity);
                            }
                            MidiMessage::NoteOff { .. } => {
                                self.state_manager.update_note_off(&element.id);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Check if a MIDI message matches an element's config
    #[cfg(feature = "midi")]
    fn message_matches_config(
        message: &MidiMessage,
        config: &mapmap_control::midi::MidiConfig,
    ) -> bool {
        use mapmap_control::midi::MidiConfig;

        match (message, config) {
            (
                MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                },
                MidiConfig::Cc {
                    channel: cfg_ch,
                    controller: cfg_cc,
                },
            ) => *channel == *cfg_ch && *controller == *cfg_cc,
            (
                MidiMessage::ControlChange {
                    channel,
                    controller,
                    ..
                },
                MidiConfig::CcRelative {
                    channel: cfg_ch,
                    controller: cfg_cc,
                },
            ) => *channel == *cfg_ch && *controller == *cfg_cc,
            (
                MidiMessage::NoteOn { channel, note, .. },
                MidiConfig::Note {
                    channel: cfg_ch,
                    note: cfg_note,
                },
            ) => *channel == *cfg_ch && *note == *cfg_note,
            (
                MidiMessage::NoteOff { channel, note },
                MidiConfig::Note {
                    channel: cfg_ch,
                    note: cfg_note,
                },
            ) => *channel == *cfg_ch && *note == *cfg_note,
            _ => false,
        }
    }

    /// Start MIDI learn for an element
    #[cfg(feature = "midi")]
    pub fn start_learn(&mut self, element_id: &str, target: MidiLearnTarget) {
        self.learn_target = Some(target);
        self.learn_manager.start_learning(element_id);
    }

    /// Cancel MIDI learn
    #[cfg(feature = "midi")]
    pub fn cancel_learn(&mut self) {
        self.learn_target = None;
        self.learn_manager.cancel();
    }

    /// Check if currently learning
    #[cfg(feature = "midi")]
    pub fn is_learning(&self) -> bool {
        self.learn_manager.is_learning()
    }

    /// Show the panel UI
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        visible: bool,
        midi_connected: bool,
        user_config: &mut UserConfig,
    ) {
        if !visible {
            return;
        }

        // Ensure resources are loaded
        self.ensure_resources_loaded(ctx);

        egui::Window::new("🎛️ Ecler NUO 4 Controller")
            .resizable(true)
            .auto_sized()
            .collapsible(true)
            .min_width(400.0)
            .min_height(300.0)
            .max_width(2500.0)
            .show(ctx, |ui| {
                // === TOOLBAR ===
                ui.horizontal(|ui| {
                    // MIDI Connection Status
                    if midi_connected {
                        ui.colored_label(Color32::GREEN, "🟢 MIDI");
                    } else {
                        ui.colored_label(Color32::RED, "🔴 MIDI");
                    }

                    ui.separator();

                    // Scale slider
                    ui.label("Zoom:");
                    if ui
                        .add(egui::Slider::new(&mut self.scale, MIN_SCALE..=1.0).show_value(false))
                        .changed()
                    {
                        // Scale changed
                    }
                    ui.label(format!("{}%", (self.scale * 100.0) as i32));

                    ui.separator();

                    // Toggle buttons
                    ui.checkbox(&mut self.show_labels, "Labels");
                    ui.checkbox(&mut self.show_values, "Values");

                    ui.separator();

                    // Element list toggle
                    if ui
                        .button(if self.show_element_list {
                            "🎛️ Overlay"
                        } else {
                            "📋 Liste"
                        })
                        .clicked()
                    {
                        self.show_element_list = !self.show_element_list;
                    }

                    // Assignment colors toggle
                    let assign_btn = if self.show_assignment_colors {
                        egui::Button::new("🎨 Zuweisungen").fill(Color32::from_rgb(60, 80, 100))
                    } else {
                        egui::Button::new("🎨 Zuweisungen")
                    };
                    if ui.add(assign_btn).on_hover_text("Zeigt alle Elemente farblich nach Zuweisung:\n🟢 Frei\n🔵 MapFlow\n🟣 Streamer.bot\n🟠 Mixxx").clicked() {
                        self.show_assignment_colors = !self.show_assignment_colors;
                    }

                    ui.separator();

                    let edit_btn = if self.is_edit_mode {
                        egui::Button::new("✏️ Edit").fill(Color32::YELLOW)
                    } else {
                        egui::Button::new("✏️ Edit")
                    };
                    if ui.add(edit_btn).on_hover_text("Verschiebemodus aktivieren (Elemente am Overlay verschieben)").clicked() {
                        self.is_edit_mode = !self.is_edit_mode;
                        // Auto-save when exiting edit mode
                        if !self.is_edit_mode {
                            self.save_elements();
                        }
                    }

                    if self.is_edit_mode && ui.button("💾").on_hover_text("Positionen speichern").clicked() {
                        self.save_elements();
                    }
                });

                ui.separator();

                // === MIDI LEARN BUTTONS ===
                ui.horizontal(|ui| {
                    ui.label("MIDI Learn:");

                    #[cfg(feature = "midi")]
                    {
                        let is_learning = self.is_learning();

                        // MapFlow Learn
                        let mapflow_btn = if is_learning
                            && matches!(self.learn_target, Some(MidiLearnTarget::MapFlow))
                        {
                            ui.add(egui::Button::new("⏳ MapFlow...").fill(Color32::YELLOW))
                        } else {
                            ui.button("🎯 MapFlow")
                        };
                        if mapflow_btn.clicked() && !is_learning {
                            self.learn_target = Some(MidiLearnTarget::MapFlow);
                            // Will start learn when element is clicked
                        }

                        ui.separator();

                        // Streamer.bot Learn with input
                        ui.label("Streamer.bot:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.streamerbot_function)
                                .desired_width(100.0)
                                .hint_text("Funktion"),
                        );
                        let sb_btn = if is_learning
                            && matches!(self.learn_target, Some(MidiLearnTarget::StreamerBot(_)))
                        {
                            ui.add(egui::Button::new("⏳...").fill(Color32::YELLOW))
                        } else {
                            ui.button("🎯")
                        };
                        if sb_btn.clicked() && !is_learning && !self.streamerbot_function.is_empty()
                        {
                            self.learn_target = Some(MidiLearnTarget::StreamerBot(
                                self.streamerbot_function.clone(),
                            ));
                        }

                        ui.separator();

                        // Mixxx Learn with input
                        ui.label("Mixxx:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mixxx_function)
                                .desired_width(100.0)
                                .hint_text("Funktion"),
                        );
                        let mx_btn = if is_learning
                            && matches!(self.learn_target, Some(MidiLearnTarget::Mixxx(_)))
                        {
                            ui.add(egui::Button::new("⏳...").fill(Color32::YELLOW))
                        } else {
                            ui.button("🎯")
                        };
                        if mx_btn.clicked() && !is_learning && !self.mixxx_function.is_empty() {
                            self.learn_target =
                                Some(MidiLearnTarget::Mixxx(self.mixxx_function.clone()));
                        }

                        // Cancel button
                        if is_learning && ui.button("❌ Abbrechen").clicked() {
                            self.cancel_learn();
                        }
                    }

                    #[cfg(not(feature = "midi"))]
                    {
                        ui.label("(MIDI deaktiviert)");
                    }
                });

                ui.separator();

                if self.show_element_list {
                    self.show_element_list_view(ui, user_config);
                } else {
                    egui::ScrollArea::both()
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            self.show_overlay_view(ui, &user_config.midi_assignments);
                        });
                }
            });
    }

    /// Show the visual overlay with mixer background
    fn show_overlay_view(&mut self, ui: &mut Ui, assignments: &[MidiAssignment]) {
        let (base_w, base_h) = if let Some(tex) = &self.background_texture {
            let size = tex.size();
            (size[0] as f32, size[1] as f32)
        } else {
            (MAX_WIDTH, MAX_HEIGHT)
        };

        let panel_width = base_w * self.scale;
        let panel_height = base_h * self.scale;

        // Allocate space for the overlay
        let (response, painter) = ui.allocate_painter(
            Vec2::new(panel_width, panel_height),
            Sense::click_and_drag(),
        );

        let rect = response.rect;

        // Draw background image
        if let Some(texture) = &self.background_texture {
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Fallback: dark background
            let bg_color = Color32::from_rgb(30, 30, 35);
            painter.rect_filled(rect, 4.0, bg_color);
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::from_rgb(80, 80, 80)),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Hintergrundbild wird geladen...",
                egui::FontId::default(),
                Color32::WHITE,
            );
        }

        // Draw elements with frames
        #[cfg(feature = "midi")]
        if self.is_edit_mode {
            // Local state to avoid borrow conflicts
            let mut next_selection = self.selected_element.clone();
            let mut next_clipboard = self.clipboard_size;

            if let Some(elements) = &mut self.elements {
                for element in &mut elements.elements {
                    let elem_rect = Rect::from_min_size(
                        Pos2::new(
                            rect.min.x + element.position.x * rect.width(),
                            rect.min.y + element.position.y * rect.height(),
                        ),
                        Vec2::new(
                            element.position.width * rect.width(),
                            element.position.height * rect.height(),
                        ),
                    );

                    // Resize Handle Rect (Bottom-Right)
                    let handle_size = 10.0;
                    let handle_rect = Rect::from_min_size(
                        elem_rect.max - Vec2::splat(handle_size),
                        Vec2::splat(handle_size),
                    );

                    // Interactions
                    let id = ui.make_persistent_id(&element.id);
                    let handle_id = id.with("resize");

                    // Define Move interaction (click_and_drag for selection)
                    let move_interact = ui.interact(elem_rect, id, Sense::click_and_drag());
                    // Define Handle interaction
                    let handle_interact = ui.interact(handle_rect, handle_id, Sense::drag());

                    // Handle Selection
                    if move_interact.clicked() || move_interact.dragged() {
                        next_selection = Some(element.id.clone());
                    }

                    let is_selected = next_selection.as_ref() == Some(&element.id);
                    let mut anim_handle_dragged = false;

                    // Animation Handles (Fader only)
                    if is_selected && element.element_type == ElementType::Fader {
                        let mut range = element.animation_range.unwrap_or([0.0, 1.0]);
                        let mut changed = false;

                        let h = elem_rect.height();
                        let y_top = elem_rect.min.y + range[0] * h;
                        let y_bot = elem_rect.min.y + range[1] * h;

                        // Handles
                        let h_top_rect = Rect::from_center_size(
                            Pos2::new(elem_rect.center().x, y_top),
                            Vec2::new(elem_rect.width(), 8.0),
                        );
                        let h_bot_rect = Rect::from_center_size(
                            Pos2::new(elem_rect.center().x, y_bot),
                            Vec2::new(elem_rect.width(), 8.0),
                        );

                        let h_top_res = ui.interact(h_top_rect, id.with("h_top"), Sense::drag());
                        let h_bot_res = ui.interact(h_bot_rect, id.with("h_bot"), Sense::drag());

                        if h_top_res.dragged() {
                            range[0] += h_top_res.drag_delta().y / h;
                            changed = true;
                            anim_handle_dragged = true;
                        }
                        if h_bot_res.dragged() {
                            range[1] += h_bot_res.drag_delta().y / h;
                            changed = true;
                            anim_handle_dragged = true;
                        }

                        if changed {
                            element.animation_range = Some(range);
                        }

                        // Draw Handles
                        painter.line_segment(
                            [
                                Pos2::new(elem_rect.min.x, y_top),
                                Pos2::new(elem_rect.max.x, y_top),
                            ],
                            Stroke::new(1.0, Color32::RED),
                        );
                        painter.line_segment(
                            [
                                Pos2::new(elem_rect.min.x, y_bot),
                                Pos2::new(elem_rect.max.x, y_bot),
                            ],
                            Stroke::new(1.0, Color32::RED),
                        );

                        painter.text(
                            Pos2::new(elem_rect.max.x + 2.0, y_top),
                            egui::Align2::LEFT_CENTER,
                            "Top",
                            egui::FontId::proportional(12.0),
                            Color32::RED,
                        );
                        painter.text(
                            Pos2::new(elem_rect.max.x + 2.0, y_bot),
                            egui::Align2::LEFT_CENTER,
                            "Bot",
                            egui::FontId::proportional(12.0),
                            Color32::RED,
                        );
                    }

                    // Logic
                    if handle_interact.dragged() {
                        let delta = handle_interact.drag_delta();
                        if rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.width += delta.x / rect.width();
                            element.position.height += delta.y / rect.height();
                            element.position.width = element.position.width.max(0.01);
                            element.position.height = element.position.height.max(0.01);
                        }
                    } else if anim_handle_dragged {
                        // Consumed by animation handles
                    } else if move_interact.dragged() {
                        let delta = move_interact.drag_delta();
                        if rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.x += delta.x / rect.width();
                            element.position.y += delta.y / rect.height();
                        }
                    }

                    // Keyboard Actions (only if selected)
                    let is_selected = next_selection.as_ref() == Some(&element.id);
                    if is_selected {
                        let mut delta_key = Vec2::ZERO;
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                            delta_key.x -= 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                            delta_key.x += 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            delta_key.y -= 1.0;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            delta_key.y += 1.0;
                        }

                        if delta_key != Vec2::ZERO && rect.width() > 0.0 && rect.height() > 0.0 {
                            element.position.x += delta_key.x / rect.width();
                            element.position.y += delta_key.y / rect.height();
                        }

                        // Copy / Paste Size (Ctrl+C / Ctrl+V)
                        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
                            next_clipboard =
                                Some([element.position.width, element.position.height]);
                        }
                        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::V)) {
                            if let Some(size) = next_clipboard {
                                element.position.width = size[0];
                                element.position.height = size[1];
                            }
                        }
                    }

                    // Draw Asset (in background of frame)
                    let current_val = self
                        .state_manager
                        .get(&element.id)
                        .map(|s| s.value as f32 / 127.0)
                        .unwrap_or(0.0);
                    Self::draw_asset(
                        &painter,
                        &self.assets,
                        self.scale,
                        elem_rect,
                        element,
                        current_val,
                    );

                    // Draw edit frame
                    let base_color = if is_selected {
                        Color32::from_rgb(255, 0, 255)
                    } else {
                        Color32::YELLOW
                    }; // Magenta for selected
                    let stroke = Stroke::new(if is_selected { 2.0 } else { 1.0 }, base_color);

                    match element.element_type {
                        ElementType::Knob | ElementType::Encoder => {
                            let radius = elem_rect.width().min(elem_rect.height()) / 2.0;
                            painter.circle_stroke(elem_rect.center(), radius, stroke);
                        }
                        _ => {
                            painter.rect_stroke(elem_rect, 0.0, stroke);
                        }
                    }

                    // Draw resize handle
                    painter.rect_filled(handle_rect, 2.0, Color32::from_rgb(0, 255, 255));
                }
            }

            // Apply state updates
            self.selected_element = next_selection;
            self.clipboard_size = next_clipboard;
        } else if let Some(elements) = self.elements.clone() {
            for element in &elements.elements {
                self.draw_element_with_frame(&painter, rect, element, &response, assignments);
            }
        }
    }

    /// Draw a single element with colored frame
    #[cfg(feature = "midi")]
    fn draw_element_with_frame(
        &mut self,
        painter: &egui::Painter,
        container: Rect,
        element: &ControllerElement,
        response: &Response,
        assignments: &[MidiAssignment],
    ) {
        // Calculate element rect based on relative position
        let elem_rect = Rect::from_min_size(
            Pos2::new(
                container.min.x + element.position.x * container.width(),
                container.min.y + element.position.y * container.height(),
            ),
            Vec2::new(
                element.position.width * container.width(),
                element.position.height * container.height(),
            ),
        );

        // Check states
        let state = self.state_manager.get(&element.id);

        // Draw Asset
        let val = state.map(|s| s.value as f32 / 127.0).unwrap_or(0.0);
        Self::draw_asset(painter, &self.assets, self.scale, elem_rect, element, val);

        let is_hovered = response
            .hover_pos()
            .map(|pos| elem_rect.contains(pos))
            .unwrap_or(false);
        let is_selected = self.selected_element.as_ref() == Some(&element.id);
        let is_learning = self.learn_manager.is_learning()
            && self.learn_manager.state().target_element() == Some(element.id.as_str());
        // Check if element was recently updated (within last 200ms)
        let is_active = state
            .map(|s: &ElementState| s.last_update.elapsed().as_millis() < 200)
            .unwrap_or(false);

        // Determine frame color based on state
        let frame_color = if is_learning {
            // Pulsing yellow for learn mode
            let t = (ui_time_seconds() * 3.0).sin() * 0.5 + 0.5;
            Color32::from_rgba_unmultiplied(255, 220, 0, (128.0 + 127.0 * t as f32) as u8)
        } else if is_active {
            Color32::GREEN
        } else if is_selected {
            Color32::from_rgb(100, 149, 237) // Cornflower blue
        } else if is_hovered {
            Color32::WHITE
        } else {
            Color32::TRANSPARENT
        };

        // Override colors assignments view is active
        let frame_color = if self.show_assignment_colors {
            let assignment = assignments.iter().find(|a| a.element_id == element.id);
            match assignment {
                Some(a) => match &a.target {
                    MidiAssignmentTarget::MapFlow(_) => Color32::from_rgb(0, 150, 255), // Blue
                    MidiAssignmentTarget::StreamerBot(_) => Color32::from_rgb(180, 0, 255), // Purple
                    MidiAssignmentTarget::Mixxx(_) => Color32::from_rgb(255, 128, 0), // Orange
                },
                None => Color32::GREEN, // Green for free elements
            }
        } else {
            frame_color
        };

        // Draw frame
        if frame_color != Color32::TRANSPARENT {
            let stroke_width = if is_learning { 3.0 } else { 2.0 };
            let stroke = Stroke::new(stroke_width, frame_color);
            match element.element_type {
                ElementType::Knob | ElementType::Encoder => {
                    let radius = elem_rect.width().min(elem_rect.height()) / 2.0;
                    painter.circle_stroke(elem_rect.center(), radius, stroke);
                }
                _ => {
                    painter.rect_stroke(elem_rect, 4.0, stroke);
                }
            }
        }

        // Update hovered element for tooltip
        if is_hovered {
            self.hovered_element = Some(element.id.clone());
        }

        // Handle click for MIDI learn
        if response.clicked() && is_hovered {
            if let Some(target) = &self.learn_target {
                self.learn_manager.start_learning(&element.id);
                tracing::info!(
                    "Started MIDI learn for {} with target {:?}",
                    element.id,
                    target
                );
            } else {
                self.selected_element = Some(element.id.clone());
            }
        }

        // Show tooltip on hover
        if is_hovered {
            #[allow(deprecated)]
            egui::show_tooltip(
                painter.ctx(),
                egui::LayerId::background(),
                egui::Id::new(&element.id),
                |ui| {
                    ui.strong(&element.label);
                    ui.label(format!("ID: {}", element.id));
                    ui.label(format!("Typ: {:?}", element.element_type));
                    if let Some(midi) = &element.midi {
                        ui.label(format!("MIDI: {:?}", midi));
                    }
                    if let Some(state) = state {
                        ui.label(format!("Wert: {:.2}", state.value));
                    }

                    // Show assignment info
                    let assignment = assignments.iter().find(|a| a.element_id == element.id);
                    if let Some(assign) = assignment {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Zuweisung:");
                            ui.colored_label(Color32::YELLOW, assign.target.to_string());
                        });
                        ui.label(
                            egui::RichText::new("(Klick für Details in Liste)")
                                .italics()
                                .size(10.0),
                        );
                    } else {
                        ui.separator();
                        ui.label(egui::RichText::new("Nicht zugewiesen").italics().weak());
                    }
                },
            );
        }
    }

    /// Save elements to disk
    fn save_elements(&self) {
        #[cfg(feature = "midi")]
        if let Some(elements) = &self.elements {
            let paths = [
                "resources/controllers/ecler_nuo4/elements.json",
                "../resources/controllers/ecler_nuo4/elements.json",
                r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\elements.json",
            ];

            for path_str in paths {
                let path = std::path::Path::new(path_str);
                if path.exists() {
                    match serde_json::to_string_pretty(elements) {
                        Ok(json) => {
                            if let Err(e) = std::fs::write(path, json) {
                                tracing::error!("Failed to save elements to {:?}: {}", path, e);
                            } else {
                                tracing::info!("Saved elements to {:?}", path);
                            }
                        }
                        Err(e) => tracing::error!("Failed to serialize elements: {}", e),
                    }
                    return;
                }
            }
            tracing::error!("Could not find elements.json to save to.");
        }
    }

    /// Show the element list view
    fn show_element_list_view(&mut self, ui: &mut Ui, user_config: &mut UserConfig) {
        // Filter buttons
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui
                .selectable_label(self.element_filter == ElementFilter::All, "Alle")
                .clicked()
            {
                self.element_filter = ElementFilter::All;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::MapFlow, "MapFlow")
                .clicked()
            {
                self.element_filter = ElementFilter::MapFlow;
            }
            if ui
                .selectable_label(
                    self.element_filter == ElementFilter::StreamerBot,
                    "Streamer.bot",
                )
                .clicked()
            {
                self.element_filter = ElementFilter::StreamerBot;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::Mixxx, "Mixxx")
                .clicked()
            {
                self.element_filter = ElementFilter::Mixxx;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::Unassigned, "Frei")
                .clicked()
            {
                self.element_filter = ElementFilter::Unassigned;
            }
        });

        ui.separator();

        // Element table
        let mut element_to_remove = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("element_list")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.strong("ID");
                    ui.strong("Name");
                    ui.strong("Typ");
                    ui.strong("MIDI");
                    ui.strong("Zuweisung / Aktion");
                    ui.end_row();

                    #[cfg(feature = "midi")]
                    if let Some(elements) = &self.elements {
                        for element in &elements.elements {
                            // Determine assignment status
                            let assignment = user_config.get_midi_assignment(&element.id);

                            // Apply filter
                            let show = match self.element_filter {
                                ElementFilter::All => true,
                                ElementFilter::MapFlow => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::MapFlow(_))),
                                ElementFilter::StreamerBot => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::StreamerBot(_))),
                                ElementFilter::Mixxx => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::Mixxx(_))),
                                ElementFilter::Unassigned => assignment.is_none(),
                            };

                            if !show {
                                continue;
                            }

                            ui.label(&element.id);
                            ui.label(&element.label);
                            ui.label(format!("{:?}", element.element_type));
                            if let Some(midi) = &element.midi {
                                ui.label(format!("{:?}", midi));
                            } else {
                                ui.label("-");
                            }

                            // Show assignment and delete button
                            if let Some(assign) = assignment {
                                ui.horizontal(|ui| {
                                    ui.label(assign.target.to_string());
                                    if ui.small_button("🗑").on_hover_text("Zuweisung löschen").clicked() {
                                        element_to_remove = Some(element.id.clone());
                                    }
                                });
                            } else {
                                ui.label("-");
                            }
                            ui.end_row();
                        }
                    }
                });
        });

        // Handle deletion request outside of borrow loop
        if let Some(id) = element_to_remove {
            user_config.remove_midi_assignment(&id);
        }
    }
}

/// Get current time in seconds for animations
fn ui_time_seconds() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
