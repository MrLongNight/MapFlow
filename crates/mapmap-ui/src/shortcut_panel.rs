//! Keyboard Shortcut Configuration Panel
//!
//! UI Panel for viewing and customizing keyboard shortcuts.

use egui::{self, Key as EguiKey, RichText, ScrollArea};
use mapmap_control::shortcuts::{DefaultShortcuts, Key, Modifiers, Shortcut, ShortcutContext};

use crate::i18n::LocaleManager;

/// Actions that can be triggered by the shortcut panel
#[derive(Debug, Clone)]
pub enum ShortcutAction {
    /// Set a new shortcut for an action
    SetShortcut(usize, Key, Modifiers),
    /// Reset a shortcut to default
    ResetShortcut(usize),
    /// Reset all shortcuts to defaults
    ResetAllShortcuts,
}

/// Shortcut panel state
#[derive(Default)]
pub struct ShortcutPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Current shortcuts (editable copy)
    shortcuts: Vec<Shortcut>,
    /// Search filter
    search_filter: String,
    /// Currently recording shortcut for this index
    recording_index: Option<usize>,
    /// Filter by context
    context_filter: Option<ShortcutContext>,
    /// Initialized flag
    initialized: bool,
}

impl ShortcutPanel {
    /// Create a new shortcut panel
    pub fn new() -> Self {
        Self {
            visible: false,
            shortcuts: Vec::new(),
            search_filter: String::new(),
            recording_index: None,
            context_filter: None,
            initialized: false,
        }
    }

    /// Initialize with default shortcuts
    fn initialize(&mut self) {
        if !self.initialized {
            self.shortcuts = DefaultShortcuts::all();
            self.initialized = true;
        }
    }

    /// Get current shortcuts
    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    /// Show the shortcut panel
    pub fn show(&mut self, ctx: &egui::Context, i18n: &LocaleManager) -> Vec<ShortcutAction> {
        self.initialize();
        let mut actions = Vec::new();

        if !self.visible {
            return actions;
        }

        let mut is_open = self.visible;
        egui::Window::new(i18n.t("panel-shortcuts"))
            .default_size([600.0, 500.0])
            .resizable(true)
            .open(&mut is_open)
            .show(ctx, |ui| {
                self.render_ui(ui, i18n, &mut actions);
            });
        self.visible = is_open;

        actions
    }

    /// Render the panel UI
    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        i18n: &LocaleManager,
        actions: &mut Vec<ShortcutAction>,
    ) {
        // Header with search and filter
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(16.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.search_filter)
                    .hint_text(i18n.t("hint-search-shortcuts"))
                    .desired_width(200.0),
            );

            ui.separator();

            // Context filter
            ui.label(i18n.t("label-context"));
            egui::ComboBox::from_id_source("context_filter")
                .selected_text(match &self.context_filter {
                    None => i18n.t("filter-all"),
                    Some(ctx) => context_display_name(ctx).to_string(),
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.context_filter.is_none(), i18n.t("filter-all"))
                        .clicked()
                    {
                        self.context_filter = None;
                    }
                    for ctx in [
                        ShortcutContext::Global,
                        ShortcutContext::MainWindow,
                        ShortcutContext::Editor,
                        ShortcutContext::Timeline,
                    ] {
                        if ui
                            .selectable_label(
                                self.context_filter == Some(ctx),
                                context_display_name(&ctx),
                            )
                            .clicked()
                        {
                            self.context_filter = Some(ctx);
                        }
                    }
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n.t("btn-reset-all")).clicked() {
                    self.shortcuts = DefaultShortcuts::all();
                    actions.push(ShortcutAction::ResetAllShortcuts);
                }
            });
        });

        ui.separator();

        // Recording indicator
        if let Some(idx) = self.recording_index {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⏺").size(16.0).color(egui::Color32::RED));
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        i18n.t("msg-recording"),
                        self.shortcuts
                            .get(idx)
                            .map(|s| s.description.as_str())
                            .unwrap_or("?")
                    ))
                    .strong(),
                );
                if ui.button(i18n.t("btn-cancel")).clicked() {
                    self.recording_index = None;
                }
            });
            ui.separator();

            // Capture keyboard input
            if let Some(key) = self.capture_key_press(ui.ctx()) {
                let modifiers = self.get_current_modifiers(ui.ctx());
                if let Some(shortcut) = self.shortcuts.get_mut(idx) {
                    shortcut.key = key;
                    shortcut.modifiers = modifiers.clone();
                    actions.push(ShortcutAction::SetShortcut(idx, key, modifiers));
                }
                self.recording_index = None;
            }
        }

        // Shortcuts table
        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("shortcuts_grid")
                .num_columns(4)
                .striped(true)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    // Header
                    ui.label(RichText::new(i18n.t("col-action")).strong());
                    ui.label(RichText::new(i18n.t("col-shortcut")).strong());
                    ui.label(RichText::new(i18n.t("col-context")).strong());
                    ui.label(RichText::new(i18n.t("col-actions")).strong());
                    ui.end_row();

                    let search_lower = self.search_filter.to_lowercase();

                    let mut pending_record = None;
                    let mut pending_reset = None;

                    for (idx, shortcut) in self.shortcuts.iter().enumerate() {
                        // Apply search filter
                        if !self.search_filter.is_empty()
                            && !shortcut.description.to_lowercase().contains(&search_lower)
                        {
                            continue;
                        }

                        if let Some(ref ctx_filter) = self.context_filter {
                            if shortcut.context != *ctx_filter {
                                continue;
                            }
                        }

                        // Action description
                        ui.label(&shortcut.description);

                        // Shortcut key combination
                        let shortcut_str = shortcut.to_shortcut_string();
                        let is_recording = self.recording_index == Some(idx);

                        if is_recording {
                            ui.label(RichText::new("...").italics().color(egui::Color32::YELLOW));
                        } else {
                            ui.label(RichText::new(&shortcut_str).monospace());
                        }

                        // Context
                        ui.label(context_display_name(&shortcut.context));

                        // Actions
                        ui.horizontal(|ui| {
                            if ui
                                .small_button("✏")
                                .on_hover_text(i18n.t("tip-edit-shortcut"))
                                .clicked()
                            {
                                pending_record = Some(idx);
                            }
                            if ui
                                .small_button("↺")
                                .on_hover_text(i18n.t("tip-reset-shortcut"))
                                .clicked()
                            {
                                pending_reset = Some(idx);
                            }
                        });

                        ui.end_row();
                    }

                    if let Some(idx) = pending_record {
                        self.recording_index = Some(idx);
                    }
                    if let Some(idx) = pending_reset {
                        // Reset to default
                        let defaults = DefaultShortcuts::all();
                        if let Some(default) = defaults.get(idx) {
                            if let Some(current) = self.shortcuts.get_mut(idx) {
                                current.key = default.key;
                                current.modifiers = default.modifiers.clone();
                                actions.push(ShortcutAction::ResetShortcut(idx));
                            }
                        }
                    }
                });
        });
    }

    /// Capture a key press from the current frame
    fn capture_key_press(&self, ctx: &egui::Context) -> Option<Key> {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key, pressed: true, ..
                } = event
                {
                    return egui_key_to_key(*key);
                }
            }
            None
        })
    }

    /// Get current modifier state
    fn get_current_modifiers(&self, ctx: &egui::Context) -> Modifiers {
        ctx.input(|i| {
            let egui_mods = i.modifiers;
            Modifiers {
                ctrl: egui_mods.ctrl,
                alt: egui_mods.alt,
                shift: egui_mods.shift,
                meta: egui_mods.mac_cmd || egui_mods.command,
            }
        })
    }
}

