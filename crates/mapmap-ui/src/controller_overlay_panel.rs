//! Controller Overlay Panel
//!
//! Visual representation of the Ecler NUO 4 (or other MIDI controllers)
//! with live state visualization and MIDI Learn functionality.

#[cfg(feature = "midi")]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[cfg(feature = "midi")]
use mapmap_control::midi::{
    ControllerElement, ControllerElements, ElementState, ElementStateManager, MidiLearnManager,
    MidiMessage,
};

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
    learn_target: Option<MidiLearnTarget>,

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
        }
    }

    /// Load background image
    fn ensure_background_loaded(&mut self, ctx: &egui::Context) {
        if self.background_texture.is_some() {
            return;
        }

        let paths = [
            "resources/controllers/ecler_nuo4/background.jpg",
            "../resources/controllers/ecler_nuo4/background.jpg",
            r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\background.jpg",
        ];

        for path_str in paths {
            let path = std::path::Path::new(path_str);
            if path.exists() {
                if let Ok(image_data) = std::fs::read(path) {
                    if let Ok(img) = image::load_from_memory(&image_data) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();

                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

                        self.background_texture = Some(ctx.load_texture(
                            "mixer_background",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                        tracing::info!("Loaded mixer background from {}", path_str);
                        break;
                    }
                }
            }
        }
    }

    /// Load controller elements from JSON
    #[cfg(feature = "midi")]
    pub fn load_elements(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let elements = ControllerElements::from_json(json)?;
        self.elements = Some(elements);
        Ok(())
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

        // Ensure background is loaded
        self.ensure_background_loaded(ctx);

        // Calculate window size based on scale
        let window_width = MAX_WIDTH * self.scale;
        let window_height = MAX_HEIGHT * self.scale;

        egui::Window::new("🎛️ Ecler NUO 4 Controller")
            .resizable(false) // Use slider for scaling instead
            .collapsible(true)
            .default_size([window_width + 20.0, window_height + 120.0])
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
                    self.show_overlay_view(ui, &user_config.midi_assignments);
                }
            });
    }

    /// Show the visual overlay with mixer background
    fn show_overlay_view(&mut self, ui: &mut Ui, assignments: &[MidiAssignment]) {
        let panel_width = MAX_WIDTH * self.scale;
        let panel_height = MAX_HEIGHT * self.scale;

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
            painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 35));
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
        if let Some(elements) = self.elements.clone() {
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
            painter.rect_stroke(elem_rect, 4.0, Stroke::new(stroke_width, frame_color));
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
            egui::show_tooltip_at_pointer(painter.ctx(), egui::Id::new(&element.id), |ui| {
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
            });
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
