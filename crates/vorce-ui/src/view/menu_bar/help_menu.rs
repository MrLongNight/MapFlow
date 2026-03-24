use super::menu_item;
use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

pub fn show(ui: &mut egui::Ui, ui_state: &AppUI, actions: &mut Vec<UIAction>, compact_menu: bool) {
    let menu_help_label = ui_state.i18n.t("menu-help");
    let top_label = if compact_menu {
        "❓"
    } else {
        &menu_help_label
    };
    let response = ui.menu_button(top_label, |ui| {
        if ui.button(ui_state.i18n.t("menu-help-docs")).clicked() {
            actions.push(UIAction::OpenDocs);
            ui.close();
        }
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-help-about"),
            Some(AppIcon::InfoCircle),
        ) {
            actions.push(UIAction::OpenAbout);
            ui.close();
        }
        if ui.button(ui_state.i18n.t("menu-help-license")).clicked() {
            actions.push(UIAction::OpenLicense);
            ui.close();
        }
        ui.separator();
        ui.menu_button("Language", |ui| {
            if ui.button("English").clicked() {
                actions.push(UIAction::SetLanguage("en".to_string()));
                ui.close();
            }
            if ui.button("Deutsch").clicked() {
                actions.push(UIAction::SetLanguage("de".to_string()));
                ui.close();
            }
        });
    });

    if compact_menu {
        response.response.on_hover_text(menu_help_label);
    }
}