/// Convert egui key to our Key type
fn egui_key_to_key(egui_key: EguiKey) -> Option<Key> {
    Some(match egui_key {
        EguiKey::A => Key::A,
        EguiKey::B => Key::B,
        EguiKey::C => Key::C,
        EguiKey::D => Key::D,
        EguiKey::E => Key::E,
        EguiKey::F => Key::F,
        EguiKey::G => Key::G,
        EguiKey::H => Key::H,
        EguiKey::I => Key::I,
        EguiKey::J => Key::J,
        EguiKey::K => Key::K,
        EguiKey::L => Key::L,
        EguiKey::M => Key::M,
        EguiKey::N => Key::N,
        EguiKey::O => Key::O,
        EguiKey::P => Key::P,
        EguiKey::Q => Key::Q,
        EguiKey::R => Key::R,
        EguiKey::S => Key::S,
        EguiKey::T => Key::T,
        EguiKey::U => Key::U,
        EguiKey::V => Key::V,
        EguiKey::W => Key::W,
        EguiKey::X => Key::X,
        EguiKey::Y => Key::Y,
        EguiKey::Z => Key::Z,
        EguiKey::Num0 => Key::Key0,
        EguiKey::Num1 => Key::Key1,
        EguiKey::Num2 => Key::Key2,
        EguiKey::Num3 => Key::Key3,
        EguiKey::Num4 => Key::Key4,
        EguiKey::Num5 => Key::Key5,
        EguiKey::Num6 => Key::Key6,
        EguiKey::Num7 => Key::Key7,
        EguiKey::Num8 => Key::Key8,
        EguiKey::Num9 => Key::Key9,
        EguiKey::F1 => Key::F1,
        EguiKey::F2 => Key::F2,
        EguiKey::F3 => Key::F3,
        EguiKey::F4 => Key::F4,
        EguiKey::F5 => Key::F5,
        EguiKey::F6 => Key::F6,
        EguiKey::F7 => Key::F7,
        EguiKey::F8 => Key::F8,
        EguiKey::F9 => Key::F9,
        EguiKey::F10 => Key::F10,
        EguiKey::F11 => Key::F11,
        EguiKey::F12 => Key::F12,
        EguiKey::Space => Key::Space,
        EguiKey::Enter => Key::Enter,
        EguiKey::Escape => Key::Escape,
        EguiKey::Tab => Key::Tab,
        EguiKey::Backspace => Key::Backspace,
        EguiKey::Delete => Key::Delete,
        EguiKey::Insert => Key::Insert,
        EguiKey::Home => Key::Home,
        EguiKey::End => Key::End,
        EguiKey::PageUp => Key::PageUp,
        EguiKey::PageDown => Key::PageDown,
        EguiKey::ArrowUp => Key::ArrowUp,
        EguiKey::ArrowDown => Key::ArrowDown,
        EguiKey::ArrowLeft => Key::ArrowLeft,
        EguiKey::ArrowRight => Key::ArrowRight,
        _ => return None,
    })
}

/// Get display name for a shortcut context
fn context_display_name(ctx: &ShortcutContext) -> &'static str {
    match ctx {
        ShortcutContext::Global => "Global",
        ShortcutContext::MainWindow => "Main Window",
        ShortcutContext::OutputWindow => "Output Window",
        ShortcutContext::Editor => "Editor",
        ShortcutContext::Timeline => "Timeline",
        ShortcutContext::LayerPanel => "Layer Panel",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_panel_creation() {
        let panel = ShortcutPanel::new();
        assert!(!panel.visible);
        assert!(panel.shortcuts.is_empty());
    }
}
