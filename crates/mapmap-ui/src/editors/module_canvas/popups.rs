use super::state::ModuleCanvas;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use mapmap_core::module::{
    BlendModeType, HueNodeType, LayerType, MapFlowModule, MaskType, ModulizerType, ModuleManager,
    ModulePartType, OutputType, SourceType, TriggerType, AudioBand, AudioTriggerOutputConfig,
    MaskShape, MeshType, NodeLinkData,
};
use mapmap_core::diagnostics::IssueSeverity;

impl ModuleCanvas {
    pub fn draw_search_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Search popup in top-center
        let popup_width = 300.0;
        let popup_height = 200.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 30, 40, 240),
        );
        painter.rect_stroke(
            popup_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(80, 120, 200)),
            egui::StrokeKind::Middle,
        );

        // Popup content
        let inner_rect = popup_rect.shrink(10.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.search_filter);
                });
                ui.add_space(8.0);

                // Filter and show matching nodes
                let filter_lower = self.search_filter.to_lowercase();
                let matching_parts: Vec<_> = module
                    .parts
                    .iter()
                    .filter(|p| {
                        if filter_lower.is_empty() {
                            return true;
                        }
                        let name = Self::get_part_property_text(&p.part_type).to_lowercase();
                        let (_, _, _, type_name) = Self::get_part_style(&p.part_type);
                        name.contains(&filter_lower)
                            || type_name.to_lowercase().contains(&filter_lower)
                    })
                    .take(6)
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for part in matching_parts {
                            let (_, _, icon, type_name) = Self::get_part_style(&part.part_type);
                            let label = format!(
                                "{} {} - {}",
                                icon,
                                type_name,
                                Self::get_part_property_text(&part.part_type)
                            );
                            if ui
                                .selectable_label(self.selected_parts.contains(&part.id), &label)
                                .clicked()
                            {
                                self.selected_parts.clear();
                                self.selected_parts.push(part.id);
                                // Center view on selected node
                                self.pan_offset =
                                    Vec2::new(-part.position.0 + 200.0, -part.position.1 + 150.0);
                                self.show_search = false;
                            }
                        }
                    });
            });
        });
    }

    pub fn draw_presets_popup(&mut self, ui: &mut Ui, canvas_rect: Rect, module: &mut MapFlowModule) {
        // Presets popup in top-center
        let popup_width = 280.0;
        let popup_height = 220.0;
        let popup_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.center().x - popup_width / 2.0,
                canvas_rect.min.y + 50.0,
            ),
            Vec2::new(popup_width, popup_height),
        );

        // Draw popup background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 35, 45, 245),
        );
        painter.rect_stroke(
            popup_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(100, 180, 80)),
            egui::StrokeKind::Middle,
        );

        // Popup content
        let inner_rect = popup_rect.shrink(12.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.heading("📋 Presets / Templates");
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        let presets = self.presets.clone();
                        for preset in &presets {
                            ui.horizontal(|ui| {
                                if ui.button(&preset.name).clicked() {
                                    // Clear current and load preset
                                    module.parts.clear();
                                    module.connections.clear();

                                    // Add parts from preset
                                    let mut part_ids = Vec::new();
                                    let mut next_id =
                                        module.parts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                                    for (part_type, position, size) in &preset.parts {
                                        let id = next_id;
                                        next_id += 1;

                                        let (inputs, outputs) =
                                            Self::get_sockets_for_part_type(part_type);

                                        module.parts.push(mapmap_core::module::ModulePart {
                                            id,
                                            part_type: part_type.clone(),
                                            position: *position,
                                            size: *size,
                                            inputs,
                                            outputs,
                                            link_data: NodeLinkData::default(),
                                            trigger_targets: std::collections::HashMap::new(),
                                        });
                                        part_ids.push(id);
                                    }

                                    // Add connections
                                    for (from_idx, from_socket, to_idx, to_socket) in
                                        &preset.connections
                                    {
                                        if *from_idx < part_ids.len() && *to_idx < part_ids.len() {
                                            module.connections.push(
                                                mapmap_core::module::ModuleConnection {
                                                    from_part: part_ids[*from_idx],
                                                    from_socket: *from_socket,
                                                    to_part: part_ids[*to_idx],
                                                    to_socket: *to_socket,
                                                },
                                            );
                                        }
                                    }

                                    self.show_presets = false;
                                }
                                ui.label(format!("({} nodes)", preset.parts.len()));
                            });
                        }
                    });

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_presets = false;
                }
            });
        });
    }

    pub fn render_diagnostics_popup(&mut self, ui: &mut Ui) {
        if !self.show_diagnostics {
            return;
        }

        let popup_size = Vec2::new(350.0, 250.0);
        let available = ui.available_rect_before_wrap();
        let popup_pos = Pos2::new(
            (available.min.x + available.max.x - popup_size.x) / 2.0,
            (available.min.y + available.max.y - popup_size.y) / 2.0,
        );
        let popup_rect = egui::Rect::from_min_size(popup_pos, popup_size);

        // Background
        let painter = ui.painter();
        painter.rect_filled(
            popup_rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 35, 45, 245),
        );
        painter.rect_stroke(
            popup_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(180, 100, 80)),
            egui::StrokeKind::Middle,
        );

        let inner_rect = popup_rect.shrink(12.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
            ui.vertical(|ui| {
                ui.heading(if self.diagnostic_issues.is_empty() {
                    "âœ“ Module Check: OK"
                } else {
                    "\u{26A0} Module Check: Issues Found"
                });
                ui.add_space(8.0);

                if self.diagnostic_issues.is_empty() {
                    ui.label("No issues found. Your module looks good!");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for issue in &self.diagnostic_issues {
                                let (icon, color) = match issue.severity {
                                    IssueSeverity::Error => {
                                        ("â Œ", Color32::RED)
                                    }
                                    IssueSeverity::Warning => {
                                        ("\u{26A0}", Color32::YELLOW)
                                    }
                                    IssueSeverity::Info => {
                                        ("\u{2139}", Color32::LIGHT_BLUE)
                                    }
                                };
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, icon);
                                    ui.label(&issue.message);
                                });
                            }
                        });
                }

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_diagnostics = false;
                }
            });
        });
    }

    /// Add a Trigger node with specified type
    pub fn add_trigger_node(
        &mut self,
        manager: &mut ModuleManager,
        trigger_type: TriggerType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((100.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    ModulePartType::Trigger(trigger_type),
                    pos,
                );
            }
        }
    }

    /// Add a Source node with specified type
    pub fn add_source_node(
        &mut self,
        manager: &mut ModuleManager,
        source_type: SourceType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((200.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    ModulePartType::Source(source_type),
                    pos,
                );
            }
        }
    }

    /// Add a Mask node with specified type
    pub fn add_mask_node(
        &mut self,
        manager: &mut ModuleManager,
        mask_type: MaskType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((300.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module
                    .add_part_with_type(ModulePartType::Mask(mask_type), pos);
            }
        }
    }

    /// Add a Modulator node with specified type
    pub fn add_modulator_node(
        &mut self,
        manager: &mut ModuleManager,
        mod_type: ModulizerType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((400.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    ModulePartType::Modulizer(mod_type),
                    pos,
                );
            }
        }
    }

    /// Add a Hue node with specified type
    pub fn add_hue_node(
        &mut self,
        manager: &mut ModuleManager,
        hue_type: HueNodeType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((500.0, 100.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(ModulePartType::Hue(hue_type), pos);
            }
        }
    }

    pub fn add_layer_node(
        &mut self,
        manager: &mut ModuleManager,
        layer_type: LayerType,
        pos_override: Option<(f32, f32)>,
    ) {
        if let Some(id) = self.active_module_id {
            if let Some(module) = manager.get_module_mut(id) {
                let preferred_pos = pos_override.unwrap_or((400.0, 200.0));
                let pos = Self::find_free_position(&module.parts, preferred_pos);
                module.add_part_with_type(
                    ModulePartType::Layer(layer_type),
                    pos,
                );
            }
        }
    }

    /// Content for the Sources menu
    fn render_sources_menu_content(
        &mut self,
        ui: &mut egui::Ui,
        manager: &mut ModuleManager,
        pos_override: Option<(f32, f32)>,
    ) {
        ui.label("--- 📁 File Based ---");
        if ui.button("\u{1F4F9} Media File").clicked() {
            self.add_source_node(
                manager,
                SourceType::new_media_file(String::new()),
                pos_override,
            );
            ui.close();
        }
        if ui.button("\u{1F4F9} Video (Uni)").clicked() {
            self.add_source_node(
                manager,
                SourceType::VideoUni {
                    path: String::new(),
                    speed: 1.0,
                    loop_enabled: true,
                    start_time: 0.0,
                    end_time: 0.0,
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    target_width: None,
                    target_height: None,
                    target_fps: None,
                    flip_horizontal: false,
                    flip_vertical: false,
                    reverse_playback: false,
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("\u{1F5BC} Image (Uni)").clicked() {
            self.add_source_node(
                manager,
                SourceType::ImageUni {
                    path: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    target_width: None,
                    target_height: None,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- \u{1F517} Shared (Multi) ---");
        if ui.button("\u{1F4F9} Video (Multi)").clicked() {
            self.add_source_node(
                manager,
                SourceType::VideoMulti {
                    shared_id: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("\u{1F5BC} Image (Multi)").clicked() {
            self.add_source_node(
                manager,
                SourceType::ImageMulti {
                    shared_id: String::new(),
                    opacity: 1.0,
                    blend_mode: None,
                    brightness: 0.0,
                    contrast: 1.0,
                    saturation: 1.0,
                    hue_shift: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- \u{1F4E1} Hardware & Network ---");
        if ui.button("\u{1F4F9} Live Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::LiveInput { device_id: 0 },
                pos_override,
            );
            ui.close();
        }
        if ui.button("\u{1F4E1} NDI Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::NdiInput { source_name: None },
                pos_override,
            );
            ui.close();
        }
        #[cfg(target_os = "windows")]
        if ui.button("\u{1F6B0} Spout Input").clicked() {
            self.add_source_node(
                manager,
                SourceType::SpoutInput {
                    sender_name: String::new(),
                },
                pos_override,
            );
            ui.close();
        }

        ui.add_space(4.0);
        ui.label("--- \u{1F3A8} Procedural & Misc ---");
        if ui.button("\u{1F3A8} Shader").clicked() {
            self.add_source_node(
                manager,
                SourceType::Shader {
                    name: "New Shader".to_string(),
                    params: vec![],
                },
                pos_override,
            );
            ui.close();
        }
        if ui.button("\u{1F3AE} Bevy Scene").clicked() {
            self.add_source_node(manager, SourceType::Bevy, pos_override);
            ui.close();
        }
    }

    /// Content for the Add Node menu (used by both toolbar and context menu)
    pub fn render_add_node_menu_content(
        &mut self,
        ui: &mut egui::Ui,
        manager: &mut ModuleManager,
        pos_override: Option<(f32, f32)>,
    ) {
        ui.set_min_width(150.0);

        ui.menu_button("📽️ Sources", |ui| {
            self.render_sources_menu_content(ui, manager, pos_override);
        });

        ui.menu_button("\u{26A1} Triggers", |ui| {
            if ui.button("\u{1F3B5} Audio FFT").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::AudioFFT {
                        band: AudioBand::Bass,
                        threshold: 0.5,
                        output_config: AudioTriggerOutputConfig::default(),
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("\u{1F3B2} Random").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Random {
                        min_interval_ms: 500,
                        max_interval_ms: 2000,
                        probability: 0.5,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("⏱️ Fixed").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Fixed {
                        interval_ms: 1000,
                        offset_ms: 0,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("\u{1F3B9} MIDI").clicked() {
                self.add_trigger_node(
                    manager,
                    TriggerType::Midi {
                        device: "Default".to_string(),
                        channel: 1,
                        note: 60,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("\u{1F3AD} Masks", |ui| {
            if ui.button("\u{2B55} Shape").clicked() {
                self.add_mask_node(
                    manager,
                    MaskType::Shape(MaskShape::Circle),
                    pos_override,
                );
                ui.close();
            }
            if ui.button("\u{1F308} Gradient").clicked() {
                self.add_mask_node(
                    manager,
                    MaskType::Gradient {
                        angle: 0.0,
                        softness: 0.5,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("🎛️ Modulators", |ui| {
            if ui.button("🎚️ Blend Mode").clicked() {
                self.add_modulator_node(
                    manager,
                    ModulizerType::BlendMode(BlendModeType::Normal),
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("\u{1F4D1} Layers", |ui| {
            if ui.button("\u{1F4D1} Single Layer").clicked() {
                self.add_layer_node(
                    manager,
                    LayerType::Single {
                        id: 0,
                        name: "New Layer".to_string(),
                        opacity: 1.0,
                        blend_mode: None,
                        mesh: MeshType::default(),
                        mapping_mode: false,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("📁 Layer Group").clicked() {
                self.add_layer_node(
                    manager,
                    LayerType::Group {
                        name: "New Group".to_string(),
                        opacity: 1.0,
                        blend_mode: None,
                        mesh: MeshType::default(),
                        mapping_mode: false,
                    },
                    pos_override,
                );
                ui.close();
            }
            if ui.button("\u{1F4D1} All Layers").clicked() {
                self.add_layer_node(
                    manager,
                    LayerType::All {
                        opacity: 1.0,
                        blend_mode: None,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.menu_button("\u{1F4A1} Philips Hue", |ui| {
            if ui.button("\u{1F4A1} Single Lamp").clicked() {
                self.add_hue_node(
                    manager,
                    HueNodeType::SingleLamp {
                        id: String::new(),
                        name: "New Lamp".to_string(),
                        brightness: 1.0,
                        color: [1.0, 1.0, 1.0],
                        effect: None,
                        effect_active: false,
                    },
                    pos_override,
                );
                ui.close();
            }
        });

        ui.separator();

        if ui.button("\u{1F5BC} Output").clicked() {
            if let Some(id) = self.active_module_id {
                if let Some(module) = manager.get_module_mut(id) {
                    let preferred_pos = pos_override.unwrap_or((600.0, 100.0));
                    let pos = Self::find_free_position(&module.parts, preferred_pos);
                    module.add_part_with_type(
                        ModulePartType::Output(
                            OutputType::Projector {
                                id: 1,
                                name: "Projector 1".to_string(),
                                hide_cursor: false,
                                target_screen: 0,
                                show_in_preview_panel: true,
                                extra_preview_window: false,
                                output_width: 0,
                                output_height: 0,
                                output_fps: 60.0,
                                ndi_enabled: false,
                                ndi_stream_name: String::new(),
                            },
                        ),
                        pos,
                    );
                }
            }
            ui.close();
        }
    }
}
