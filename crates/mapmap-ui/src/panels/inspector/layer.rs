use egui::{Color32, Ui};
use mapmap_core::{Layer, Transform};
use mapmap_core::module::{BlendModeType, LayerType, MaskShape, MaskType, ModulePartId, MeshType};
use crate::icons::IconManager;
use crate::theme::colors;
use crate::widgets::custom::styled_slider;
use crate::widgets::panel::render_panel_header;
use super::{inspector_section, inspector_row, inspector_value, InspectorAction};
use crate::editors::module_canvas::state::ModuleCanvas;
use crate::editors::module_canvas::mesh;

/// Show layer properties inspector
#[allow(clippy::too_many_arguments)]
pub fn show_layer_inspector(
    ui: &mut Ui,
    layer: &Layer,
    transform: &Transform,
    index: usize,
    is_learning: bool,
    last_active_element: Option<&String>,
    last_active_time: Option<std::time::Instant>,
    global_actions: &mut Vec<crate::UIAction>,
) -> Option<InspectorAction> {
    let mut action = None;

    // Layer header with icon
    render_panel_header(ui, &format!("📦 {}", layer.name), |_| {});
    ui.add_space(8.0);

    // Transform section
    inspector_section(ui, "Transform", true, |ui| {
        inspector_row(ui, "Position", |ui| {
            inspector_value(
                ui,
                &format!("({:.1}, {:.1})", transform.position.x, transform.position.y),
            );
        });

        inspector_row(ui, "Scale", |ui| {
            inspector_value(
                ui,
                &format!("({:.2}, {:.2})", transform.scale.x, transform.scale.y),
            );
        });

        inspector_row(ui, "Rotation", |ui| {
            inspector_value(ui, &format!("{:.1}°", transform.rotation.z.to_degrees()));
        });
    });

    ui.add_space(4.0);

    // Blending section
    inspector_section(ui, "Blending", true, |ui| {
        inspector_row(ui, "Opacity", |ui| {
            let mut opacity = layer.opacity;
            let response = styled_slider(ui, &mut opacity, 0.0..=1.0, 1.0);
            if response.changed() {
                action = Some(InspectorAction::UpdateOpacity(layer.id, opacity));
            }

            // MIDI Learn
            use mapmap_control::target::ControlTarget;
            crate::AppUI::midi_learn_helper(
                ui,
                &response,
                ControlTarget::LayerOpacity(index as u32),
                is_learning,
                last_active_element,
                last_active_time,
                global_actions,
            );
        });

        inspector_row(ui, "Blend Mode", |ui| {
            inspector_value(ui, &format!("{:?}", layer.blend_mode));
        });
    });

    ui.add_space(4.0);

    // Layer state
    inspector_section(ui, "State", true, |ui| {
        inspector_row(ui, "Visible", |ui| {
            let (text, color) = if layer.visible {
                ("Visible", colors::CYAN_ACCENT)
            } else {
                ("Hidden", Color32::GRAY)
            };
            ui.label(egui::RichText::new(text).color(color).strong());
        });

        inspector_row(ui, "Solo", |ui| {
            if layer.solo {
                ui.label(
                    egui::RichText::new("ACTIVE")
                        .color(colors::MINT_ACCENT)
                        .strong(),
                );
            } else {
                ui.label(egui::RichText::new("—").color(Color32::GRAY));
            }
        });

        inspector_row(ui, "Bypass", |ui| {
            if layer.bypass {
                ui.label(
                    egui::RichText::new("ACTIVE")
                        .color(colors::WARN_COLOR)
                        .strong(),
                );
            } else {
                ui.label(egui::RichText::new("—").color(Color32::GRAY));
            }
        });
    });

    action
}

