//! Phase 6: Media Browser UI
//!
//! File browsing interface with thumbnails, search/filter, drag-and-drop support,
//! color coding, and hover preview playback.

use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};
use egui::{Color32, Response, Sense, Ui, Vec2};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Media file entry in the browser
#[derive(Debug, Clone)]
pub struct MediaEntry {
    /// File system path to the asset or resource.
    pub path: PathBuf,
    /// Human-readable display name.
    pub name: String,
    /// Lowercased name for fast searching without allocation
    pub name_lower: String,
    pub file_type: MediaType,
    pub size_bytes: u64,
    pub duration_secs: Option<f32>,
    pub thumbnail: Option<ThumbnailHandle>,
    pub color_tag: Option<Color32>,
    pub tags: Vec<String>,
    /// Lowercased tags for fast searching without allocation
    pub tags_lower: Vec<String>,
}

/// Media type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Image,
    ImageSequence,
    Audio,
    /// HAP video (GPU-accelerated codec)
    Hap,
    Unknown,
}

impl MediaType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp4" | "avi" | "mpeg" | "mpg" | "mkv" | "webm" => Self::Video,
            // MOV can be HAP or regular video - we'll detect at open time
            "mov" => Self::Video, // Could be HAP, will be detected when opened
            "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" | "dds" => Self::Image,
            "gif" => Self::ImageSequence,
            "wav" | "mp3" | "aac" | "flac" | "ogg" => Self::Audio,
            _ => Self::Unknown,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Video => "🎬",
            Self::Image => "🖼",
            Self::ImageSequence => "🎞",
            Self::Audio => "🎵",
            Self::Hap => "⚡", // Lightning for GPU-accelerated
            Self::Unknown => "📄",
        }
    }

    pub fn app_icon(&self) -> Option<AppIcon> {
        match self {
            Self::Video => Some(AppIcon::VideoFile),
            Self::Image => Some(AppIcon::ImageFile),
            Self::ImageSequence => Some(AppIcon::VideoFile),
            Self::Audio => Some(AppIcon::AudioFile),
            Self::Hap => Some(AppIcon::VideoPlayer), // Use VideoPlayer for HAP
            Self::Unknown => None,
        }
    }
}

/// Thumbnail handle (reference to generated thumbnail)
#[derive(Clone)]
pub struct ThumbnailHandle {
    pub texture_handle: egui::TextureHandle,
    pub size: (u32, u32),
}

impl std::fmt::Debug for ThumbnailHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThumbnailHandle")
            .field("size", &self.size)
            .finish()
    }
}

/// Media browser state
pub struct MediaBrowser {
    /// Current directory
    current_dir: PathBuf,
    /// Path input for editing
    path_input: String,
    /// Media entries in current directory
    entries: Vec<MediaEntry>,
    /// Search query
    search_query: String,
    /// Filter by type
    filter_type: Option<MediaType>,
    /// View mode
    view_mode: ViewMode,
    /// Thumbnail size in pixels
    thumbnail_size: f32,
    /// Selected entry index
    selected: Option<usize>,
    /// Hovered entry (for preview)
    hovered: Option<usize>,
    /// Hover start time (for delayed preview)
    hover_start: Option<Instant>,
    /// Preview delay in seconds
    preview_delay: f32,
    /// Thumbnail cache
    thumbnail_cache: Arc<RwLock<HashMap<PathBuf, ThumbnailHandle>>>,
    /// Receiver for generated thumbnails
    thumbnail_rx: std::sync::mpsc::Receiver<(PathBuf, Result<egui::ColorImage, String>)>,
    /// Sender for generated thumbnails
    thumbnail_tx: std::sync::mpsc::Sender<(PathBuf, Result<egui::ColorImage, String>)>,
    /// Currently generating thumbnails
    generating_thumbnails: Arc<RwLock<HashSet<PathBuf>>>,
    /// Show hidden files
    show_hidden: bool,
    /// Sort mode
    sort_mode: SortMode,
    /// Directory history (for back/forward navigation)
    history: Vec<PathBuf>,
    history_index: usize,
    /// Media folders per type
    pub media_folders: MediaFolders,
    /// Show folder settings
    show_folder_settings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Type,
    Size,
    DateModified,
}

