use super::super::mesh;
use super::super::state::{LayerInspectorViewMode, ModuleCanvas};
use super::capabilities;
use super::common;
use egui::Ui;
use mapmap_core::module::{BlendModeType, LayerType, MaskShape, MaskType, MeshType, ModulePartId};

/// Renders the configuration UI for a `ModulePartType::Layer`.
pub fn render_layer_ui(
    canvas: &mut ModuleCanvas,
    mesh_editor: &mut crate::editors::mesh_editor::MeshEditor,
    last_mesh_edit_id: &mut Option<u64>,
    ui: &mut Ui,
    layer: &mut LayerType,
    part_id: ModulePartId,
) {
    ui.label("📋 Layer:");

    // Helper to render mesh UI
    let mut render_mesh_ui =
        |ui: &mut Ui, mesh: &mut MeshType, id_salt: u64, show_visual_editor: bool| {
            mesh::render_mesh_editor_ui(
                mesh_editor,
                last_mesh_edit_id,
                ui,
                mesh,
                part_id,
                id_salt,
                show_visual_editor,
            );
        };

    let show_mesh_editor = canvas.layer_inspector_view_mode == LayerInspectorViewMode::MeshEditor;

    match layer {
        LayerType::Single {
            id,
            name,
            opacity,
            blend_mode,
            mesh,
            mapping_mode,
        } => {
            ui.label("🔳 Single Layer");
            ui.horizontal(|ui| {
                ui.label("ID:");
                ui.add(egui::DragValue::new(id));
            });
            ui.text_edit_singleline(name);
            ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));

            // Blend mode
            let blend_text = blend_mode
                .as_ref()
                .map(|b| format!("{:?}", b))
                .unwrap_or_else(|| "None".to_string());
            egui::ComboBox::from_id_salt("layer_blend")
                .selected_text(blend_text)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(blend_mode.is_none(), "None").clicked() {
                        *blend_mode = None;
                    }
                    ui.add_enabled_ui(
                        capabilities::is_blend_mode_supported(&BlendModeType::Normal),
                        |ui| {
                            if ui
                                .selectable_label(
                                    matches!(blend_mode, Some(BlendModeType::Normal)),
                                    "Normal",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Normal);
                            }
                        },
                    );
                    ui.add_enabled_ui(
                        capabilities::is_blend_mode_supported(&BlendModeType::Add),
                        |ui| {
                            if ui
                                .selectable_label(
                                    matches!(blend_mode, Some(BlendModeType::Add)),
                                    "Add",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Add);
                            }
                        },
                    );
                    ui.add_enabled_ui(
                        capabilities::is_blend_mode_supported(&BlendModeType::Multiply),
                        |ui| {
                            if ui
                                .selectable_label(
                                    matches!(blend_mode, Some(BlendModeType::Multiply)),
                                    "Multiply",
                                )
                                .clicked()
                            {
                                *blend_mode = Some(BlendModeType::Multiply);
                            }
                        },
                    );
                });
            if !capabilities::is_blend_mode_supported(
                blend_mode.as_ref().unwrap_or(&BlendModeType::Normal),
            ) {
                capabilities::render_unsupported_warning(
                    ui,
                    "Blend modes other than Normal are currently ignored in final render.",
                );
            }

            ui.add_enabled_ui(capabilities::is_mapping_mode_supported(), |ui| {
                ui.checkbox(mapping_mode, "Mapping Mode (Grid)");
            });
            if !capabilities::is_mapping_mode_supported() {
                capabilities::render_unsupported_warning(
                    ui,
                    "Mapping mode grid is currently not end-to-end supported.",
                );
            }

            render_mesh_ui(ui, mesh, *id, show_mesh_editor);
        }
        LayerType::Group {
            name,
            opacity,
            mesh,
            mapping_mode,
            ..
        } => {
            ui.add_enabled_ui(false, |ui| {
                ui.label("📂 Group");
                ui.text_edit_singleline(name);
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
                ui.checkbox(mapping_mode, "Mapping Mode (Grid)");
                render_mesh_ui(ui, mesh, 9999, show_mesh_editor); // Dummy ID
            });
            ui.label(
                egui::RichText::new(
                    "⚠ Group layers are currently unsupported and act like a Single layer.",
                )
                .color(crate::theme::colors::WARN_COLOR)
                .small(),
            );
        }
        LayerType::All { opacity, .. } => {
            ui.add_enabled_ui(false, |ui| {
                ui.label("🎚️ Master");
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
            });
            ui.label(
                egui::RichText::new(
                    "⚠ Master layers are currently unsupported and will not be rendered.",
                )
                .color(crate::theme::colors::WARN_COLOR)
                .small(),
            );
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
                ui.horizontal(|ui| {
                    if ui.button("Select...").clicked() {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                            .pick_file()
                        {
                            *path = picked.display().to_string();
                        }
                    }
                    common::render_info_label(ui, "No mask loaded");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(path).desired_width(120.0));
                    if ui
                        .button("\u{1F4C2}")
                        .on_hover_text("Select Mask File")
                        .clicked()
                    {
                        if let Some(picked) = rfd::FileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
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
                        .selectable_label(matches!(shape, MaskShape::Circle), "Circle")
                        .clicked()
                    {
                        *shape = MaskShape::Circle;
                    }
                    if ui
                        .selectable_label(matches!(shape, MaskShape::Rectangle), "Rectangle")
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
        MaskType::Gradient { angle, softness } => {
            ui.label("\u{1F308} Gradient Mask");
            ui.add(egui::Slider::new(angle, 0.0..=360.0).text("Angle Â°"));
            ui.add(egui::Slider::new(softness, 0.0..=1.0).text("Softness"));
        }
    }
}