/// Renders the configuration UI for a `ModulePartType::Layer`.
pub fn render_layer_ui(canvas: &mut ModuleCanvas, ui: &mut Ui, layer: &mut LayerType, part_id: ModulePartId) {
    ui.label("📋 Layer:");

    // Helper to render mesh UI
    let mut render_mesh_ui = |ui: &mut Ui, mesh: &mut MeshType, id_salt: u64| {
        mesh::render_mesh_editor_ui(canvas, ui, mesh, part_id, id_salt);
    };

    match layer {
        LayerType::Single { id, name, opacity, blend_mode, mesh, mapping_mode } => {
            ui.label("🔳 Single Layer");
            ui.horizontal(|ui| { ui.label("ID:"); ui.add(egui::DragValue::new(id)); });
            ui.text_edit_singleline(name);
            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));

            // Blend mode
            let blend_text = blend_mode.as_ref().map(|b| format!("{:?}", b)).unwrap_or_else(|| "None".to_string());
            egui::ComboBox::from_id_salt("layer_blend").selected_text(blend_text).show_ui(ui, |ui| {
                if ui.selectable_label(blend_mode.is_none(), "None").clicked() { *blend_mode = None; }
                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Normal)), "Normal").clicked() { *blend_mode = Some(BlendModeType::Normal); }
                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Add)), "Add").clicked() { *blend_mode = Some(BlendModeType::Add); }
                if ui.selectable_label(matches!(blend_mode, Some(BlendModeType::Multiply)), "Multiply").clicked() { *blend_mode = Some(BlendModeType::Multiply); }
            });

            ui.checkbox(mapping_mode, "Mapping Mode (Grid)");

            render_mesh_ui(ui, mesh, *id);
        }
        LayerType::Group { name, opacity, mesh, mapping_mode, .. } => {
            ui.label("📂 Group");
            ui.text_edit_singleline(name);
            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
            ui.checkbox(mapping_mode, "Mapping Mode (Grid)");
            render_mesh_ui(ui, mesh, 9999); // Dummy ID
        }
        LayerType::All { opacity, .. } => {
            ui.label("🎚️ Master");
            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
        }
    }
}

/// Renders the configuration UI for a `ModulePartType::Mask`.
pub fn render_mask_ui(ui: &mut Ui, mask: &mut MaskType) {
    ui.label("Mask Type:");
    match mask {
        MaskType::File { path } => {
            ui.label("📁 Mask File");
            if path.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui.add(egui::Button::new("\u{1F4C2} Select Mask File")
                        .min_size(egui::vec2(150.0, 30.0)))
                        .clicked()
                    {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter(
                                "Image",
                                &[
                                    "png", "jpg", "jpeg", "webp",
                                    "bmp",
                                ],
                            )
                            .pick_file()
                        {
                            *path = picked.display().to_string();
                        }
                    }
                    ui.label(egui::RichText::new("No mask loaded").weak());
                    ui.add_space(10.0);
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(path)
                            .desired_width(120.0),
                    );
                    if ui.button("\u{1F4C2}").on_hover_text("Select Mask File").clicked() {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter(
                                "Image",
                                &[
                                    "png", "jpg", "jpeg", "webp",
                                    "bmp",
                                ],
                            )
                            .pick_file()
                        {
                            *path = picked.display().to_string();
                        }
                    }
                });
            }
        }
        MaskType::Shape(shape) => {
            ui.label("\u{1F537} Shape Mask");
            egui::ComboBox::from_id_salt("mask_shape")
                .selected_text(format!("{:?}", shape))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            matches!(shape, MaskShape::Circle),
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
                            matches!(shape, MaskShape::Star),
                            "Star",
                        )
                        .clicked()
                    {
                        *shape = MaskShape::Star;
                    }
                    if ui
                        .selectable_label(
                            matches!(shape, MaskShape::Ellipse),
                            "Ellipse",
                        )
                        .clicked()
                    {
                        *shape = MaskShape::Ellipse;
                    }
                });
        }
        MaskType::Gradient { angle, softness } => {
            ui.label("\u{1F308} Gradient Mask");
            ui.add(
                egui::Slider::new(angle, 0.0..=360.0)
                    .text("Angle Â°"),
            );
            ui.add(
                egui::Slider::new(softness, 0.0..=1.0)
                    .text("Softness"),
            );
        }
    }
}