/// Media folder configuration per type
#[derive(Debug, Clone)]
pub struct MediaFolders {
    pub video_folder: PathBuf,
    pub image_folder: PathBuf,
    pub audio_folder: PathBuf,
    pub default_folder: PathBuf,
}

impl Default for MediaFolders {
    fn default() -> Self {
        let default = std::env::current_dir().unwrap_or_default();
        Self {
            video_folder: dirs::video_dir().unwrap_or(default.clone()),
            image_folder: dirs::picture_dir().unwrap_or(default.clone()),
            audio_folder: dirs::audio_dir().unwrap_or(default.clone()),
            default_folder: default,
        }
    }
}

impl MediaBrowser {
    pub fn new(initial_dir: PathBuf) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let path_str = initial_dir.display().to_string();
        let mut browser = Self {
            current_dir: initial_dir.clone(),
            path_input: path_str,
            entries: Vec::new(),
            search_query: String::new(),
            filter_type: None,
            view_mode: ViewMode::Grid,
            thumbnail_size: 80.0, // Reduced from 120 for compact view
            selected: None,
            hovered: None,
            hover_start: None,
            preview_delay: 0.5,
            thumbnail_cache: Arc::new(RwLock::new(HashMap::new())),
            thumbnail_rx: rx,
            thumbnail_tx: tx,
            generating_thumbnails: Arc::new(RwLock::new(HashSet::new())),
            show_hidden: false,
            sort_mode: SortMode::Name,
            history: vec![initial_dir.clone()],
            history_index: 0,
            media_folders: MediaFolders {
                video_folder: initial_dir.clone(),
                image_folder: initial_dir.clone(),
                audio_folder: initial_dir.clone(),
                default_folder: initial_dir,
            },
            show_folder_settings: false,
        };
        browser.refresh();
        browser
    }

    /// Refresh the file list
    pub fn refresh(&mut self) {
        self.entries.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();

                        // Skip hidden files if not showing them
                        if !self.show_hidden && name.starts_with('.') {
                            continue;
                        }

                        let file_type = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(MediaType::from_extension)
                            .unwrap_or(MediaType::Unknown);

                        // Only include media files
                        if matches!(
                            file_type,
                            MediaType::Video
                                | MediaType::Image
                                | MediaType::ImageSequence
                                | MediaType::Audio
                                | MediaType::Hap
                        ) {
                            let thumbnail = self.get_or_generate_thumbnail(&path);

                            self.entries.push(MediaEntry {
                                path,
                                name: name.clone(),
                                name_lower: name.to_lowercase(),
                                file_type,
                                size_bytes: metadata.len(),
                                duration_secs: None, // TODO: Extract from media file
                                thumbnail,
                                color_tag: None,
                                tags: Vec::new(),
                                tags_lower: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        self.sort_entries();
    }

    /// Sort entries based on sort mode
    fn sort_entries(&mut self) {
        match self.sort_mode {
            SortMode::Name => self.entries.sort_by(|a, b| a.name.cmp(&b.name)),
            SortMode::Type => self.entries.sort_by_key(|e| e.file_type as u8),
            SortMode::Size => self.entries.sort_by_key(|e| e.size_bytes),
            SortMode::DateModified => {
                // Would need to store modification time
            }
        }
    }

    /// Get or generate thumbnail for a file
    fn get_or_generate_thumbnail(&self, path: &Path) -> Option<ThumbnailHandle> {
        // Check cache first
        if let Some(thumb) = self.thumbnail_cache.read().get(path) {
            return Some(thumb.clone());
        }

        // Check if already generating
        let mut generating = self.generating_thumbnails.write();
        if generating.contains(path) {
            return None;
        }

        let file_type = path
            .extension()
            .and_then(|e| e.to_str())
            .map(MediaType::from_extension)
            .unwrap_or(MediaType::Unknown);

        // Generate thumbnail in background for supported media types
        if matches!(file_type, MediaType::Image) {
            generating.insert(path.to_path_buf());
            let tx = self.thumbnail_tx.clone();
            let path_clone = path.to_path_buf();

            rayon::spawn(move || {
                let result = match image::open(&path_clone) {
                    Ok(img) => {
                        let thumbnail = img.thumbnail(128, 128); // Standard thumbnail size
                        let size = [thumbnail.width() as _, thumbnail.height() as _];
                        let rgba = thumbnail.to_rgba8();
                        Ok(egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            rgba.as_flat_samples().as_slice(),
                        ))
                    }
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send((path_clone, result));
            });
        }

        None
    }

    /// Navigate to a directory
    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_dir = path.clone();
            self.path_input = path.display().to_string();
            self.refresh();

            // Update history
            self.history.truncate(self.history_index + 1);
            self.history.push(path);
            self.history_index = self.history.len() - 1;
        }
    }

    /// Navigate back in history
    pub fn navigate_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_dir = self.history[self.history_index].clone();
            self.path_input = self.current_dir.display().to_string();
            self.refresh();
        }
    }

    /// Navigate forward in history
    pub fn navigate_forward(&mut self) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_dir = self.history[self.history_index].clone();
            self.path_input = self.current_dir.display().to_string();
            self.refresh();
        }
    }

    /// Navigate to parent directory
    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.navigate_to(parent.to_path_buf());
        }
    }

    /// Get filtered and searched entries
    fn filtered_entries(&self) -> Vec<(usize, &MediaEntry)> {
        let query = if !self.search_query.is_empty() {
            Some(self.search_query.to_lowercase())
        } else {
            None
        };

        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                // Filter by type
                if let Some(filter) = self.filter_type {
                    if entry.file_type != filter {
                        return false;
                    }
                }

                // Filter by search query
                if let Some(q) = &query {
                    let name_matches = entry.name_lower.contains(q);
                    let tag_matches = entry.tags_lower.iter().any(|t| t.contains(q));
                    if !name_matches && !tag_matches {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Render the media browser UI
    /// Process completed thumbnails and clear flags
    pub fn process_thumbnails(&mut self, ctx: &egui::Context) {
        while let Ok((path, result)) = self.thumbnail_rx.try_recv() {
            if let Ok(color_image) = result {
                let size = (color_image.size[0] as u32, color_image.size[1] as u32);
                let texture = ctx.load_texture(
                    format!("thumb_{}", path.display()),
                    color_image,
                    egui::TextureOptions {
                        magnification: egui::TextureFilter::Linear,
                        minification: egui::TextureFilter::Linear,
                        wrap_mode: egui::TextureWrapMode::ClampToEdge,
                        mipmap_mode: None,
                    },
                );

                let handle = ThumbnailHandle {
                    texture_handle: texture,
                    size,
                };

                self.thumbnail_cache
                    .write()
                    .insert(path.clone(), handle.clone());

                // Update the corresponding entry in the list
                for entry in &mut self.entries {
                    if entry.path == path {
                        entry.thumbnail = Some(handle.clone());
                        break;
                    }
                }

                // Trigger a UI update to reflect the new thumbnail
                ctx.request_repaint();
            }

            // Remove from generating set
            self.generating_thumbnails.write().remove(&path);
        }
    }

    /// Render the media browser UI
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icons: Option<&IconManager>,
    ) -> Option<MediaBrowserAction> {
        self.process_thumbnails(ui.ctx());

        let mut action = None;

        // Compact toolbar with navigation
        ui.horizontal(|ui| {
            // Navigation buttons (compact, icons only)
            ui.add_enabled_ui(self.history_index > 0, |ui| {
                if ui
                    .button("◀")
                    .clone()
                    .on_hover_text(locale.t("media-browser-back"))
                    .clicked()
                {
                    self.navigate_back();
                }
            });

            ui.add_enabled_ui(self.history_index < self.history.len() - 1, |ui| {
                if ui
                    .button("▶")
                    .clone()
                    .on_hover_text(locale.t("media-browser-forward"))
                    .clicked()
                {
                    self.navigate_forward();
                }
            });

            if ui
                .button("⬆")
                .clone()
                .on_hover_text(locale.t("media-browser-up"))
                .clicked()
            {
                self.navigate_up();
            }

            if ui
                .button("🔄")
                .clone()
                .on_hover_text(locale.t("media-browser-refresh"))
                .clicked()
            {
                self.refresh();
            }

            if ui
                .button("⚙")
                .clone()
                .on_hover_text("Folder Settings")
                .clicked()
            {
                self.show_folder_settings = !self.show_folder_settings;
            }

            ui.separator();

            // Editable path input
            let path_response = ui.add(
                egui::TextEdit::singleline(&mut self.path_input)
                    .desired_width(ui.available_width() - 30.0)
                    .hint_text("Enter path..."),
            );

            if path_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let new_path = PathBuf::from(&self.path_input);
                if new_path.is_dir() {
                    self.navigate_to(new_path);
                }
            }
        });

        // Folder Settings Panel (collapsible)
        if self.show_folder_settings {
            ui.group(|ui| {
                ui.label("📁 Media Folder Settings");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("🎬 Video:");
                    let mut video_path = self.media_folders.video_folder.display().to_string();
                    if ui.text_edit_singleline(&mut video_path).changed() {
                        self.media_folders.video_folder = PathBuf::from(video_path);
                    }
                    if ui.button("📂").clone().on_hover_text("Browse").clicked() {
                        // Would trigger folder dialog
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("🖼 Image:");
                    let mut image_path = self.media_folders.image_folder.display().to_string();
                    if ui.text_edit_singleline(&mut image_path).changed() {
                        self.media_folders.image_folder = PathBuf::from(image_path);
                    }
                    if ui.button("📂").clone().on_hover_text("Browse").clicked() {
                        // Would trigger folder dialog
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("🎵 Audio:");
                    let mut audio_path = self.media_folders.audio_folder.display().to_string();
                    if ui.text_edit_singleline(&mut audio_path).changed() {
                        self.media_folders.audio_folder = PathBuf::from(audio_path);
                    }
                    if ui.button("📂").clone().on_hover_text("Browse").clicked() {
                        // Would trigger folder dialog
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Apply Video Folder").clicked() {
                        let path = self.media_folders.video_folder.clone();
                        self.navigate_to(path);
                    }
                    if ui.button("Apply Image Folder").clicked() {
                        let path = self.media_folders.image_folder.clone();
                        self.navigate_to(path);
                    }
                    if ui.button("Apply Audio Folder").clicked() {
                        let path = self.media_folders.audio_folder.clone();
                        self.navigate_to(path);
                    }
                });
            });
        }

        ui.separator();

        // Search and filter bar - wrapped in horizontal scroll to prevent forcing sidebar width
        egui::ScrollArea::horizontal()
            .id_salt("media_filter_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let search_response = ui.text_edit_singleline(&mut self.search_query);
                    if search_response.changed() {
                        // Search query changed
                    }

                    ui.separator();

                    ui.label(locale.t("media-browser-filter"));
                    ui.selectable_value(&mut self.filter_type, None, locale.t("media-browser-all"));
                    ui.selectable_value(
                        &mut self.filter_type,
                        Some(MediaType::Video),
                        locale.t("media-browser-video"),
                    );
                    ui.selectable_value(
                        &mut self.filter_type,
                        Some(MediaType::Image),
                        locale.t("media-browser-image"),
                    );
                    ui.selectable_value(
                        &mut self.filter_type,
                        Some(MediaType::Audio),
                        locale.t("media-browser-audio"),
                    );

                    ui.separator();

                    // View mode
                    ui.selectable_value(
                        &mut self.view_mode,
                        ViewMode::Grid,
                        locale.t("media-browser-view-grid"),
                    );
                    ui.selectable_value(
                        &mut self.view_mode,
                        ViewMode::List,
                        locale.t("media-browser-view-list"),
                    );

                    ui.separator();

                    // Sort mode
                    egui::ComboBox::from_label(locale.t("media-browser-sort"))
                        .selected_text(format!("{:?}", self.sort_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.sort_mode,
                                SortMode::Name,
                                locale.t("media-browser-sort-name"),
                            );
                            ui.selectable_value(
                                &mut self.sort_mode,
                                SortMode::Type,
                                locale.t("media-browser-sort-type"),
                            );
                            ui.selectable_value(
                                &mut self.sort_mode,
                                SortMode::Size,
                                locale.t("media-browser-sort-size"),
                            );
                        });
                });
            });

        ui.separator();

        // Content area
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Collect indices to avoid borrowing issues
            let entry_indices: Vec<usize> = self
                .filtered_entries()
                .into_iter()
                .map(|(i, _)| i)
                .collect();

            if entry_indices.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    // Differentiate between empty folder and no search results
                    if self.entries.is_empty() {
                        ui.label(locale.t("media-browser-empty-folder"));
                    } else {
                        ui.label(locale.t("media-browser-no-results"));
                    }
                });
            } else {
                match self.view_mode {
                    ViewMode::Grid => {
                        action = self.render_grid_view(ui, &entry_indices, icons);
                    }
                    ViewMode::List => {
                        action = self.render_list_view(ui, &entry_indices, icons);
                    }
                }
            }
        });

        action
    }

    /// Render grid view
    fn render_grid_view(
        &mut self,
        ui: &mut Ui,
        entry_indices: &[usize],
        _icons: Option<&IconManager>,
    ) -> Option<MediaBrowserAction> {
        let mut action = None;
        let item_size = Vec2::new(self.thumbnail_size, self.thumbnail_size + 40.0);
        let available_width = ui.available_width();
        let columns = (available_width / (item_size.x + 8.0)).floor().max(1.0) as usize;

        egui::Grid::new("media_grid")
            .spacing([8.0, 8.0])
            .min_col_width(item_size.x)
            .show(ui, |ui| {
                for (i, &idx) in entry_indices.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    let entry = &self.entries[idx];
                    let response = self.render_thumbnail_item(ui, entry, idx, _icons);

                    if response.clicked() {
                        self.selected = Some(idx);
                        action = Some(MediaBrowserAction::FileSelected(entry.path.clone()));
                    }

                    if response.double_clicked() {
                        action = Some(MediaBrowserAction::FileDoubleClicked(entry.path.clone()));
                    }

                    if response.hovered() && self.hovered != Some(idx) {
                        self.hovered = Some(idx);
                        self.hover_start = Some(Instant::now());
                    }
                }
            });

        // Check for preview trigger
        if let Some(hover_time) = self.hover_start {
            if hover_time.elapsed().as_secs_f32() > self.preview_delay {
                if let Some(hovered_idx) = self.hovered {
                    if hovered_idx < self.entries.len() {
                        let entry = &self.entries[hovered_idx];
                        action = Some(MediaBrowserAction::StartPreview(entry.path.clone()));
                    }
                }
            }
        }

        action
    }

    /// Render list view
    fn render_list_view(
        &mut self,
        ui: &mut Ui,
        entry_indices: &[usize],
        icons: Option<&IconManager>,
    ) -> Option<MediaBrowserAction> {
        let mut action = None;

        for &idx in entry_indices {
            let entry = &self.entries[idx];
            ui.horizontal(|ui| {
                // Icon
                if let Some(mgr) = icons {
                    if let Some(icon) = entry.file_type.app_icon() {
                        if let Some(img) = mgr.image(icon, 16.0) {
                            ui.add(img);
                        } else {
                            ui.label(entry.file_type.icon());
                        }
                    } else {
                        ui.label(entry.file_type.icon());
                    }
                } else {
                    ui.label(entry.file_type.icon());
                }

                // Color tag
                if let Some(color) = entry.color_tag {
                    ui.colored_label(color, "●");
                }

                // Name (clickable)
                let name_label = ui.selectable_label(self.selected == Some(idx), &entry.name);
                if name_label.clicked() {
                    self.selected = Some(idx);
                    action = Some(MediaBrowserAction::FileSelected(entry.path.clone()));
                }
                if name_label.double_clicked() {
                    action = Some(MediaBrowserAction::FileDoubleClicked(entry.path.clone()));
                }

                // Size
                ui.label(format_size(entry.size_bytes));

                // Duration
                if let Some(duration) = entry.duration_secs {
                    ui.label(format_duration(duration));
                }
            });
        }

        action
    }

    /// Render a thumbnail item
    fn render_thumbnail_item(
        &self,
        ui: &mut Ui,
        entry: &MediaEntry,
        idx: usize,
        icons: Option<&IconManager>,
    ) -> Response {
        let size = Vec2::new(self.thumbnail_size, self.thumbnail_size + 40.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Background
            let bg_color = if self.selected == Some(idx) {
                Color32::from_rgb(60, 120, 200)
            } else if response.hovered() {
                Color32::from_rgb(50, 50, 50)
            } else {
                Color32::from_rgb(35, 35, 35)
            };

            ui.painter().rect_filled(rect, 2.0, bg_color);

            // Thumbnail area
            let thumb_rect = egui::Rect::from_min_size(
                rect.min,
                Vec2::new(self.thumbnail_size, self.thumbnail_size),
            );

            if let Some(thumbnail) = &entry.thumbnail {
                // Render thumbnail texture
                ui.painter().image(
                    thumbnail.texture_handle.id(),
                    thumb_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // Placeholder
                ui.painter()
                    .rect_filled(thumb_rect, 2.0, Color32::from_rgb(45, 45, 45));

                // Try to render icon, fallback to emoji
                let mut rendered_icon = false;
                if let Some(mgr) = icons {
                    if let Some(app_icon) = entry.file_type.app_icon() {
                        if let Some(texture) = mgr.get(app_icon) {
                            // Calculate icon size (centered, 64x64 or smaller)
                            let icon_size = Vec2::new(64.0, 64.0).min(thumb_rect.size() * 0.8);
                            let icon_rect =
                                egui::Rect::from_center_size(thumb_rect.center(), icon_size);

                            ui.painter().image(
                                texture.id(),
                                icon_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::from_gray(200), // Tinted slightly
                            );
                            rendered_icon = true;
                        }
                    }
                }

                if !rendered_icon {
                    let icon_pos = thumb_rect.center() - Vec2::new(20.0, 20.0);
                    ui.painter().text(
                        icon_pos,
                        egui::Align2::LEFT_TOP,
                        entry.file_type.icon(),
                        egui::FontId::proportional(40.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
            }

            // Color tag indicator
            if let Some(color) = entry.color_tag {
                let tag_rect = egui::Rect::from_min_size(
                    thumb_rect.min + Vec2::new(4.0, 4.0),
                    Vec2::new(12.0, 12.0),
                );
                ui.painter().circle_filled(tag_rect.center(), 6.0, color);
            }

            // Name label
            let name_rect = egui::Rect::from_min_size(
                rect.min + Vec2::new(0.0, self.thumbnail_size),
                Vec2::new(self.thumbnail_size, 40.0),
            );
            ui.painter().text(
                name_rect.center_top() + Vec2::new(0.0, 4.0),
                egui::Align2::CENTER_TOP,
                &entry.name,
                egui::FontId::proportional(12.0),
                visuals.text_color(),
            );
        }

        response
    }
}

/// Actions that can be triggered by the media browser
#[derive(Debug, Clone)]
pub enum MediaBrowserAction {
    FileSelected(PathBuf),
    FileDoubleClicked(PathBuf),
    StartPreview(PathBuf),
    StopPreview,
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

/// Format duration for display
fn format_duration(seconds: f32) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("{:02}:{:02}", minutes, secs)
}
