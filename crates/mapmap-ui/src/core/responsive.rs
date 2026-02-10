//! Responsive Layout Helper für MapFlow UI
//!
//! Dieses Modul berechnet UI-Größen dynamisch basierend auf der Viewport-Größe.

use egui::Context;

/// Breakpoints für verschiedene Screen-Größen
pub const BREAKPOINT_MOBILE: f32 = 768.0;
pub const BREAKPOINT_TABLET: f32 = 1024.0;
pub const BREAKPOINT_DESKTOP: f32 = 1920.0;

/// Helper für viewport-basierte Layout-Berechnungen
#[derive(Debug, Clone)]
pub struct ResponsiveLayout {
    /// Aktuelle Viewport-Größe
    pub viewport_size: egui::Vec2,
}

impl ResponsiveLayout {
    /// Erstellt neue ResponsiveLayout-Instanz basierend auf aktuellem Context
    pub fn new(ctx: &Context) -> Self {
        let size = ctx.screen_rect().size();
        Self {
            viewport_size: size,
        }
    }

    /// Berechnet Panel-Breite als Prozent des Viewports
    ///
    /// # Argumente
    /// * `percent` - Prozentanteil (0.0 - 1.0)
    ///
    /// # Rückgabe
    /// Berechnete Breite, begrenzt zwischen 150.0 und 800.0 Pixeln
    pub fn panel_width(&self, percent: f32) -> f32 {
        (self.viewport_size.x * percent).clamp(150.0, 800.0)
    }

    /// Skaliert eine Font-Größe basierend auf Viewport-Breite
    ///
    /// # Argumente
    /// * `base_size` - Basis-Font-Größe bei 1920px Breite
    ///
    /// # Rückgabe
    /// Skalierte Font-Größe (Faktor: 0.7x - 1.5x)
    pub fn scale_font(&self, base_size: f32) -> f32 {
        let scale_factor = (self.viewport_size.x / BREAKPOINT_DESKTOP).clamp(0.7, 1.5);
        base_size * scale_factor
    }

    /// Prüft ob Viewport kompakt ist (Mobile/kleine Tablets)
    pub fn is_compact(&self) -> bool {
        self.viewport_size.x < BREAKPOINT_TABLET || self.viewport_size.y < 768.0
    }

    /// Prüft ob Viewport mobil ist
    pub fn is_mobile(&self) -> bool {
        self.viewport_size.x < BREAKPOINT_MOBILE
    }

    /// Gibt empfohlene Sidebar-Breite zurück
    pub fn sidebar_width(&self) -> f32 {
        if self.is_mobile() {
            self.viewport_size.x * 0.9 // 90% bei Mobile
        } else if self.is_compact() {
            self.panel_width(0.25) // 25% bei Tablets
        } else {
            self.panel_width(0.15) // 15% bei Desktop
        }
    }

    /// Gibt empfohlene maximale Sidebar-Breite zurück
    pub fn sidebar_max_width(&self) -> f32 {
        (self.viewport_size.x * 0.4).max(400.0)
    }

    /// Berechnet dynamische Window-Größe
    ///
    /// # Argumente
    /// * `default_width` - Standard-Breite bei Desktop
    /// * `default_height` - Standard-Höhe bei Desktop
    pub fn window_size(&self, default_width: f32, default_height: f32) -> [f32; 2] {
        if self.is_compact() {
            [self.viewport_size.x * 0.9, self.viewport_size.y * 0.8]
        } else {
            [default_width, default_height]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_width_clamps() {
        let layout = ResponsiveLayout {
            viewport_size: egui::Vec2::new(1920.0, 1080.0),
        };

        assert_eq!(layout.panel_width(0.1), 192.0);
        assert_eq!(layout.panel_width(0.05), 150.0); // Min clamp
        assert_eq!(layout.panel_width(0.5), 800.0); // Max clamp
    }

    #[test]
    fn test_is_compact() {
        let desktop = ResponsiveLayout {
            viewport_size: egui::Vec2::new(1920.0, 1080.0),
        };
        assert!(!desktop.is_compact());

        let tablet = ResponsiveLayout {
            viewport_size: egui::Vec2::new(1000.0, 768.0),
        };
        assert!(tablet.is_compact());
    }
}
