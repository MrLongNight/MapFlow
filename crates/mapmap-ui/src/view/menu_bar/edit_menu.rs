use super::menu_item;
use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

pub fn show(ui: &mut egui::Ui, ui_state: &AppUI, actions: &mut Vec<UIAction>) {
    ui.menu_button(ui_state.i18n.t("menu-edit"), |ui| {
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-edit-undo"),
            Some(AppIcon::ArrowLeft),
        ) {
            actions.push(UIAction::Undo);
            ui.close();
        }
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-edit-redo"),
            Some(AppIcon::ArrowRight),
        ) {
            actions.push(UIAction::Redo);
            ui.close();
        }
        ui.separator();
        if ui.button(ui_state.i18n.t("menu-edit-cut")).clicked() {
            actions.push(UIAction::Cut);
            ui.close();
        }
        if ui.button(ui_state.i18n.t("menu-edit-copy")).clicked() {
            actions.push(UIAction::Copy);
            ui.close();
        }
        if ui.button(ui_state.i18n.t("menu-edit-paste")).clicked() {
            actions.push(UIAction::Paste);
            ui.close();
        }
        if menu_item(
            ui,
            ui_state,
            ui_state.i18n.t("menu-edit-delete"),
            Some(AppIcon::Remove),
        ) {
            actions.push(UIAction::Delete);
            ui.close();
        }
        ui.separator();
        if ui.button(ui_state.i18n.t("menu-edit-select-all")).clicked() {
            actions.push(UIAction::SelectAll);
            ui.close();
        }
    });
}
