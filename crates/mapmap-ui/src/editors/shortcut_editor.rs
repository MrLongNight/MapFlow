//! UI Panel for editing keyboard shortcuts (Phase 7)
use crate::i18n::LocaleManager;
use egui::{Context, RichText, Ui};
use mapmap_control::shortcuts::Shortcut;

/// Represents the state of the shortcut editor UI.
enum EditorState {
    /// The default, idle state.
    Idle,
    /// The UI is waiting for a key press to bind it to a specific action.
    /// The `usize` holds the index of the shortcut being edited.
    WaitingForKey(usize),
}

/// A UI panel for viewing and editing keyboard shortcuts.
pub struct ShortcutEditor {
    /// Controls whether the shortcut editor window is visible.
    pub visible: bool,
    /// The internal state of the editor, used to manage interactions like key binding.
    state: EditorState,
    /// A temporary store for the key bindings being edited.
    bindings: Vec<Shortcut>,
}

impl ShortcutEditor {
    /// Creates a new `ShortcutEditor`, initialized with default key bindings.
    pub fn new() -> Self {
        Self {
            visible: false,
            state: EditorState::Idle,
            bindings: mapmap_control::shortcuts::DefaultShortcuts::all(),
        }
    }

    /// Renders the shortcut editor window and handles its logic.
    ///
    /// # Arguments
    /// * `ctx` - The egui context.
    /// * `i18n` - The localization manager for translating UI text.
    pub fn show(&mut self, ctx: &Context, i18n: &LocaleManager) {
        if !self.visible {
            return;
        }

        let mut is_open = self.visible;
        egui::Window::new(i18n.t("panel-shortcut-editor"))
            .open(&mut is_open)
            .default_size([600.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.ui(ui, i18n);
            });
        self.visible = is_open;
    }

    /// Renders the main content of the shortcut editor.
    fn ui(&mut self, ui: &mut Ui, i18n: &LocaleManager) {
        ui.heading(i18n.t("header-keyboard-shortcuts"));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(i18n.t("btn-reset-defaults")).clicked() {
                self.bindings = mapmap_control::shortcuts::DefaultShortcuts::all();
            }
            let _ = ui.button(i18n.t("btn-import"));
            let _ = ui.button(i18n.t("btn-export"));
        });

        ui.separator();

        self.render_shortcut_table(ui, i18n);
    }

    /// Renders the table of shortcuts.
    fn render_shortcut_table(&mut self, ui: &mut Ui, i18n: &LocaleManager) {
        use egui_extras::{Column, TableBuilder};

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto()) // Action
            .column(Column::auto()) // Shortcut
            .column(Column::remainder()) // Description
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(i18n.t("header-action"));
                });
                header.col(|ui| {
                    ui.strong(i18n.t("header-shortcut"));
                });
                header.col(|ui| {
                    ui.strong(i18n.t("header-description"));
                });
            })
            .body(|mut body| {
                for (index, shortcut) in self.bindings.iter_mut().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            let action_name = format!("{:?}", shortcut.action);
                            ui.label(action_name);
                        });

                        row.col(|ui| {
                            let text = match self.state {
                                EditorState::WaitingForKey(editing_index)
                                    if editing_index == index =>
                                {
                                    RichText::new("Press any key...")
                                        .color(ui.visuals().warn_fg_color)
                                }
                                _ => RichText::new(shortcut.to_shortcut_string()),
                            };

                            if ui.button(text).clicked() {
                                self.state = EditorState::WaitingForKey(index);
                            }
                        });

                        row.col(|ui| {
                            ui.label(&shortcut.description);
                        });
                    });
                }
            });
    }
}

impl Default for ShortcutEditor {
    fn default() -> Self {
        Self::new()
    }
}
