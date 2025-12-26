//! Controller Overlay Panel
//!
//! Visual representation of the Ecler NUO 4 (or other MIDI controllers)
//! with live state visualization and MIDI Learn functionality.

#[cfg(feature = "midi")]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Vec2};

#[cfg(feature = "midi")]
use mapmap_control::midi::{
    ControllerElement, ControllerElements, ElementState, ElementStateManager, ElementType,
    MidiLearnManager, MidiMessage,
};

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

    /// Show element labels
    show_labels: bool,

    /// Show element values
    show_values: bool,

    /// Show MIDI info on hover
    #[allow(dead_code)]
    show_midi_info: bool,

    /// Selected element for editing
    #[allow(dead_code)]
    selected_element: Option<String>,

    /// Panel is expanded
    is_expanded: bool,
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
            show_labels: true,
            show_values: true,
            show_midi_info: true,
            selected_element: None,
            is_expanded: true,
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
    pub fn start_learn(&mut self, element_id: &str) {
        self.learn_manager.start_learning(element_id);
    }

    /// Cancel MIDI learn
    #[cfg(feature = "midi")]
    pub fn cancel_learn(&mut self) {
        self.learn_manager.cancel();
    }

    /// Show the panel UI
    pub fn show(&mut self, ctx: &egui::Context, is_open: &mut bool) {
        if !*is_open {
            return;
        }

        egui::Window::new("Controller Overlay")
            .open(is_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("🎛️ Controller Overlay");

                    if ui
                        .button(if self.is_expanded { "⏷" } else { "⏵" })
                        .clicked()
                    {
                        self.is_expanded = !self.is_expanded;
                    }

                    ui.separator();

                    ui.checkbox(&mut self.show_labels, "Labels");
                    ui.checkbox(&mut self.show_values, "Values");
                });

                if !self.is_expanded {
                    return;
                }

                ui.separator();

                #[cfg(feature = "midi")]
                {
                    // Show learn mode status
                    if self.learn_manager.is_learning() {
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::YELLOW, "⏳ MIDI Learn aktiv");
                            if let Some(remaining) = self.learn_manager.state().remaining_time() {
                                ui.label(format!("({:.0}s)", remaining.as_secs_f32()));
                            }
                            if ui.button("Abbrechen").clicked() {
                                self.learn_manager.cancel();
                            }
                        });
                        ui.separator();
                    }

                    // Check for detected mapping
                    if self.learn_manager.has_detection() {
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::GREEN, "✓ MIDI erkannt!");
                            if let Some(key) = self.learn_manager.state().get_detected_key() {
                                ui.label(format!("{:?}", key));
                            }
                            if ui.button("Übernehmen").clicked() {
                                if let Some((_element_id, _key)) = self.learn_manager.accept() {
                                    // TODO: Store the mapping
                                }
                            }
                        });
                        ui.separator();
                    }

                    // Update learn manager
                    self.learn_manager.update();

                    // Draw controller overlay
                    if let Some(elements) = &self.elements {
                        self.draw_controller(ui, elements);
                    } else {
                        ui.label("Kein Controller geladen");
                        if ui.button("Ecler NUO 4 laden").clicked() {
                            // Load default Ecler NUO 4 elements
                            let json = include_str!(
                                "../../../resources/controllers/ecler_nuo4/elements.json"
                            );
                            if let Err(e) = self.load_elements(json) {
                                tracing::error!("Failed to load elements: {}", e);
                            }
                        }
                    }
                }

                #[cfg(not(feature = "midi"))]
                {
                    ui.label("MIDI-Feature ist nicht aktiviert");
                }
            });
    }

    /// Draw the controller visualization
    #[cfg(feature = "midi")]
    fn draw_controller(&mut self, ui: &mut egui::Ui, elements: &ControllerElements) {
        let available_size = ui.available_size();
        let panel_size = Vec2::new(available_size.x.min(600.0), available_size.y.min(400.0));

        let (response, painter) = ui.allocate_painter(panel_size, Sense::click());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 8.0, Color32::from_rgb(30, 30, 35));
        painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(60, 60, 70)));

        // Draw each element
        for element in &elements.elements {
            let state = self.state_manager.get(&element.id);
            self.draw_element(&painter, rect, element, state, &response);
        }

        // Draw section labels
        let sections = elements.sections();
        for (i, section) in sections.iter().enumerate() {
            let y = rect.min.y + 15.0 + (i as f32 * 12.0);
            painter.text(
                Pos2::new(rect.min.x + 5.0, y),
                egui::Align2::LEFT_TOP,
                *section,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(100, 100, 110),
            );
        }
    }

    /// Draw a single element
    #[cfg(feature = "midi")]
    fn draw_element(
        &self,
        painter: &egui::Painter,
        container: Rect,
        element: &ControllerElement,
        state: Option<&ElementState>,
        _response: &Response,
    ) {
        let pos = element.position;
        let elem_rect = Rect::from_min_size(
            Pos2::new(
                container.min.x + pos.x * container.width(),
                container.min.y + pos.y * container.height(),
            ),
            Vec2::new(
                pos.width * container.width(),
                pos.height * container.height(),
            ),
        );

        let normalized = state.map(|s| s.normalized).unwrap_or(0.0);
        let active = state.map(|s| s.active).unwrap_or(false);

        match element.element_type {
            ElementType::Knob | ElementType::Encoder => {
                self.draw_knob(painter, elem_rect, normalized, &element.label);
            }
            ElementType::Fader => {
                self.draw_fader(painter, elem_rect, normalized, &element.label);
            }
            ElementType::Crossfader => {
                self.draw_crossfader(painter, elem_rect, normalized, &element.label);
            }
            ElementType::Button => {
                self.draw_button(painter, elem_rect, active, &element.label);
            }
            ElementType::Toggle => {
                self.draw_toggle(painter, elem_rect, active, &element.label);
            }
        }
    }

    #[cfg(feature = "midi")]
    fn draw_knob(&self, painter: &egui::Painter, rect: Rect, value: f32, label: &str) {
        let center = rect.center();
        let radius = rect.width().min(rect.height()) / 2.0 * 0.8;

        // Knob body
        painter.circle_filled(center, radius, Color32::from_rgb(50, 50, 55));
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(2.0, Color32::from_rgb(80, 80, 90)),
        );

        // Indicator line
        let angle = std::f32::consts::PI * 0.75 + value * std::f32::consts::PI * 1.5;
        let indicator_end = Pos2::new(
            center.x + angle.cos() * radius * 0.7,
            center.y + angle.sin() * radius * 0.7,
        );
        painter.line_segment(
            [center, indicator_end],
            Stroke::new(2.0, Color32::from_rgb(200, 100, 50)),
        );

        // Label
        if self.show_labels {
            painter.text(
                Pos2::new(center.x, rect.max.y + 2.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(150, 150, 160),
            );
        }

        // Value
        if self.show_values {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{:.0}", value * 127.0),
                egui::FontId::proportional(9.0),
                Color32::WHITE,
            );
        }
    }

    #[cfg(feature = "midi")]
    fn draw_fader(&self, painter: &egui::Painter, rect: Rect, value: f32, label: &str) {
        // Track
        let track_rect = Rect::from_center_size(
            rect.center(),
            Vec2::new(rect.width() * 0.3, rect.height() * 0.9),
        );
        painter.rect_filled(track_rect, 2.0, Color32::from_rgb(40, 40, 45));

        // Fader position
        let fader_y = track_rect.max.y - value * track_rect.height();
        let fader_rect = Rect::from_center_size(
            Pos2::new(rect.center().x, fader_y),
            Vec2::new(rect.width() * 0.8, 8.0),
        );
        painter.rect_filled(fader_rect, 2.0, Color32::from_rgb(200, 100, 50));

        // Label
        if self.show_labels {
            painter.text(
                Pos2::new(rect.center().x, rect.max.y + 2.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(150, 150, 160),
            );
        }
    }

    #[cfg(feature = "midi")]
    fn draw_crossfader(&self, painter: &egui::Painter, rect: Rect, value: f32, label: &str) {
        // Track
        let track_rect = Rect::from_center_size(
            rect.center(),
            Vec2::new(rect.width() * 0.9, rect.height() * 0.3),
        );
        painter.rect_filled(track_rect, 2.0, Color32::from_rgb(40, 40, 45));

        // Fader position
        let fader_x = track_rect.min.x + value * track_rect.width();
        let fader_rect = Rect::from_center_size(
            Pos2::new(fader_x, rect.center().y),
            Vec2::new(12.0, rect.height() * 0.7),
        );
        painter.rect_filled(fader_rect, 2.0, Color32::from_rgb(200, 100, 50));

        // Label
        if self.show_labels {
            painter.text(
                Pos2::new(rect.center().x, rect.min.y - 2.0),
                egui::Align2::CENTER_BOTTOM,
                label,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(150, 150, 160),
            );
        }
    }

    #[cfg(feature = "midi")]
    fn draw_button(&self, painter: &egui::Painter, rect: Rect, active: bool, label: &str) {
        let color = if active {
            Color32::from_rgb(80, 200, 100)
        } else {
            Color32::from_rgb(50, 50, 55)
        };

        painter.rect_filled(rect, 3.0, color);
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 90)));

        if self.show_labels {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(7.0),
                Color32::WHITE,
            );
        }
    }

    #[cfg(feature = "midi")]
    fn draw_toggle(&self, painter: &egui::Painter, rect: Rect, active: bool, label: &str) {
        let color = if active {
            Color32::from_rgb(200, 150, 50)
        } else {
            Color32::from_rgb(50, 50, 55)
        };

        painter.rect_filled(rect, 2.0, color);
        painter.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 90)));

        if self.show_labels {
            painter.text(
                Pos2::new(rect.center().x, rect.max.y + 2.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(6.0),
                Color32::from_rgb(150, 150, 160),
            );
        }
    }
}
