//! Egui-based shortcuts configuration panel

use crate::core::theme::colors;
use crate::LocaleManager;
use egui::{RichText, ScrollArea, TextEdit, Ui};
use mapmap_control::shortcuts::KeyBindings;
use std::collections::HashSet;

/// Panel for viewing and configuring keyboard shortcuts
#[derive(Default)]
pub struct ShortcutsPanel {
    editing_shortcut_index: Option<usize>,
    conflicts: HashSet<usize>,
    show_conflict_warning: bool,
    search_filter: String,
}

impl ShortcutsPanel {
    pub fn detect_conflicts(&mut self, key_bindings: &KeyBindings) {
        self.conflicts.clear();
        let shortcuts = key_bindings.get_shortcuts();
        for i in 0..shortcuts.len() {
            for j in (i + 1)..shortcuts.len() {
                if shortcuts[i].key == shortcuts[j].key
                    && shortcuts[i].modifiers == shortcuts[j].modifiers
                    && (shortcuts[i].context == shortcuts[j].context
                        || shortcuts[i].context
                            == mapmap_control::shortcuts::ShortcutContext::Global
                        || shortcuts[j].context
                            == mapmap_control::shortcuts::ShortcutContext::Global)
                {
                    self.conflicts.insert(i);
                    self.conflicts.insert(j);
                }
            }
        }
    }

    /// Render the shortcuts panel
    pub fn render(&mut self, ui: &mut Ui, locale: &LocaleManager, key_bindings: &mut KeyBindings) {
        // --- Header Section ---
        ui.horizontal(|ui| {
            ui.heading(locale.t("shortcuts-panel-title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(locale.t("shortcuts-reset-defaults")).clicked() {
                    key_bindings.reset_to_defaults();
                    self.detect_conflicts(key_bindings);
                    self.search_filter.clear();
                }
            });
        });

        ui.add_space(8.0);

        // --- Search Bar ---
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(TextEdit::singleline(&mut self.search_filter).hint_text("Search shortcuts..."));
            if !self.search_filter.is_empty() && ui.button("✖").clicked() {
                self.search_filter.clear();
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        let shortcuts_clone = key_bindings.get_shortcuts().to_vec();

        // --- Filter and Group Shortcuts ---
        let filter_lower = self.search_filter.to_lowercase();
        let filtered_indices: Vec<usize> = shortcuts_clone
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                if filter_lower.is_empty() {
                    return true;
                }
                s.description.to_lowercase().contains(&filter_lower)
                    || s.to_shortcut_string()
                        .to_lowercase()
                        .contains(&filter_lower)
            })
            .map(|(i, _)| i)
            .collect();

        if filtered_indices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No shortcuts found").weak().italics());
            });
        } else {
            // --- Shortcuts List ---
            ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .num_columns(3)
                    .spacing([20.0, 8.0])
                    .striped(true)
                    .min_col_width(150.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(locale.t("shortcuts-header-action")).strong());
                        ui.label(RichText::new(locale.t("shortcuts-header-shortcut")).strong());
                        ui.label(""); // Edit button column
                        ui.end_row();

                        for index in filtered_indices {
                            let shortcut = &shortcuts_clone[index];
                            let is_conflict = self.conflicts.contains(&index);

                            // Description
                            ui.label(&shortcut.description);

                            // Shortcut Key Display
                            let shortcut_text = shortcut.to_shortcut_string();
                            let text_color = if is_conflict {
                                colors::ERROR_COLOR
                            } else {
                                colors::CYAN_ACCENT
                            };

                            let key_label = ui.label(
                                RichText::new(if shortcut_text.is_empty() {
                                    "(None)"
                                } else {
                                    &shortcut_text
                                })
                                .color(text_color)
                                .monospace(),
                            );

                            if is_conflict {
                                key_label.clone().on_hover_text(
                                    "⚠️ Conflict: This shortcut is used by multiple actions.",
                                );
                            }

                            // Edit Button
                            if ui
                                .add(egui::Button::new(locale.t("shortcuts-edit")))
                                .clicked()
                            {
                                self.editing_shortcut_index = Some(index);
                                self.show_conflict_warning = false;
                            }
                            ui.end_row();
                        }
                    });
            });
        }

        // --- Edit Dialog ---
        if let Some(index) = self.editing_shortcut_index {
            let mut new_shortcut_key = None;
            let mut is_open = true;

            let shortcut_desc = &shortcuts_clone[index].description;

            egui::Window::new(locale.t("shortcuts-edit-dialog-title"))
                .open(&mut is_open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    ui.heading(format!("Edit: {}", shortcut_desc));
                    ui.separator();

                    ui.label(locale.t("shortcuts-edit-dialog-prompt"));

                    ui.group(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("Press any key combination...")
                                    .strong()
                                    .color(colors::CYAN_ACCENT),
                            );
                            ui.label("(Press ESC to cancel)"); // Removed Backspace instruction
                            ui.add_space(10.0);
                        });
                    });

                    if self.show_conflict_warning {
                        ui.colored_label(
                            colors::ERROR_COLOR,
                            format!("⚠️ {}", locale.t("shortcuts-edit-dialog-conflict-warning")),
                        );
                    }

                    ui.separator();
                    if ui
                        .button(locale.t("shortcuts-edit-dialog-cancel"))
                        .clicked()
                    {
                        self.editing_shortcut_index = None;
                    }

                    // Input Handling
                    let input = ui.input(|i| i.clone());

                    if input.key_pressed(egui::Key::Escape) {
                        self.editing_shortcut_index = None;
                    } else if let Some(key) = input.events.iter().find_map(|e| match e {
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => Some(key),
                        _ => None,
                    }) {
                        // Ignore modifier-only presses
                        if !matches!(key, egui::Key::PageUp | egui::Key::PageDown) {
                            let modifiers = input.modifiers;
                            if let Some(mapmap_key) = to_mapmap_key(*key) {
                                new_shortcut_key =
                                    Some(Some((mapmap_key, to_mapmap_modifiers(modifiers))));
                            }
                        }
                    }
                });

            if !is_open {
                self.editing_shortcut_index = None;
            }

            if let Some(Some((new_key, new_modifiers))) = new_shortcut_key {
                let context = shortcuts_clone[index].context;
                // Check conflict
                if key_bindings.is_key_bound(new_key, &new_modifiers, context) {
                    // If binding to same key as current, it's fine (no-op)
                    let current = &shortcuts_clone[index];
                    if current.key == new_key && current.modifiers == new_modifiers {
                        self.editing_shortcut_index = None;
                    } else {
                        self.show_conflict_warning = true;
                    }
                } else {
                    // Apply
                    let mut shortcut = shortcuts_clone[index].clone();
                    shortcut.key = new_key;
                    shortcut.modifiers = new_modifiers;
                    key_bindings.update_shortcut(index, shortcut);
                    self.detect_conflicts(key_bindings);
                    self.editing_shortcut_index = None;
                }
            }
        }
    }
}

