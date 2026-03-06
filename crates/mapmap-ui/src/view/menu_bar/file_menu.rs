use crate::icons::AppIcon;
use crate::{AppUI, UIAction};
use super::menu_item;

pub fn show(ui: &mut egui::Ui, ui_state: &AppUI, actions: &mut Vec<UIAction>) {
    ui.menu_button(ui_state.i18n.t("menu-file"), |ui| {
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-file-new-project"),
            Some(AppIcon::Add),
        ) {
            actions.push(UIAction::NewProject);
            ui.close();
        }
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-file-open-project"),
            Some(AppIcon::LockOpen),
        ) {
            actions.push(UIAction::LoadProject(String::new()));
            ui.close();
        }

        // Recent files submenu
        let recent_files = ui_state.recent_files.clone();
        if !recent_files.is_empty() {
            ui.menu_button(ui_state.i18n.t("menu-file-open-recent"), |ui| {
                for path in &recent_files {
                    if ui.button(path).clicked() {
                        actions.push(UIAction::LoadRecentProject(path.clone()));
                        ui.close();
                    }
                }
            });
        }

        ui.separator();

        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-file-save-project"),
            Some(AppIcon::FloppyDisk),
        ) {
            actions.push(UIAction::SaveProject(String::new()));
            ui.close();
        }
        if ui.button(ui_state.i18n.t("menu-file-save-as")).clicked() {
            actions.push(UIAction::SaveProjectAs);
            ui.close();
        }
        if ui.button(ui_state.i18n.t("menu-file-export")).clicked() {
            actions.push(UIAction::Export);
            ui.close();
        }

        ui.separator();

        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-file-settings"),
            Some(AppIcon::Cog),
        ) {
            actions.push(UIAction::OpenSettings);
            ui.close();
        }

        ui.separator();

        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-file-exit"),
            Some(AppIcon::ButtonStop),
        ) {
            actions.push(UIAction::Exit);
            ui.close();
        }
    });
}
