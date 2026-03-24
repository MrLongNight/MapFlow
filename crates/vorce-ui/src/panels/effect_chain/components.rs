use super::types::EffectType;
use crate::i18n::LocaleManager;
use crate::icons::IconManager;
use crate::theme::colors;
use crate::widgets::custom::{delete_button, styled_slider};
use egui::{CornerRadius, RichText, Ui};

impl super::panel::EffectChainPanel {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub(crate) fn render_effect_card_static(
        ui: &mut Ui,
        effect_id: u64,
        effect_type: EffectType,
        mut enabled: bool,
        mut expanded: bool,
        mut intensity: f32,
        lut_path: &str,
        error: Option<&String>,
        parameters: &std::collections::HashMap<String, f32>,
        is_first: bool,
        is_last: bool,
        is_dragging: bool,
        index: usize,
        locale: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) -> (
        bool,
        bool,
        bool,
        bool,
        bool,
        egui::Rect,
        bool,
        bool,
        f32,
        Vec<(String, f32)>,
        Option<String>,
        Option<Option<String>>,
    ) {
        let mut remove = false;
        let mut reset = false;
        let mut move_up = false;
        let mut move_down = false;
        let mut dragged = false;
        let mut param_changes = Vec::new();
        let mut new_lut_path = None;
        let mut new_error = None;

        #[allow(clippy::manual_is_multiple_of)]
        let is_even = index % 2 == 0;

        let frame_color = if is_dragging {
            colors::CYAN_ACCENT.linear_multiply(0.4) // Highlight when dragging
        } else if enabled {
            // Active effect background - subtle tint
            colors::CYAN_ACCENT
                .linear_multiply(0.05)
                .gamma_multiply(0.5)
        } else {
            if is_even {
                colors::DARK_GREY
            } else {
                colors::DARKER_GREY
            }
        };

        // Add stroke if dragging or enabled
        let stroke = if is_dragging {
            egui::Stroke::new(2.0, colors::CYAN_ACCENT)
        } else if enabled {
            egui::Stroke::new(1.0, colors::CYAN_ACCENT.linear_multiply(0.5))
        } else {
            egui::Stroke::new(1.0, colors::STROKE_GREY)
        };

        let response = egui::Frame::default()
            .fill(frame_color)
            .stroke(stroke)
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(4.0)
            .outer_margin(2.0)
            .show(ui, |ui| {
                // Header row
                ui.horizontal(|ui| {
                    // Drag Handle
                    let handle_resp = ui
                        .add(
                            egui::Button::new("⋮⋮")
                                .frame(false)
                                .sense(egui::Sense::drag()),
                        )
                        .on_hover_text("Drag to reorder. Hold Alt+Up/Down to move.");

                    if handle_resp.drag_started() {
                        dragged = true;
                    }
                    if handle_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else if handle_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);

                        // Keyboard reordering
                        if ui.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowUp)) {
                            move_up = true;
                        }
                        if ui.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowDown)) {
                            move_down = true;
                        }
                    }

                    // Enable toggle
                    ui.checkbox(&mut enabled, "")
                        .on_hover_text("Enable/Disable Effect");

                    // Effect name with icon
                    let header_text = effect_type.display_name(locale);
                    if let Some(mgr) = icon_manager {
                        if let Some(img) = mgr.image(effect_type.app_icon(), 16.0) {
                            if ui
                                .add(
                                    egui::Button::image_and_text(img, header_text)
                                        .selected(expanded),
                                )
                                .clicked()
                            {
                                expanded = !expanded;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (Hold to Confirm)
                        if delete_button(ui) {
                            remove = true;
                        }
                        if crate::widgets::custom::hold_to_action_button(
                            ui,
                            "↺ Reset",
                            colors::WARN_COLOR,
                            "Reset",
                        ) {
                            reset = true;
                        }

                        // Move buttons
                        ui.add_enabled_ui(!is_last, |ui| {
                            if ui.small_button("▼").clicked() {
                                move_down = true;
                            }
                        });
                        ui.add_enabled_ui(!is_first, |ui| {
                            if ui.small_button("▲").clicked() {
                                move_up = true;
                            }
                        });
                    });
                });

                // Expanded content
                if expanded {
                    ui.separator();

                    // Intensity slider
                    ui.horizontal(|ui| {
                        ui.label(locale.t("effect-intensity"));
                        ui.add(egui::Slider::new(&mut intensity, 0.0..=1.0));
                    });

                    // LUT Path
                    if effect_type == EffectType::LoadLUT {
                        ui.horizontal(|ui| {
                            ui.label("LUT:");
                            let path_label = if lut_path.is_empty() {
                                "None"
                            } else {
                                std::path::Path::new(lut_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(lut_path)
                            };

                            if ui.button(path_label).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("LUT", &["cube", "png"])
                                    .pick_file()
                                {
                                    let path_str = path.to_string_lossy().to_string();
                                    match vorce_core::lut::Lut3D::from_file(&path) {
                                        Ok(_) => {
                                            new_lut_path = Some(path_str);
                                            new_error = Some(None);
                                        }
                                        Err(e) => {
                                            new_error = Some(Some(e.to_string()));
                                        }
                                    }
                                }
                            }
                        });

                        if let Some(err) = error {
                            ui.label(
                                RichText::new(format!("Error: {}", err))
                                    .color(colors::ERROR_COLOR)
                                    .small(),
                            );
                        }
                    }

                    // Effect-specific parameters
                    Self::render_effect_parameters_static(
                        ui,
                        effect_type,
                        effect_id,
                        parameters,
                        &mut param_changes,
                        locale,
                    );
                }
            });

        (
            remove,
            reset,
            move_up,
            move_down,
            dragged,
            response.response.rect, // Return the rect of the whole card
            enabled,
            expanded,
            intensity,
            param_changes,
            new_lut_path,
            new_error,
        )
    }

    pub(crate) fn render_effect_parameters_static(
        ui: &mut Ui,
        effect_type: EffectType,
        effect_id: u64,
        parameters: &std::collections::HashMap<String, f32>,
        param_changes: &mut Vec<(String, f32)>,
        locale: &LocaleManager,
    ) {
        match effect_type {
            EffectType::ColorAdjust => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "brightness",
                    &locale.t("param-brightness"),
                    -1.0,
                    1.0,
                    0.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "contrast",
                    &locale.t("param-contrast"),
                    0.0,
                    2.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "saturation",
                    &locale.t("param-saturation"),
                    0.0,
                    2.0,
                    1.0,
                );
            }
            EffectType::Blur => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    20.0,
                    5.0,
                );
            }
            EffectType::ChromaticAberration => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amount",
                    &locale.t("param-amount"),
                    0.0,
                    0.1,
                    0.01,
                );
            }
            EffectType::Glow => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "threshold",
                    &locale.t("param-threshold"),
                    0.0,
                    1.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    30.0,
                    10.0,
                );
            }
            EffectType::Kaleidoscope => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "segments",
                    &locale.t("param-segments"),
                    2.0,
                    16.0,
                    6.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "rotation",
                    &locale.t("param-rotation"),
                    0.0,
                    360.0,
                    0.0,
                );
            }
            EffectType::Pixelate => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "pixel_size",
                    &locale.t("param-pixel-size"),
                    1.0,
                    64.0,
                    8.0,
                );
            }
            EffectType::Vignette => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.0,
                    1.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "softness",
                    &locale.t("param-softness"),
                    0.0,
                    1.0,
                    0.5,
                );
            }
            EffectType::FilmGrain => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amount",
                    &locale.t("param-amount"),
                    0.0,
                    0.5,
                    0.1,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "speed",
                    &locale.t("param-speed"),
                    0.0,
                    5.0,
                    1.0,
                );
            }
            EffectType::Wave => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "frequency",
                    &locale.t("param-frequency"),
                    0.0,
                    50.0,
                    0.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "amplitude",
                    &locale.t("param-amplitude"),
                    0.0,
                    2.0,
                    0.0,
                );
            }
            EffectType::Glitch => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "block_size",
                    &locale.t("param-block-size"),
                    1.0,
                    50.0,
                    0.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "color_shift",
                    &locale.t("param-color-shift"),
                    0.0,
                    20.0,
                    0.0,
                );
            }
            EffectType::RgbSplit => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "offset_x",
                    &locale.t("param-offset-x"),
                    -50.0,
                    50.0,
                    0.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "offset_y",
                    &locale.t("param-offset-y"),
                    -50.0,
                    50.0,
                    0.0,
                );
            }
            EffectType::Mirror => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "center",
                    &locale.t("param-center"),
                    0.0,
                    1.0,
                    0.0,
                );
            }
            EffectType::HueShift => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "hue_shift",
                    &locale.t("param-hue-shift"),
                    0.0,
                    1.0,
                    0.0,
                );
            }
            EffectType::Voronoi => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "scale",
                    &locale.t("param-scale"),
                    1.0,
                    50.0,
                    10.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "offset",
                    &locale.t("param-offset"),
                    0.0,
                    10.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "cell_size",
                    &locale.t("param-cell-size"),
                    0.1,
                    5.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "distortion",
                    &locale.t("param-distortion"),
                    0.0,
                    2.0,
                    0.5,
                );
            }
            EffectType::Tunnel => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "scale",
                    &locale.t("param-scale"),
                    0.1,
                    2.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "rotation",
                    &locale.t("param-rotation"),
                    0.0,
                    5.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "speed",
                    &locale.t("param-speed"),
                    0.0,
                    5.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "distortion",
                    &locale.t("param-distortion"),
                    0.0,
                    2.0,
                    0.5,
                );
            }
            EffectType::Galaxy => {
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "zoom",
                    &locale.t("param-zoom"),
                    0.1,
                    5.0,
                    0.5,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "speed",
                    &locale.t("param-speed"),
                    0.0,
                    2.0,
                    0.2,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "radius",
                    &locale.t("param-radius"),
                    0.1,
                    3.0,
                    1.0,
                );
                Self::render_param_slider_static(
                    ui,
                    parameters,
                    param_changes,
                    "brightness",
                    &locale.t("param-brightness"),
                    0.0,
                    2.0,
                    1.0,
                );
            }
            _ => {
                ui.label(locale.t("no-parameters"));
            }
        }
        let _ = effect_id; // Silence unused warning
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_param_slider_static(
        ui: &mut Ui,
        parameters: &std::collections::HashMap<String, f32>,
        param_changes: &mut Vec<(String, f32)>,
        param_name: &str,
        label: &str,
        min: f32,
        max: f32,
        default_value: f32,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            let old_value = *parameters.get(param_name).unwrap_or(&default_value);
            let mut value = old_value;
            let response = styled_slider(ui, &mut value, min..=max, default_value);

            response.context_menu(|ui| {
                if crate::widgets::custom::hold_to_action_button(
                    ui,
                    "↺ Reset",
                    colors::WARN_COLOR,
                    "Reset",
                ) {
                    value = default_value;
                    ui.close();
                }
            });
            if (value - old_value).abs() > 0.0001 {
                param_changes.push((param_name.to_string(), value));
            }
        });
    }
}
