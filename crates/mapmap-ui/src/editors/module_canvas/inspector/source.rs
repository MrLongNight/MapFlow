use super::state::ModuleCanvas;
use crate::UIAction;
use egui::{Ui, Color32};
use mapmap_core::module::{SourceType, ModuleId, ModulePartId, BevyCameraMode};
use crate::widgets::{styled_slider, styled_drag_value};

/// Renders the UI for a source part
pub fn render_source_ui(
    canvas: &mut ModuleCanvas,
    ui: &mut Ui,
    part_id: ModulePartId,
    source: &mut SourceType,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    shared_media_ids: &[String],
) {
    ui.horizontal(|ui| {
        ui.label("Source Type:");
        // ... (Source type picker logic will be moved here)
    });

    ui.separator();

    match source {
        SourceType::MediaFile { path, .. } | SourceType::VideoUni { path, .. } => {
            ui.label(format!("📁 File: {}", path));
            if ui.button("Select Media").clicked() {
                actions.push(UIAction::PickMediaFile(module_id, part_id, "".to_string()));
            }
            // ... (Media controls logic)
        }
        SourceType::ImageUni { path, .. } => {
            ui.label(format!("🖼 Image: {}", path));
            if ui.button("Select Image").clicked() {
                actions.push(UIAction::PickMediaFile(module_id, part_id, "".to_string()));
            }
        }
        SourceType::VideoMulti { shared_id, .. } | SourceType::ImageMulti { shared_id, .. } => {
            ui.label("🔗 Shared Source");
            ui.text_edit_singleline(shared_id);
        }
        SourceType::Shader { name, .. } => {
            ui.label("🎨 Shader Source");
            ui.text_edit_singleline(name);
        }
        SourceType::LiveInput { device_id } => {
            ui.label("📹 Live Input");
            ui.add(egui::Slider::new(device_id, 0..=10).text("Device ID"));
        }
        SourceType::NdiInput { source_name } => {
            ui.label("📡 NDI Input");
            let name = source_name.clone().unwrap_or_else(|| "None".to_string());
            ui.label(format!("Connected: {}", name));
        }
        SourceType::BevyAtmosphere { turbidity, rayleigh, sun_position, .. } => {
            ui.label("☁ Atmosphere");
            ui.add(egui::Slider::new(turbidity, 0.0..=10.0).text("Turbidity"));
            ui.add(egui::Slider::new(rayleigh, 0.0..=10.0).text("Rayleigh"));
        }
        SourceType::Bevy3DShape { shape_type, position, rotation, color, .. } => {
            ui.label("🧊 3D Shape");
            ui.label(format!("Type: {:?}", shape_type));
            // Transform UI
        }
        _ => {
            ui.label("Bevy / Other Source");
        }
    }
}

/// Helper for common media controls (opacity, transform, etc.)
pub fn render_common_controls(
    ui: &mut Ui,
    opacity: &mut f32,
    // ... other params
) {
    ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
    // ...
}
