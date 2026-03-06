use egui::Ui;
use crate::i18n::LocaleManager;
use mapmap_core::{Layer, Transform};
use super::InspectorAction;

pub fn render_layer_inspector(
    ui: &mut Ui,
    layer: &Layer,
    transform: &Transform,
    _index: usize,
    i18n: &LocaleManager,
) -> Option<InspectorAction> {
    let mut action = None;

    ui.vertical(|ui| {
        // Name & Type info
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&layer.name).strong().size(16.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("LAYER").color(egui::Color32::GRAY));
            });
        });
        
        ui.add_space(10.0);
        ui.separator();
        
        // --- TRANSFORM SECTION ---
        egui::CollapsingHeader::new(i18n.t("inspector-transform"))
            .default_open(true)
            .show(ui, |ui| {
                let mut new_transform = *transform;
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Position:");
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.position.x).speed(1.0).prefix("X: ")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.position.y).speed(1.0).prefix("Y: ")).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.scale.x).speed(0.01).prefix("X: ")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.scale.y).speed(0.01).prefix("Y: ")).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.rotation.x).speed(1.0).prefix("X: ").suffix("°")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.rotation.y).speed(1.0).prefix("Y: ").suffix("°")).changed();
                    changed |= ui.add(egui::DragValue::new(&mut new_transform.rotation.z).speed(1.0).prefix("Z: ").suffix("°")).changed();
                });

                if changed {
                    action = Some(InspectorAction::UpdateTransform(layer.id, new_transform));
                }
            });

        ui.add_space(10.0);

        // --- APPEARANCE SECTION ---
        egui::CollapsingHeader::new(i18n.t("inspector-appearance"))
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Opacity:");
                    let mut opacity = layer.opacity;
                    if ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0)).changed() {
                        action = Some(InspectorAction::UpdateOpacity(layer.id, opacity));
                    }
                });
            });

        ui.add_space(10.0);

        // --- MESH EDITOR SECTION ---
        egui::CollapsingHeader::new("Mesh Warp Editor")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Click to edit mesh points in the preview window.");
                if ui.button("Reset Mesh").clicked() {
                    // Reset logic
                }
            });
    });

    action
}
