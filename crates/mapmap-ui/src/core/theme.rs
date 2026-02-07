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
            theme: Theme::Resolume, // Default to Cyber Dark (Resolume)
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

/// Shared color constants for the refined Cyber Dark palette (Deep Navy tones)
pub mod colors {
    use egui::Color32;

    pub const CYAN_ACCENT: Color32 = Color32::from_rgb(0, 229, 255); // Neon Cyan
    pub const MINT_ACCENT: Color32 = Color32::from_rgb(0, 255, 170); // Mint for selection/alt
    pub const WARN_COLOR: Color32 = Color32::from_rgb(255, 170, 0); // Orange
    pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 50, 50); // Red
    pub const DARK_GREY: Color32 = Color32::from_rgb(18, 18, 24); // Main Panel Background (Deep Navy / Cyber Base)
    pub const DARKER_GREY: Color32 = Color32::from_rgb(5, 5, 8); // Window/Deep Background (Almost Black)
    pub const LIGHTER_GREY: Color32 = Color32::from_rgb(40, 40, 45); // Widget Background
    pub const STROKE_GREY: Color32 = Color32::from_rgb(80, 80, 90); // Borders (High Contrast)
}

impl ThemeConfig {
    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = Style::default();
        let visuals = match self.theme {
            Theme::Dark => Self::dark_visuals(),
            Theme::Light => Self::light_visuals(),
            Theme::HighContrast => Self::high_contrast_visuals(),
            Theme::Resolume => Self::resolume_visuals(),
            Theme::Synthwave => Self::synthwave_visuals(),
            Theme::Custom => self.custom_visuals(),
        };

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(self.spacing * 2.0, self.spacing);
        style.spacing.button_padding = egui::vec2(self.spacing * 2.0, self.spacing);