fn to_mapmap_key(key: egui::Key) -> Option<mapmap_control::shortcuts::Key> {
    use egui::Key::*;
    use mapmap_control::shortcuts::Key as Mk;

    match key {
        A => Some(Mk::A),
        B => Some(Mk::B),
        C => Some(Mk::C),
        D => Some(Mk::D),
        E => Some(Mk::E),
        F => Some(Mk::F),
        G => Some(Mk::G),
        H => Some(Mk::H),
        I => Some(Mk::I),
        J => Some(Mk::J),
        K => Some(Mk::K),
        L => Some(Mk::L),
        M => Some(Mk::M),
        N => Some(Mk::N),
        O => Some(Mk::O),
        P => Some(Mk::P),
        Q => Some(Mk::Q),
        R => Some(Mk::R),
        S => Some(Mk::S),
        T => Some(Mk::T),
        U => Some(Mk::U),
        V => Some(Mk::V),
        W => Some(Mk::W),
        X => Some(Mk::X),
        Y => Some(Mk::Y),
        Z => Some(Mk::Z),
        Num0 => Some(Mk::Key0),
        Num1 => Some(Mk::Key1),
        Num2 => Some(Mk::Key2),
        Num3 => Some(Mk::Key3),
        Num4 => Some(Mk::Key4),
        Num5 => Some(Mk::Key5),
        Num6 => Some(Mk::Key6),
        Num7 => Some(Mk::Key7),
        Num8 => Some(Mk::Key8),
        Num9 => Some(Mk::Key9),
        F1 => Some(Mk::F1),
        F2 => Some(Mk::F2),
        F3 => Some(Mk::F3),
        F4 => Some(Mk::F4),
        F5 => Some(Mk::F5),
        F6 => Some(Mk::F6),
        F7 => Some(Mk::F7),
        F8 => Some(Mk::F8),
        F9 => Some(Mk::F9),
        F10 => Some(Mk::F10),
        F11 => Some(Mk::F11),
        F12 => Some(Mk::F12),
        Space => Some(Mk::Space),
        Enter => Some(Mk::Enter),
        Escape => Some(Mk::Escape),
        Tab => Some(Mk::Tab),
        Backspace => Some(Mk::Backspace),
        Delete => Some(Mk::Delete),
        Insert => Some(Mk::Insert),
        Home => Some(Mk::Home),
        End => Some(Mk::End),
        PageUp => Some(Mk::PageUp),
        PageDown => Some(Mk::PageDown),
        ArrowUp => Some(Mk::ArrowUp),
        ArrowDown => Some(Mk::ArrowDown),
        ArrowLeft => Some(Mk::ArrowLeft),
        ArrowRight => Some(Mk::ArrowRight),
        Minus => Some(Mk::Minus),
        Plus => Some(Mk::Plus),
        _ => None,
    }
}

fn to_mapmap_modifiers(modifiers: egui::Modifiers) -> mapmap_control::shortcuts::Modifiers {
    mapmap_control::shortcuts::Modifiers {
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        shift: modifiers.shift,
        meta: modifiers.mac_cmd || modifiers.command,
    }
}
