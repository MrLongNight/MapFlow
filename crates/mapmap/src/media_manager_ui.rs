use egui::{Color32, Id, Sense, Ui, Vec2};
use mapmap_core::media_library::{MediaItem, MediaLibrary, MediaType};

pub struct MediaManagerUI {
    pub visible: bool, // Toggle visibility
    search_query: String,
    view_mode: ViewMode,
    selected_playlist: Option<String>,
    new_playlist_name: String,
    thumbnail_size: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    Grid,
    List,
}

impl Default for MediaManagerUI {
    fn default() -> Self {
        Self {
            visible: false,
            search_query: String::new(),
            view_mode: ViewMode::Grid,
            selected_playlist: None,
            new_playlist_name: String::new(),
            thumbnail_size: 100.0,
        }
    }
}

impl MediaManagerUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ctx: &egui::Context, library: &mut MediaLibrary) {
        if !self.visible {
            return;
        }

        let mut visible = self.visible;
        egui::Window::new("Media Manager")
            .open(&mut visible)
            .resizable(true)
            .default_size([800.0, 600.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.render_sidebar(ui, library);
                    ui.separator();
                    self.render_main_content(ui, library);
                });
            });
        self.visible = visible;
    }

    fn render_sidebar(&mut self, ui: &mut Ui, library: &mut MediaLibrary) {
        ui.vertical(|ui| {
            ui.set_width(200.0);
            ui.heading("Playlists");

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_playlist_name);
                if ui.button("+").clicked() && !self.new_playlist_name.is_empty() {
                    library.create_playlist(self.new_playlist_name.clone());
                    self.new_playlist_name.clear();
                }
            });

            ui.separator();

            if ui
                .selectable_label(self.selected_playlist.is_none(), "All Media")
                .clicked()
            {
                self.selected_playlist = None;
            }

            let mut to_remove = None;
            for playlist in &library.playlists {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            self.selected_playlist.as_ref() == Some(&playlist.name),
                            &playlist.name,
                        )
                        .clicked()
                    {
                        self.selected_playlist = Some(playlist.name.clone());
                    }
                    if ui.small_button("x").clicked() {
                        to_remove = Some(playlist.name.clone());
                    }
                });
            }

            if let Some(name) = to_remove {
                library.remove_playlist(&name);
                if self.selected_playlist.as_ref() == Some(&name) {
                    self.selected_playlist = None;
                }
            }
        });
    }

    fn render_main_content(&mut self, ui: &mut Ui, library: &mut MediaLibrary) {
        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(self.view_mode == ViewMode::List, "List")
                        .clicked()
                    {
                        self.view_mode = ViewMode::List;
                    }
                    if ui
                        .selectable_label(self.view_mode == ViewMode::Grid, "Grid")
                        .clicked()
                    {
                        self.view_mode = ViewMode::Grid;
                    }
                    ui.add(egui::Slider::new(&mut self.thumbnail_size, 50.0..=200.0).text("Size"));
                    if ui.button("Refresh").clicked() {
                        library.refresh();
                    }
                    if ui.button("Add Folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            library.add_scan_path(path);
                            library.refresh();
                        }
                    }
                });
            });

            ui.separator();

            // Content Area
            egui::ScrollArea::vertical().show(ui, |ui| {
                let items = if let Some(playlist_name) = &self.selected_playlist {
                    if let Some(playlist) =
                        library.playlists.iter().find(|p| &p.name == playlist_name)
                    {
                        playlist
                            .items
                            .iter()
                            .filter_map(|path| library.items.get(path))
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                } else {
                    library.get_items()
                };

                let filtered_items: Vec<&MediaItem> = items
                    .into_iter()
                    .filter(|item| {
                        self.search_query.is_empty()
                            || item
                                .name
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                    })
                    .collect();

                match self.view_mode {
                    ViewMode::Grid => self.render_grid(ui, &filtered_items),
                    ViewMode::List => self.render_list(ui, &filtered_items),
                }
            });
        });
    }

    fn render_grid(&mut self, ui: &mut Ui, items: &[&MediaItem]) {
        let available_width = ui.available_width();
        let columns = (available_width / (self.thumbnail_size + 10.0)).floor() as usize;
        let columns = columns.max(1);

        egui::Grid::new("media_grid").striped(true).show(ui, |ui| {
            for (i, item) in items.iter().enumerate() {
                if i > 0 && i % columns == 0 {
                    ui.end_row();
                }

                let size = Vec2::splat(self.thumbnail_size);
                let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

                // Draw thumbnail placeholder
                if ui.is_rect_visible(rect) {
                    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(50));

                    // Icon based on type
                    let icon = match item.media_type {
                        MediaType::Video => "🎬",
                        MediaType::Image => "🖼",
                        MediaType::Audio => "🔊",
                        MediaType::Unknown => "?",
                    };
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        icon,
                        egui::FontId::proportional(size.y * 0.5),
                        Color32::WHITE,
                    );

                    // Name
                    ui.painter().text(
                        rect.center() + Vec2::new(0.0, size.y * 0.4),
                        egui::Align2::CENTER_BOTTOM,
                        &item.name,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );

                    // Hover effect
                    if response.hovered() {
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(2.0, Color32::LIGHT_BLUE),
                            egui::StrokeKind::Outside,
                        );
                    }
                }

                // Drag Source
                if response.drag_started() {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(Id::new("media_path"), item.path.clone()));
                }
            }
        });
    }

    fn render_list(&mut self, ui: &mut Ui, items: &[&MediaItem]) {
        for item in items {
            ui.horizontal(|ui| {
                let icon = match item.media_type {
                    MediaType::Video => "🎬",
                    MediaType::Image => "🖼",
                    MediaType::Audio => "🔊",
                    MediaType::Unknown => "?",
                };
                ui.label(icon);

                let response = ui.label(&item.name).interact(Sense::click_and_drag());
                ui.label(format_size(
                    item.metadata.as_ref().map(|m| m.file_size).unwrap_or(0),
                ));

                // Drag Source
                if response.drag_started() {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(Id::new("media_path"), item.path.clone()));
                }
            });
        }
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}
