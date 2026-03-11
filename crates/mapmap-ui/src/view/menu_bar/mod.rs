//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

pub mod edit_menu;
pub mod file_menu;
pub mod help_menu;
pub mod toolbar;
pub mod view_menu;

use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Helper for menu items with icons
pub(crate) fn menu_item(
    ui: &mut egui::Ui,
    ui_state: &AppUI,
    text: String,
    icon: Option<AppIcon>,
) -> bool {
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
        .inner_margin(egui::Margin::symmetric(8, 2));

    egui::TopBottomPanel::top("top_panel")
        .frame(frame)
        .show(ctx, |ui| {
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;

            let compact_menu = ctx.content_rect().height() < 900.0;

            // --- Main Menu Bar ---
            egui::MenuBar::new().ui(ui, |ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);
                ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 2.0);

                // --- File Menu ---
                file_menu::show(ui, ui_state, &mut actions, compact_menu);

                // --- Edit Menu ---
                edit_menu::show(ui, ui_state, &mut actions, compact_menu);

                // --- View Menu ---
                view_menu::show(ui, ui_state, &mut actions, compact_menu);

                // --- Help Menu ---
                help_menu::show(ui, ui_state, &mut actions, compact_menu);
            });

            if !compact_menu {
                ui.add_space(2.0);
                ui.separator();
            }
        });

    actions
}
