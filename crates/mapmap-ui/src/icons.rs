//! Icon loader module for Ultimate Colors icons
//!
//! Loads SVG icons from assets/icons and provides them as egui images.

use egui::{ColorImage, Context, TextureHandle};
use std::collections::HashMap;
use std::path::Path;

/// Icon identifiers for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppIcon {
    VideoPlayer,
    AudioFile,
    MusicNote,
    ButtonStop,
    VinylRecord,
    VideoFile,
    Cog,
    MagicWand,
    ImageFile,
    Screen,
    Monitor,
    FloppyDisk,
    ArrowLeft,
    ArrowRight,
    Pencil,
    AppWindow,
    InfoCircle,
    Hand,
}

impl AppIcon {
    /// Get the file name for this icon
    pub fn file_name(&self) -> &'static str {
        match self {
            Self::VideoPlayer => "ultimate_video_player.svg",
            Self::AudioFile => "ultimate_audio_file.svg",
            Self::MusicNote => "ultimate_music_note.svg",
            Self::ButtonStop => "ultimate_button_stop.svg",
            Self::VinylRecord => "ultimate_vinyl_record.svg",
            Self::VideoFile => "ultimate_video_file.svg",
            Self::Cog => "ultimate_cog.svg",
            Self::MagicWand => "ultimate_magic_wand.svg",
            Self::ImageFile => "ultimate_image_file.svg",
            Self::Screen => "ultimate_screen.svg",
            Self::Monitor => "ultimate_monitor.svg",
            Self::FloppyDisk => "ultimate_floppy_disk.svg",
            Self::ArrowLeft => "ultimate_arrow_left.svg",
            Self::ArrowRight => "ultimate_arrow_right.svg",
            Self::Pencil => "ultimate_pencil.svg",
            Self::AppWindow => "ultimate_app_window.svg",
            Self::InfoCircle => "ultimate_info_circle.svg",
            Self::Hand => "ultimate_hand.svg",
        }
    }

    /// Get all available icons
    pub fn all() -> &'static [AppIcon] {
        &[
            Self::VideoPlayer,
            Self::AudioFile,
            Self::MusicNote,
            Self::ButtonStop,
            Self::VinylRecord,
            Self::VideoFile,
            Self::Cog,
            Self::MagicWand,
            Self::ImageFile,
            Self::Screen,
            Self::Monitor,
            Self::FloppyDisk,
            Self::ArrowLeft,
            Self::ArrowRight,
            Self::Pencil,
            Self::AppWindow,
            Self::InfoCircle,
            Self::Hand,
        ]
    }
}

/// Icon manager that loads and caches icons
pub struct IconManager {
    icons: HashMap<AppIcon, TextureHandle>,
    icon_size: u32,
}

impl IconManager {
    /// Create a new icon manager and load all icons
    pub fn new(ctx: &Context, assets_dir: &Path, icon_size: u32) -> Self {
        let mut icons = HashMap::new();
        let icons_dir = assets_dir.join("icons");

        for icon in AppIcon::all() {
            let icon_path = icons_dir.join(icon.file_name());
            if let Some(texture) = Self::load_svg_icon(ctx, &icon_path, icon_size) {
                icons.insert(*icon, texture);
            } else {
                tracing::warn!("Failed to load icon: {:?} from {:?}", icon, icon_path);
            }
        }

        tracing::info!("Loaded {} icons", icons.len());

        Self { icons, icon_size }
    }

    /// Load an SVG icon as a texture
    fn load_svg_icon(ctx: &Context, path: &Path, size: u32) -> Option<TextureHandle> {
        // Read SVG file
        let svg_data = std::fs::read_to_string(path).ok()?;

        // Parse SVG using resvg
        let opt = usvg::Options::default();
        let fontdb = usvg::fontdb::Database::new();
        let tree = usvg::Tree::from_str(&svg_data, &opt, &fontdb).ok()?;

        // Create pixmap
        let pixmap_size = tiny_skia::IntSize::from_wh(size, size)?;
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

        // Calculate scale to fit
        let svg_size = tree.size();
        let scale_x = size as f32 / svg_size.width();
        let scale_y = size as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // Center the icon
        let x_offset = (size as f32 - svg_size.width() * scale) / 2.0;
        let y_offset = (size as f32 - svg_size.height() * scale) / 2.0;

        let transform =
            tiny_skia::Transform::from_scale(scale, scale).post_translate(x_offset, y_offset);

        // Render SVG
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Convert to ColorImage
        let pixels: Vec<egui::Color32> = pixmap
            .pixels()
            .iter()
            .map(|p| {
                egui::Color32::from_rgba_premultiplied(p.red(), p.green(), p.blue(), p.alpha())
            })
            .collect();

        let image = ColorImage {
            size: [size as usize, size as usize],
            pixels,
        };

        // Create texture
        let texture = ctx.load_texture(
            format!("icon_{}", path.file_stem()?.to_string_lossy()),
            image,
            egui::TextureOptions::LINEAR,
        );

        Some(texture)
    }

    /// Get an icon texture
    pub fn get(&self, icon: AppIcon) -> Option<&TextureHandle> {
        self.icons.get(&icon)
    }

    /// Get the icon size
    pub fn icon_size(&self) -> u32 {
        self.icon_size
    }

    /// Render an icon as an egui Image
    pub fn image(&self, icon: AppIcon, size: f32) -> Option<egui::Image<'_>> {
        self.get(icon)
            .map(|texture| egui::Image::new(texture).fit_to_exact_size(egui::vec2(size, size)))
    }

    /// Show an icon in the UI
    pub fn show(&self, ui: &mut egui::Ui, icon: AppIcon, size: f32) -> Option<egui::Response> {
        self.image(icon, size).map(|img| ui.add(img))
    }
}
