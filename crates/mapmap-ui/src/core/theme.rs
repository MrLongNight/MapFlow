//! Phase 6: Theme System
//!
//! Professional theme support with dark, light, and high-contrast modes.
//! Includes accessibility features and customizable color schemes.

use egui::{Color32, Style, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    /// Dark theme (default for professional video applications)
    #[default]
    Dark,
    /// Light theme
    Light,
    /// High contrast for accessibility
    HighContrast,
    /// Custom theme
    Custom,
    /// Resolume Arena-like theme
    Resolume,
    /// Synthwave (Neon/Retro)
    Synthwave,
    /// Cyberpunk (Black & Yellow)
    Cyber,
    /// Midnight (Deep Black)
    Midnight,
    /// Purple Majesty
    Purple,
    /// Pink Paradise
    Pink,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub theme: Theme,
    pub custom_colors: Option<CustomColors>,
    pub font_size: f32,
    pub spacing: f32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Resolume,
            custom_colors: None,
            font_size: 14.0,
            spacing: 4.0,
        }
    }
}

/// Custom color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub background: [u8; 4],
    pub panel_background: [u8; 4],
    pub text: [u8; 4],
    pub accent: [u8; 4],
    pub warning: [u8; 4],
    pub error: [u8; 4],
}

/// Shared color constants
pub mod colors {
    use egui::Color32;

    pub const CYAN_ACCENT: Color32 = Color32::from_rgb(0, 229, 255);
    pub const MINT_ACCENT: Color32 = Color32::from_rgb(0, 255, 170);
    pub const WARN_COLOR: Color32 = Color32::from_rgb(255, 170, 0);
    pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 50, 50);
    pub const DARK_GREY: Color32 = Color32::from_rgb(18, 18, 24);
    pub const DARKER_GREY: Color32 = Color32::from_rgb(5, 5, 8);
    pub const LIGHTER_GREY: Color32 = Color32::from_rgb(40, 40, 45);
    pub const STROKE_GREY: Color32 = Color32::from_rgb(60, 60, 70);
}

impl ThemeConfig {
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = Style::default();
        let visuals = match self.theme {
            Theme::Dark => Self::dark_visuals(),
            Theme::Light => Self::light_visuals(),
            Theme::HighContrast => Self::high_contrast_visuals(),
            Theme::Resolume => Self::resolume_visuals(),
            Theme::Synthwave => Self::synthwave_visuals(),
            Theme::Cyber => Self::cyber_visuals(),
            Theme::Midnight => Self::midnight_visuals(),
            Theme::Purple => Self::purple_visuals(),
            Theme::Pink => Self::pink_visuals(),
            Theme::Custom => self.custom_visuals(),
        };

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);

        ctx.set_style(style);
    }

    fn dark_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.window_fill = Color32::from_rgb(0x1A, 0x1A, 0x2E);
        visuals.panel_fill = Color32::from_rgb(0x16, 0x21, 0x3E);
        visuals
    }

    fn light_visuals() -> Visuals {
        Visuals::light()
    }

    fn high_contrast_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.window_fill = Color32::BLACK;
        visuals.panel_fill = Color32::from_rgb(10, 10, 10);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, Color32::WHITE);
        visuals
    }

    fn custom_visuals(&self) -> Visuals {
        Self::dark_visuals()
    }

    fn resolume_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.window_fill = colors::DARKER_GREY;
        visuals.panel_fill = colors::DARK_GREY;
        visuals.selection.bg_fill = colors::CYAN_ACCENT.linear_multiply(0.2);
        visuals.selection.stroke = egui::Stroke::new(1.0, colors::CYAN_ACCENT);
        visuals
    }

    fn synthwave_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        let deep_purple = Color32::from_rgb(20, 13, 33);
        visuals.window_fill = deep_purple;
        visuals.panel_fill = deep_purple;
        visuals
    }

    fn cyber_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.window_fill = Color32::BLACK;
        visuals.panel_fill = Color32::from_rgb(15, 15, 15);
        visuals.widgets.active.bg_fill = Color32::from_rgb(255, 215, 0); // Gold/Yellow
        visuals.selection.bg_fill = Color32::from_rgb(255, 215, 0).linear_multiply(0.3);
        visuals
    }

    fn midnight_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.window_fill = Color32::from_rgb(5, 5, 5);
        visuals.panel_fill = Color32::BLACK;
        visuals
    }

    fn purple_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        let deep_purple = Color32::from_rgb(30, 0, 50);
        visuals.window_fill = deep_purple;
        visuals.panel_fill = Color32::from_rgb(45, 0, 75);
        visuals.widgets.active.bg_fill = Color32::from_rgb(180, 0, 255);
        visuals
    }

    fn pink_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        let deep_pink = Color32::from_rgb(50, 0, 30);
        visuals.window_fill = deep_pink;
        visuals.panel_fill = Color32::from_rgb(75, 0, 45);
        visuals.widgets.active.bg_fill = Color32::from_rgb(255, 0, 180);
        visuals
    }
}

pub fn theme_picker(ui: &mut egui::Ui, theme: &mut Theme) -> bool {
    let mut changed = false;
    ui.label("Theme:");
    egui::ComboBox::from_id_salt("theme_picker")
        .selected_text(format!("{:?}", theme))
        .show_ui(ui, |ui| {
            for t in [
                Theme::Dark,
                Theme::Light,
                Theme::HighContrast,
                Theme::Resolume,
                Theme::Synthwave,
                Theme::Cyber,
                Theme::Midnight,
                Theme::Purple,
                Theme::Pink,
            ] {
                changed |= ui.selectable_value(theme, t, format!("{:?}", t)).clicked();
            }
        });
    changed
}