        ctx.set_style(style);
    }

    /// Dark theme visuals (professional video application style)
    fn dark_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Color32::from_rgb(0xEA, 0xEA, 0xEA)); // Primary Text
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, Color32::from_rgb(0xA0, 0xA0, 0xA0)); // Secondary Text
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, Color32::from_rgb(0xA0, 0xA0, 0xA0)); // Secondary Text

        // Backgrounds
        visuals.window_fill = Color32::from_rgb(0x1A, 0x1A, 0x2E); // Dark Navy
        visuals.panel_fill = Color32::from_rgb(0x16, 0x21, 0x3E); // Slightly lighter Navy/Blue
        visuals.faint_bg_color = Color32::from_rgb(0x0F, 0x0F, 0x23); // Darkest Blue
        visuals.extreme_bg_color = Color32::from_rgb(0x0F, 0x0F, 0x23); // Darkest Blue for popups

        // Accents
        let hot_pink = Color32::from_rgb(0xE9, 0x45, 0x60);
        let purple = Color32::from_rgb(0x9D, 0x4E, 0xDD);

        visuals.widgets.hovered.bg_fill = Color32::from_gray(0x40);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, hot_pink);
        visuals.widgets.active.bg_fill = hot_pink;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, Color32::WHITE);
        visuals.widgets.open.bg_fill = Color32::from_rgb(0x16, 0x21, 0x3E);

        visuals.selection.bg_fill = purple.linear_multiply(0.5);
        visuals.selection.stroke = egui::Stroke::new(1.0, purple);
        visuals.hyperlink_color = hot_pink;

        // Window styling
        visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_gray(0x40));

        visuals
    }

    /// Light theme visuals
    fn light_visuals() -> Visuals {
        Visuals {
            dark_mode: false,
            override_text_color: Some(Color32::from_rgb(30, 30, 30)),
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(240, 240, 240),
                    weak_bg_fill: Color32::from_rgb(245, 245, 245),
                    bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)),
                    corner_radius: egui::CornerRadius::same(2),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(230, 230, 230),
                    weak_bg_fill: Color32::from_rgb(235, 235, 235),
                    bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(190, 190, 190)),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(50, 50, 50)),
                    corner_radius: egui::CornerRadius::same(2),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(220, 220, 220),
                    weak_bg_fill: Color32::from_rgb(225, 225, 225),
                    bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(170, 170, 170)),
                    fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(30, 30, 30)),
                    corner_radius: egui::CornerRadius::same(2),
                    expansion: 1.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(60, 120, 200),
                    weak_bg_fill: Color32::from_rgb(70, 130, 210),
                    bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(50, 110, 190)),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(2),
                    expansion: 1.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(235, 235, 235),
                    weak_bg_fill: Color32::from_rgb(240, 240, 240),
                    bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(180, 180, 180)),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 40)),
                    corner_radius: egui::CornerRadius::same(2),
                    expansion: 0.0,
                },
            },
            selection: egui::style::Selection {
                bg_fill: Color32::from_rgb(60, 120, 200).linear_multiply(0.3),
                stroke: egui::Stroke::new(1.0, Color32::from_rgb(50, 110, 190)),
            },
            hyperlink_color: Color32::from_rgb(50, 100, 200),
            faint_bg_color: Color32::from_rgb(250, 250, 250),
            extreme_bg_color: Color32::WHITE,
            code_bg_color: Color32::from_rgb(235, 235, 235),
            warn_fg_color: Color32::from_rgb(200, 120, 0),
            error_fg_color: Color32::from_rgb(200, 0, 0),
            window_fill: Color32::from_rgb(245, 245, 245),
            panel_fill: Color32::from_rgb(240, 240, 240),
            window_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
            ..Default::default()
        }
    }

    /// High contrast visuals for accessibility
    fn high_contrast_visuals() -> Visuals {
        Visuals {
            dark_mode: true,
            override_text_color: Some(Color32::WHITE),
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: Color32::BLACK,
                    weak_bg_fill: Color32::from_rgb(10, 10, 10),
                    bg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(20, 20, 20),
                    weak_bg_fill: Color32::from_rgb(15, 15, 15),
                    bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(200, 200, 200)),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(50, 50, 50),
                    weak_bg_fill: Color32::from_rgb(40, 40, 40),
                    bg_stroke: egui::Stroke::new(3.0, Color32::from_rgb(255, 255, 0)),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 2.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(0, 200, 255),
                    weak_bg_fill: Color32::from_rgb(0, 180, 230),
                    bg_stroke: egui::Stroke::new(3.0, Color32::WHITE),
                    fg_stroke: egui::Stroke::new(3.0, Color32::BLACK),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 2.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(30, 30, 30),
                    weak_bg_fill: Color32::from_rgb(25, 25, 25),
                    bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(220, 220, 220)),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
            },
            selection: egui::style::Selection {
                bg_fill: Color32::from_rgb(0, 200, 255),
                stroke: egui::Stroke::new(3.0, Color32::WHITE),
            },
            hyperlink_color: Color32::from_rgb(100, 200, 255),
            faint_bg_color: Color32::BLACK,
            extreme_bg_color: Color32::BLACK,
            code_bg_color: Color32::from_rgb(20, 20, 20),
            warn_fg_color: Color32::from_rgb(255, 255, 0),
            error_fg_color: Color32::from_rgb(255, 50, 50),
            window_fill: Color32::BLACK,
            panel_fill: Color32::from_rgb(10, 10, 10),
            window_stroke: egui::Stroke::new(3.0, Color32::WHITE),
            ..Default::default()
        }
    }

    /// Custom theme visuals
    fn custom_visuals(&self) -> Visuals {
        if let Some(colors) = &self.custom_colors {
            let bg = Color32::from_rgba_premultiplied(
                colors.background[0],
                colors.background[1],
                colors.background[2],
                colors.background[3],
            );
            let panel = Color32::from_rgba_premultiplied(
                colors.panel_background[0],
                colors.panel_background[1],
                colors.panel_background[2],
                colors.panel_background[3],
            );
            let text = Color32::from_rgba_premultiplied(
                colors.text[0],
                colors.text[1],
                colors.text[2],
                colors.text[3],
            );
            let accent = Color32::from_rgba_premultiplied(
                colors.accent[0],
                colors.accent[1],
                colors.accent[2],
                colors.accent[3],
            );

            let mut visuals = Self::dark_visuals();
            visuals.override_text_color = Some(text);
            visuals.window_fill = bg;
            visuals.panel_fill = panel;
            visuals.widgets.active.bg_fill = accent;
            visuals
        } else {
            Self::dark_visuals()
        }
    }

    /// Resolume Arena-like theme visuals (Cyber Dark: Neutral Dark + Cyan/Mint Accents)
    fn resolume_visuals() -> Visuals {
        Visuals {
            dark_mode: true,
            override_text_color: Some(Color32::from_rgb(240, 240, 240)),
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: colors::DARK_GREY,
                    weak_bg_fill: colors::DARK_GREY,
                    bg_stroke: egui::Stroke::new(1.0, colors::STROKE_GREY),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(180, 180, 180)),
                    corner_radius: egui::CornerRadius::same(0), // Sharp corners
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: colors::LIGHTER_GREY,
                    weak_bg_fill: colors::LIGHTER_GREY,
                    bg_stroke: egui::Stroke::new(1.0, colors::STROKE_GREY),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(220, 220, 220)),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: Color32::from_rgb(60, 60, 60),
                    weak_bg_fill: Color32::from_rgb(60, 60, 60),
                    bg_stroke: egui::Stroke::new(1.0, colors::CYAN_ACCENT), // Cyan border on hover
                    fg_stroke: egui::Stroke::new(1.5, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: colors::CYAN_ACCENT,
                    weak_bg_fill: colors::CYAN_ACCENT,
                    bg_stroke: egui::Stroke::new(1.0, colors::CYAN_ACCENT),
                    fg_stroke: egui::Stroke::new(2.0, Color32::BLACK), // Black text on Cyan
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: colors::DARK_GREY,
                    weak_bg_fill: colors::DARK_GREY,
                    bg_stroke: egui::Stroke::new(1.0, colors::STROKE_GREY),
                    fg_stroke: egui::Stroke::new(1.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(0),
                    expansion: 0.0,
                },
            },
            selection: egui::style::Selection {
                bg_fill: colors::CYAN_ACCENT.linear_multiply(0.2),
                stroke: egui::Stroke::new(1.0, colors::CYAN_ACCENT),
            },
            hyperlink_color: colors::CYAN_ACCENT,
            faint_bg_color: colors::DARKER_GREY,
            extreme_bg_color: colors::DARKER_GREY,
            code_bg_color: colors::DARK_GREY,
            warn_fg_color: colors::WARN_COLOR,   // Orange
            error_fg_color: colors::ERROR_COLOR, // Red
            window_fill: colors::DARKER_GREY,    // Darker window background for hierarchy
            panel_fill: colors::DARK_GREY,       // Lighter panel background
            window_stroke: egui::Stroke::new(1.0, colors::STROKE_GREY),
            ..Default::default()
        }
    }

    /// Synthwave visuals (Neon/Retro)
    fn synthwave_visuals() -> Visuals {
        let neon_pink = Color32::from_rgb(255, 0, 179);
        let neon_cyan = Color32::from_rgb(0, 243, 255);
        let deep_purple = Color32::from_rgb(20, 13, 33);
        let mid_purple = Color32::from_rgb(37, 22, 58);
        let light_purple = Color32::from_rgb(58, 35, 90);

        Visuals {
            dark_mode: true,
            override_text_color: Some(Color32::from_rgb(240, 230, 255)),
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: deep_purple,
                    weak_bg_fill: deep_purple,
                    bg_stroke: egui::Stroke::new(1.0, light_purple),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(180, 160, 200)),
                    corner_radius: egui::CornerRadius::same(4),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: mid_purple,
                    weak_bg_fill: mid_purple,
                    bg_stroke: egui::Stroke::new(1.0, light_purple),
                    fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 255)),
                    corner_radius: egui::CornerRadius::same(4),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: light_purple,
                    weak_bg_fill: light_purple,
                    bg_stroke: egui::Stroke::new(1.0, neon_cyan),
                    fg_stroke: egui::Stroke::new(1.5, neon_cyan),
                    corner_radius: egui::CornerRadius::same(4),
                    expansion: 1.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: neon_pink.linear_multiply(0.5),
                    weak_bg_fill: neon_pink.linear_multiply(0.5),
                    bg_stroke: egui::Stroke::new(1.0, neon_pink),
                    fg_stroke: egui::Stroke::new(2.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(4),
                    expansion: 1.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: mid_purple,
                    weak_bg_fill: mid_purple,
                    bg_stroke: egui::Stroke::new(1.0, light_purple),
                    fg_stroke: egui::Stroke::new(1.0, Color32::WHITE),
                    corner_radius: egui::CornerRadius::same(4),
                    expansion: 0.0,
                },
            },
            selection: egui::style::Selection {
                bg_fill: neon_pink.linear_multiply(0.4),
                stroke: egui::Stroke::new(1.0, neon_pink),
            },
            hyperlink_color: neon_cyan,
            faint_bg_color: deep_purple,
            extreme_bg_color: Color32::BLACK,
            code_bg_color: mid_purple,
            warn_fg_color: Color32::from_rgb(255, 180, 0),
            error_fg_color: Color32::from_rgb(255, 50, 50),
            window_fill: deep_purple.linear_multiply(0.95),
            panel_fill: deep_purple,
            window_stroke: egui::Stroke::new(1.0, neon_cyan.linear_multiply(0.5)),
            ..Default::default()
        }
    }
}

/// Theme picker widget
pub fn theme_picker(ui: &mut egui::Ui, theme: &mut Theme) -> bool {
    let mut changed = false;

    ui.label("Theme:");
    ui.horizontal(|ui| {
        changed |= ui.selectable_value(theme, Theme::Dark, "Dark").clicked();
        changed |= ui.selectable_value(theme, Theme::Light, "Light").clicked();
        changed |= ui
            .selectable_value(theme, Theme::HighContrast, "High Contrast")
            .clicked();
        changed |= ui
            .selectable_value(theme, Theme::Resolume, "Resolume")
            .clicked();
        changed |= ui
            .selectable_value(theme, Theme::Synthwave, "Synthwave")
            .clicked();
    });

    changed
}
