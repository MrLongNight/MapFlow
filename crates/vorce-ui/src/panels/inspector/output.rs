use super::{inspector_row, inspector_section, inspector_value};
use crate::widgets::panel::render_panel_header;
use egui::Ui;
use vorce_core::OutputConfig;

/// Show output properties inspector
pub fn show_output_inspector(ui: &mut Ui, output: &OutputConfig) {
    // Header
    render_panel_header(ui, &format!("🖥 {}", output.name), |_| {});
    ui.add_space(8.0);

    // Resolution section
    inspector_section(ui, "Resolution", true, |ui| {
        inspector_row(ui, "Size", |ui| {
            inspector_value(
                ui,
                &format!("{}x{}", output.resolution.0, output.resolution.1),
            );
        });
    });

    ui.add_space(4.0);

    // Canvas Region section
    inspector_section(ui, "Canvas Region", true, |ui| {
        let region = &output.canvas_region;
        inspector_row(ui, "Position", |ui| {
            inspector_value(ui, &format!("({:.0}, {:.0})", region.x, region.y));
        });

        inspector_row(ui, "Size", |ui| {
            inspector_value(ui, &format!("{:.0}x{:.0}", region.width, region.height));
        });
    });

    ui.add_space(4.0);

    // Edge Blend indicator
    inspector_section(ui, "Edge Blend", false, |ui| {
        let eb = &output.edge_blend;
        inspector_row(ui, "Left", |ui| {
            inspector_value(ui, &format!("{:.0}px", eb.left.width * 100.0));
        });
        inspector_row(ui, "Right", |ui| {
            inspector_value(ui, &format!("{:.0}px", eb.right.width * 100.0));
        });
        inspector_row(ui, "Top", |ui| {
            inspector_value(ui, &format!("{:.0}px", eb.top.width * 100.0));
        });
        inspector_row(ui, "Bottom", |ui| {
            inspector_value(ui, &format!("{:.0}px", eb.bottom.width * 100.0));
        });
    });
}
