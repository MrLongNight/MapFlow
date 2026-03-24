use super::drawing::MAX_WIDTH;
// Controller Overlay Panel
//
// Visual representation of the Ecler NUO 4 (or other MIDI controllers)
// with live state visualization and MIDI Learn functionality.

#[allow(unused_imports)]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

#[allow(unused_imports)]
use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[cfg(feature = "midi")]
use vorce_control::midi::{ControllerElements, ElementStateManager, MidiLearnManager};
use vorce_control::target::ControlTarget;
#[allow(unused_imports)]
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

const MIN_SCALE: f32 = 0.3;

#[derive(Debug, Clone, PartialEq)]
pub enum MidiLearnTarget {
    MapFlow,
    StreamerBot(String), // Function name
    Mixxx(String),       // Function name
}

pub struct ControllerOverlayPanel {
    #[cfg(feature = "midi")]
    // List of interactive components (knobs, faders, buttons).
    pub(crate) elements: Option<ControllerElements>,

    #[cfg(feature = "midi")]
    pub(crate) state_manager: ElementStateManager,

    #[cfg(feature = "midi")]
    pub(crate) learn_manager: MidiLearnManager,

    pub learn_target: Option<MidiLearnTarget>,

    pub last_active_element: Option<String>,
    pub last_active_time: Option<std::time::Instant>,

    #[allow(dead_code)]
    streamerbot_function: String,

    #[allow(dead_code)]
    mixxx_function: String,

    pub(crate) show_labels: bool,

    pub(crate) show_values: bool,

    #[allow(dead_code)]
    pub(crate) show_midi_info: bool,

    #[allow(dead_code)]
    pub(crate) selected_element: Option<String>,

    #[allow(dead_code)]
    pub(crate) hovered_element: Option<String>,

    pub is_expanded: bool,

    pub(crate) scale: f32,

    pub(crate) background_texture: Option<TextureHandle>,

    pub(crate) show_element_list: bool,

    pub(crate) element_filter: ElementFilter,

    pub(crate) show_assignment_colors: bool,

    pub is_edit_mode: bool,

    #[allow(dead_code)]
    pub(crate) clipboard_size: Option<[f32; 2]>,

    #[allow(dead_code)]
    pub(crate) assets: HashMap<String, TextureHandle>,
}

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
            // List of interactive components (knobs, faders, buttons).
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

    pub(crate) fn load_texture_from_candidates<P: AsRef<std::path::Path>>(
        &self,
        ctx: &egui::Context,
        paths: &[P],
        name: &str,
    ) -> Option<TextureHandle> {
        for path in paths {
            let path = path.as_ref();
            if path.exists() {
                if let Ok(image_data) = std::fs::read(path) {
                    if let Ok(img) = image::load_from_memory(&image_data) {
                        let rgba = img.to_rgba8();
                        let width = rgba.width();
                        let height = rgba.height();
                        let size = [width as usize, height as usize];

                        let pixels: Vec<egui::Color32> = rgba
                            .pixels()
                            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                            .collect();

                        let color_image = egui::ColorImage {
                            size,
                            pixels,
                            source_size: egui::Vec2::new(width as f32, height as f32),
                        };

                        return Some(ctx.load_texture(
                            name,
                            color_image,
                            egui::TextureOptions {
                                magnification: egui::TextureFilter::Linear,
                                minification: egui::TextureFilter::Linear,
                                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                                mipmap_mode: None,
                            },
                        ));
                    }
                }
            }
        }
        None
    }

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
            .max_width(MAX_WIDTH * self.scale * 0.65) // Limit to Mixer section width (approx 65%)
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
                    if ui.add(assign_btn).clone().on_hover_text("Zeigt alle Elemente farblich nach Zuweisung:\n🟢 Frei\n🔵 MapFlow\n🟣 Streamer.bot\n🟠 Mixxx").clicked() {
                        self.show_assignment_colors = !self.show_assignment_colors;
                    }

                    ui.separator();

                    let edit_btn = if self.is_edit_mode {
                        egui::Button::new("✏️ Edit").fill(Color32::YELLOW)
                    } else {
                        egui::Button::new("✏️ Edit")
                    };
                    if ui.add(edit_btn).clone().on_hover_text("Verschiebemodus aktivieren (Elemente am Overlay verschieben)").clicked() {
                        self.is_edit_mode = !self.is_edit_mode;
                        // Auto-save when exiting edit mode
                        if !self.is_edit_mode {
                            self.save_elements();
                        }
                    }

                    if self.is_edit_mode && ui.button("💾").clone().on_hover_text("Positionen speichern").clicked() {
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
}
