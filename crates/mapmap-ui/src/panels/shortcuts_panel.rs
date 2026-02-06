//! Egui-based shortcuts configuration panel

use crate::LocaleManager;
use egui::Ui;
use mapmap_control::shortcuts::KeyBindings;
use std::collections::HashSet;

/// Panel for viewing and configuring keyboard shortcuts
#[derive(Default)]
pub struct ShortcutsPanel {
    editing_shortcut_index: Option<usize>,
    conflicts: HashSet<usize>,
    show_conflict_warning: bool,
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
        ui.heading(locale.t("shortcuts-panel-title"));

        if ui.button(locale.t("shortcuts-reset-defaults")).clicked() {
            key_bindings.reset_to_defaults();
            self.detect_conflicts(key_bindings);
        }

        ui.separator();

        let shortcuts_clone = key_bindings.get_shortcuts().to_vec();

        egui::Grid::new("shortcuts_grid")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                ui.label(locale.t("shortcuts-header-action"));
                ui.label(locale.t("shortcuts-header-shortcut"));
                ui.label(""); // For edit button
                ui.end_row();

                for (i, shortcut) in shortcuts_clone.iter().enumerate() {
                    ui.label(&shortcut.description);

                    let is_conflict = self.conflicts.contains(&i);
                    let resp = ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing.x = 4.0;

                        if shortcut.modifiers.ctrl {
                            Self::draw_key_pill(ui, "Ctrl", is_conflict);
                        }
                        if shortcut.modifiers.shift {
                            Self::draw_key_pill(ui, "Shift", is_conflict);
                        }
                        if shortcut.modifiers.alt {
                            Self::draw_key_pill(ui, "Alt", is_conflict);
                        }
                        if shortcut.modifiers.meta {
                            #[cfg(target_os = "macos")]
                            Self::draw_key_pill(ui, "Cmd", is_conflict);
                            #[cfg(not(target_os = "macos"))]
                            Self::draw_key_pill(ui, "Win", is_conflict);
                        }

                        let key_str = format!("{:?}", shortcut.key);
                        let key_display = key_str.strip_prefix("Key").unwrap_or(&key_str);
                        let key_display = match key_display {
                            "ArrowUp" => "Up",
                            "ArrowDown" => "Down",
                            "ArrowLeft" => "Left",
                            "ArrowRight" => "Right",
                            "PageUp" => "PgUp",
                            "PageDown" => "PgDn",
                            "Escape" => "Esc",
                            "Delete" => "Del",
                            "Insert" => "Ins",
                            "Backspace" => "Back",
                            k => k,
                        };

                        Self::draw_key_pill(ui, key_display, is_conflict);
                    });

                    if is_conflict {
                        resp.response.on_hover_text("This shortcut is used by another action.");
                    }

                    if ui.button(locale.t("shortcuts-edit")).clicked() {
                        self.editing_shortcut_index = Some(i);
                        self.show_conflict_warning = false;
                    }
                    ui.end_row();
                }
            });

        if let Some(index) = self.editing_shortcut_index {
            let mut new_shortcut_key = None;

            let mut is_open = true;
            egui::Window::new(locale.t("shortcuts-edit-dialog-title"))
                .open(&mut is_open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(locale.t("shortcuts-edit-dialog-prompt"));
                    ui.label(locale.t("shortcuts-edit-dialog-cancel"));

                    if self.show_conflict_warning {
                        ui.colored_label(
                            egui::Color32::RED,
                            locale.t("shortcuts-edit-dialog-conflict-warning"),
                        );
                    }

                    let input = ui.input(|i| i.clone());

                    if input.key_pressed(egui::Key::Escape) {
                        self.editing_shortcut_index = None;
                    } else if let Some(key) = input.events.iter().find_map(|e| match e {
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => Some(key),
                        _ => None,
                    }) {
                        let modifiers = input.modifiers;
                        if let Some(mapmap_key) = to_mapmap_key(*key) {
                            new_shortcut_key = Some((mapmap_key, to_mapmap_modifiers(modifiers)));
                        }
                    }
                });

            if !is_open {
                self.editing_shortcut_index = None;
            }

            if let Some((new_key, new_modifiers)) = new_shortcut_key {
                let context = shortcuts_clone[index].context;
                if key_bindings.is_key_bound(new_key, &new_modifiers, context) {
                    self.show_conflict_warning = true;
                } else {
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

    fn draw_key_pill(ui: &mut Ui, text: &str, is_conflict: bool) {
        use crate::core::theme::colors;
        let text_color = if is_conflict {
            colors::ERROR_COLOR
        } else {
            egui::Color32::WHITE
        };

        let bg_color = if is_conflict {
            colors::ERROR_COLOR.linear_multiply(0.2)
        } else {
            colors::LIGHTER_GREY
        };

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .stroke(egui::Stroke::new(1.0, colors::STROKE_GREY))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(text)
                        .color(text_color)
                        .strong()
                        .monospace(),
                );
            });
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
        // egui doesn't have separate keys for these, so we map them from other keys
        // LeftBracket => Some(Mk::LeftBracket),
        // RightBracket => Some(Mk::RightBracket),
        // Semicolon => Some(Mk::Semicolon),
        // Quote => Some(Mk::Quote),
        // Comma => Some(Mk::Comma),
        // Period => Some(Mk::Period),
        // Slash => Some(Mk::Slash),
        // Backslash => Some(Mk::Backslash),
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
