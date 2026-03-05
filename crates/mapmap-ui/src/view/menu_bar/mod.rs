//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

pub mod file_menu;
pub mod view_menu;
pub mod edit_menu;
pub mod help_menu;
pub mod toolbar;

use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Helper for menu items with icons
pub(crate) fn menu_item(ui: &mut egui::Ui, ui_state: &AppUI, text: String, icon: Option<AppIcon>) -> bool {
    if let Some(mgr) = &ui_state.icon_manager {
        if let Some(icon) = icon {
            if let Some(img) = mgr.image(icon, 14.0) {
                return ui.add(egui::Button::image_and_text(img, text)).clicked();
            }
        }
    }
    ui.button(text).clicked()
}

/// Renders the main menu bar and returns any action triggered.
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI) -> Vec<UIAction> {
    let mut actions = vec![];

    // Custom frame for modern look
    let frame = egui::Frame::default()
        .fill(ctx.style().visuals.window_fill())
        .inner_margin(egui::Margin::symmetric(16, 8));

    egui::TopBottomPanel::top("top_panel")
        .frame(frame)
        .show(ctx, |ui| {
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;

            // --- Main Menu Bar ---
            egui::MenuBar::new().ui(ui, |ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                // --- File Menu ---
                file_menu::show(ui, ui_state, &mut actions);

                // --- Edit Menu ---
                edit_menu::show(ui, ui_state, &mut actions);

                // --- View Menu ---
                view_menu::show(ui, ui_state, &mut actions);

                // --- Help Menu ---
                help_menu::show(ui, ui_state, &mut actions);
            });

            ui.add_space(4.0);

            // --- Toolbar ---
            if ui_state.show_toolbar {
                toolbar::show(ui, ui_state, &mut actions);
            }

            ui.add_space(4.0);
            ui.separator();
        });

    actions
}
