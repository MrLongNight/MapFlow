//! Egui-based shortcuts configuration panel

use crate::theme::colors;
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

    /// Helper to draw a keycap-style pill
    fn draw_key_pill(ui: &mut Ui, text: &str, is_conflict: bool) {
        let stroke_color = if is_conflict {
            colors::ERROR_COLOR
        } else {
            colors::STROKE_GREY
        };

        let bg_color = if is_conflict {
            colors::ERROR_COLOR.linear_multiply(0.2)
        } else {
            colors::LIGHTER_GREY
        };

        // Use Frame::default() as none() is deprecated
        egui::Frame::default()
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .corner_radius(4.0)
            // Use explicit struct construction to avoid type issues with symmetric()
            .inner_margin(egui::Margin {
                left: 6,
                right: 6,
                top: 2,
                bottom: 2,
            })
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(text)
                        .color(egui::Color32::WHITE)
                        .size(12.0)
                        .strong(),
                );
            });
    }

    /// Helper to render the full shortcut visualization
    fn render_shortcut_visuals(
        ui: &mut Ui,
        shortcut: &mapmap_control::shortcuts::Shortcut,
        is_conflict: bool,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            if shortcut.modifiers.ctrl {
                Self::draw_key_pill(ui, "Ctrl", is_conflict);
            }
            if shortcut.modifiers.alt {
                Self::draw_key_pill(ui, "Alt", is_conflict);
            }
            if shortcut.modifiers.shift {
                Self::draw_key_pill(ui, "Shift", is_conflict);
            }
            if shortcut.modifiers.meta {
                #[cfg(target_os = "macos")]
                Self::draw_key_pill(ui, "Cmd", is_conflict);
                #[cfg(not(target_os = "macos"))]
                Self::draw_key_pill(ui, "Win", is_conflict);
            }

            // Clean up key name
            let key_text = format!("{:?}", shortcut.key);
            let display_text = if key_text.starts_with("Key")
                && key_text.len() == 4
                && key_text.chars().nth(3).unwrap().is_numeric()
            {
                &key_text[3..]
            } else {
                &key_text
            };

            Self::draw_key_pill(ui, display_text, is_conflict);
        });
    }

    /// Render the shortcuts panel
    pub fn render(&mut self, ui: &mut Ui, locale: &LocaleManager, key_bindings: &mut KeyBindings) {
        let shortcuts_clone = key_bindings.get_shortcuts().to_vec();

        // Container Frame
        egui::Frame::default()
            .fill(colors::DARK_GREY) // Replaced PANEL_FILL with DARK_GREY
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(locale.t("shortcuts-panel-title"))
                            .color(egui::Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(locale.t("shortcuts-reset-defaults")).clicked() {
                            key_bindings.reset_to_defaults();
                            self.detect_conflicts(key_bindings);
                        }
                    });
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("shortcuts_grid")
                        .num_columns(3)
                        .striped(true)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            // Headers with Theme Color
                            ui.label(
                                egui::RichText::new(locale.t("shortcuts-header-action"))
                                    .strong()
                                    .color(colors::CYAN_ACCENT),
                            );
                            ui.label(
                                egui::RichText::new(locale.t("shortcuts-header-shortcut"))
                                    .strong()
                                    .color(colors::CYAN_ACCENT),
                            );
                            ui.label(""); // Edit button column
                            ui.end_row();

                            for (i, shortcut) in shortcuts_clone.iter().enumerate() {
                                ui.label(
                                    egui::RichText::new(&shortcut.description)
                                        .color(egui::Color32::WHITE),
                                );

                                let is_conflict = self.conflicts.contains(&i);

                                // Use new visual helper
                                Self::render_shortcut_visuals(ui, shortcut, is_conflict);

                                ui.horizontal(|ui| {
                                    if is_conflict {
                                        ui.label("⚠️").on_hover_text(
                                            "This shortcut is used by another action.",
                                        );
                                    }

                                    if ui.small_button("✏").clicked() {
                                        self.editing_shortcut_index = Some(i);
                                        self.show_conflict_warning = false;
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });
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
                            key,
                            pressed: true,
                            ..
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
