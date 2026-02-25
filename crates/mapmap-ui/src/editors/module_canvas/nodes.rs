use super::state::ModuleCanvas;
use super::types::MediaPlaybackCommand;
use crate::theme::colors;
use crate::UIAction;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{
    AudioBand, BlendModeType, EffectType, HueNodeType, LayerType, MaskShape, MaskType,
    ModulizerType, ModuleId, ModulePart, ModulePartType, OutputType, SourceType, TriggerType,
};

impl ModuleCanvas {
    pub fn get_delete_button_rect(&self, part_rect: Rect) -> Rect {
        let title_height = 28.0 * self.zoom;
        Rect::from_center_size(
            Pos2::new(
                part_rect.max.x - 10.0 * self.zoom,
                part_rect.min.y + title_height * 0.5,
            ),
            Vec2::splat(20.0 * self.zoom),
        )
    }

    pub fn draw_part_with_delete(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        part: &ModulePart,
        rect: Rect,
        actions: &mut Vec<UIAction>,
        module_id: ModuleId,
    ) {
        // Get part color and name based on type
        let (_bg_color, title_color, icon, name) = Self::get_part_style(&part.part_type);
        let category = Self::get_part_category(&part.part_type);

        // Check if this is an audio trigger and if it's active
        let (is_audio_trigger, audio_trigger_value, threshold, is_audio_active) =
            self.get_audio_trigger_state(&part.part_type);

        // Check generic trigger value from evaluator
        let generic_trigger_value = self
            .last_trigger_values
            .get(&part.id)
            .copied()
            .unwrap_or(0.0);
        let is_generic_active = generic_trigger_value > 0.1;

        // Combine
        let trigger_value = if is_generic_active {
            generic_trigger_value
        } else {
            audio_trigger_value
        };
        let is_active = is_audio_active || is_generic_active;

        // Draw glow effect if active
        if is_active {
            let glow_intensity = (trigger_value * 2.0).min(1.0);
            let base_color =
                Color32::from_rgba_unmultiplied(255, (160.0 * glow_intensity) as u8, 0, 255);

            // Cyber-Glow: Multi-layered sharp strokes
            for i in 1..=4 {
                let expansion = i as f32 * 1.5 * self.zoom;
                let alpha = (100.0 / (i as f32)).min(255.0) as u8;
                let color = base_color
                    .linear_multiply(glow_intensity)
                    .gamma_multiply(alpha as f32 / 255.0);

                painter.rect_stroke(
                    rect.expand(expansion),
                    0.0,
                    Stroke::new(1.0 * self.zoom, color),
                    egui::StrokeKind::Middle,
                );
            }

            // Inner "Light" border
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(
                    2.0 * self.zoom,
                    Color32::WHITE.gamma_multiply(180.0 * glow_intensity / 255.0),
                ),
                egui::StrokeKind::Middle,
            );
        }

        // MIDI Learn Highlight
        let is_midi_learn = self.midi_learn_part_id == Some(part.id);
        if is_midi_learn {
            let time = ui.input(|i| i.time);
            let pulse = (time * 8.0).sin().abs() as f32;
            let learn_color = Color32::from_rgb(0, 200, 255).linear_multiply(pulse);

            painter.rect_stroke(
                rect.expand(4.0 * self.zoom),
                0.0,
                Stroke::new(2.0 * self.zoom, learn_color),
                egui::StrokeKind::Middle,
            );

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "WAITING FOR MIDI...",
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE.gamma_multiply(200.0 * pulse / 255.0),
            );
        }

        // Draw background (Dark Neutral for high contrast)
        // We use a very dark grey/black to make the content pop
        let neutral_bg = colors::DARK_GREY;
        // Sharp corners for "Cyber" look
        painter.rect_filled(rect, 0.0, neutral_bg);

        // Handle drag and drop for Media Files
        if let ModulePartType::Source(
            SourceType::MediaFile { .. },
        ) = &part.part_type
        {
            if ui.rect_contains_pointer(rect) {
                if let Some(dropped_path) = ui
                    .ctx()
                    .data(|d| d.get_temp::<std::path::PathBuf>(egui::Id::new("media_path")))
                {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Middle,
                    );

                    if ui.input(|i| i.pointer.any_released()) {
                        actions.push(UIAction::SetMediaFile(
                            module_id,
                            part.id,
                            dropped_path.to_string_lossy().to_string(),
                        ));
                    }
                }
            }
        }

        // Node border - colored by type for quick identification
        // This replaces the generic gray border
        painter.rect_stroke(
            rect,
            0.0, // Sharp corners
            Stroke::new(1.5 * self.zoom, title_color.linear_multiply(0.8)),
            egui::StrokeKind::Middle,
        );

        // Title bar
        let title_height = 28.0 * self.zoom;
        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));

        // Title bar background (Dark)
        painter.rect_filled(
            title_rect,
            0.0, // Sharp corners
            colors::LIGHTER_GREY,
        );

        // Title bar Top Accent Stripe (Type Identifier)
        let stripe_height = 3.0 * self.zoom;
        let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), stripe_height));
        painter.rect_filled(stripe_rect, 0.0, title_color);

        // Title separator line - make it sharper
        painter.line_segment(
            [
                Pos2::new(rect.min.x, rect.min.y + title_height),
                Pos2::new(rect.max.x, rect.min.y + title_height),
            ],
            Stroke::new(1.0, colors::STROKE_GREY),
        );

        // Enhanced Title Rendering (Icon | Category | Name)
        let mut cursor_x = rect.min.x + 8.0 * self.zoom;
        let center_y = title_rect.center().y;

        // 1. Icon
        let icon_galley = ui.painter().layout_no_wrap(
            icon.to_string(),
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::WHITE,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - icon_galley.size().y / 2.0),
            icon_galley.clone(),
            Color32::WHITE,
        );
        cursor_x += icon_galley.size().x + 6.0 * self.zoom;

        // 2. Category (Small Caps style, Dimmed)
        let category_text = category.to_uppercase();
        let category_color = Color32::from_white_alpha(160);
        let category_galley = ui.painter().layout_no_wrap(
            category_text,
            egui::FontId::proportional(10.0 * self.zoom),
            category_color,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - category_galley.size().y / 2.0),
            category_galley.clone(),
            category_color,
        );
        cursor_x += category_galley.size().x + 6.0 * self.zoom;

        // 3. Name (Bold/Bright)
        let name_galley = ui.painter().layout_no_wrap(
            name.to_string(),
            egui::FontId::proportional(14.0 * self.zoom),
            Color32::WHITE,
        );
        painter.galley(
            Pos2::new(cursor_x, center_y - name_galley.size().y / 2.0),
            name_galley,
            Color32::WHITE,
        );

        // Delete button (x in top-right corner)
        let delete_button_rect = self.get_delete_button_rect(rect);

        // Retrieve hold progress for visualization (Mary StyleUX)
        let delete_id = egui::Id::new((part.id, "delete"));
        let progress = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(delete_id.with("progress")))
            .unwrap_or(0.0);

        crate::widgets::custom::draw_safety_radial_fill(
            painter,
            delete_button_rect.center(),
            10.0 * self.zoom,
            progress,
            Color32::from_rgb(255, 50, 50),
        );

        painter.text(
            delete_button_rect.center(),
            egui::Align2::CENTER_CENTER,
            "x",
            egui::FontId::proportional(16.0 * self.zoom),
            Color32::from_rgba_unmultiplied(255, 100, 100, 200),
        );

        // Draw property display based on part type
        let property_text = Self::get_part_property_text(&part.part_type);
        let has_property_text = !property_text.is_empty();

        if has_property_text {
            // Position at the bottom of the node to avoid overlapping sockets
            let property_y = rect.max.y - 10.0 * self.zoom;
            painter.text(
                Pos2::new(rect.center().x, property_y),
                egui::Align2::CENTER_CENTER,
                property_text,
                egui::FontId::proportional(10.0 * self.zoom),
                Color32::from_gray(180), // Slightly brighter for readability
            );
        }

        // Draw Media Playback Progress Bar
        if let ModulePartType::Source(
            SourceType::MediaFile { .. },
        ) = &part.part_type
        {
            if let Some(info) = self.player_info.get(&part.id) {
                let duration = info.duration.max(0.001);
                let progress = (info.current_time / duration).clamp(0.0, 1.0) as f32;
                let is_playing = info.is_playing;

                let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
                let bar_height = 4.0 * self.zoom;
                let bar_y = rect.max.y - (offset_from_bottom * self.zoom) - bar_height;
                let bar_width = rect.width() - 20.0 * self.zoom;
                let bar_x = rect.min.x + 10.0 * self.zoom;

                // Background
                let bar_bg =
                    Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));
                painter.rect_filled(bar_bg, 2.0 * self.zoom, Color32::from_gray(30));

                // Progress
                let progress_width = (progress * bar_width).max(2.0 * self.zoom);
                let progress_rect = Rect::from_min_size(
                    Pos2::new(bar_x, bar_y),
                    Vec2::new(progress_width, bar_height),
                );

                let color = if is_playing {
                    Color32::from_rgb(100, 255, 100) // Green
                } else {
                    Color32::from_rgb(255, 200, 50) // Yellow/Orange
                };

                painter.rect_filled(progress_rect, 2.0 * self.zoom, color);

                // Interaction (Seek)
                let interact_rect = bar_bg.expand(6.0 * self.zoom);
                let bar_response = ui.interact(
                    interact_rect,
                    ui.id().with(("seek", part.id)),
                    Sense::click_and_drag(),
                );

                if bar_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if bar_response.clicked() || bar_response.dragged() {
                    if let Some(pos) = bar_response.interact_pointer_pos() {
                        let seek_norm = ((pos.x - bar_x) / bar_width).clamp(0.0, 1.0);
                        let seek_s = seek_norm as f64 * duration;
                        actions.push(UIAction::MediaCommand(
                            part.id,
                            MediaPlaybackCommand::Seek(seek_s),
                        ));
                    }
                }
            }
        }

        // Draw audio trigger VU meter and live value display
        if is_audio_trigger {
            let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
            let meter_height = 4.0 * self.zoom; // Thinner meter
            let meter_y = rect.max.y - (offset_from_bottom * self.zoom) - meter_height;
            let meter_width = rect.width() - 20.0 * self.zoom;
            let meter_x = rect.min.x + 10.0 * self.zoom;

            // Background bar
            let meter_bg = Rect::from_min_size(
                Pos2::new(meter_x, meter_y),
                Vec2::new(meter_width, meter_height),
            );
            painter.rect_filled(meter_bg, 2.0, Color32::from_gray(20));

            // Value bar with Hardware-Segments
            let num_segments = 20;
            let segment_spacing = 1.0 * self.zoom;
            let segment_width =
                (meter_width - (num_segments as f32 - 1.0) * segment_spacing) / num_segments as f32;

            for i in 0..num_segments {
                let t = i as f32 / num_segments as f32;
                if t > trigger_value {
                    break;
                }

                let seg_x = meter_x + i as f32 * (segment_width + segment_spacing);
                let seg_rect = Rect::from_min_size(
                    Pos2::new(seg_x, meter_y),
                    Vec2::new(segment_width, meter_height),
                );

                let seg_color = if t < 0.6 {
                    Color32::from_rgb(0, 255, 100) // Green
                } else if t < 0.85 {
                    Color32::from_rgb(255, 180, 0) // Orange
                } else {
                    Color32::from_rgb(255, 50, 50) // Red
                };

                painter.rect_filled(seg_rect, 1.0, seg_color);
            }

            // Threshold line
            let threshold_x = meter_x + threshold * meter_width;
            painter.line_segment(
                [
                    Pos2::new(threshold_x, meter_y - 2.0),
                    Pos2::new(threshold_x, meter_y + meter_height + 2.0),
                ],
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 50, 50, 200)),
            );
        }

        // Draw input sockets (left side)
        let socket_start_y = rect.min.y + title_height + 10.0 * self.zoom;
        for (i, socket) in part.inputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.min.x, socket_y);
            let socket_radius = 7.0 * self.zoom;

            // Socket "Port" style (dark hole with colored ring)
            let socket_color = Self::get_socket_color(&socket.socket_type);

            // Check hover
            let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                socket_pos.distance(pointer_pos) < socket_radius * 1.5
            } else {
                false
            };

            // Outer ring (Socket Color)
            let ring_stroke = if is_hovered {
                let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
                Stroke::new(3.0 * self.zoom, Color32::WHITE.linear_multiply(pulse))
            } else {
                Stroke::new(2.0 * self.zoom, socket_color)
            };
            painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
            // Inner hole (Dark)
            painter.circle_filled(
                socket_pos,
                socket_radius - 2.0 * self.zoom,
                Color32::from_gray(20),
            );
            // Inner dot (Connector contact)
            painter.circle_filled(
                socket_pos,
                2.0 * self.zoom,
                if is_hovered {
                    socket_color
                } else {
                    Color32::from_gray(100)
                },
            );

            // Socket label
            painter.text(
                Pos2::new(rect.min.x + 14.0 * self.zoom, socket_y),
                egui::Align2::LEFT_CENTER,
                &socket.name,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::from_gray(230), // Brighter text
            );
        }

        // Draw output sockets (right side)
        for (i, socket) in part.outputs.iter().enumerate() {
            let socket_y = socket_start_y + i as f32 * 22.0 * self.zoom;
            let socket_pos = Pos2::new(rect.max.x, socket_y);
            let socket_radius = 7.0 * self.zoom;

            // Socket "Port" style
            let socket_color = Self::get_socket_color(&socket.socket_type);

            // Check hover
            let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                socket_pos.distance(pointer_pos) < socket_radius * 1.5
            } else {
                false
            };

            // Outer ring (Socket Color)
            let ring_stroke = if is_hovered {
                let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
                Stroke::new(3.0 * self.zoom, Color32::WHITE.linear_multiply(pulse))
            } else {
                Stroke::new(2.0 * self.zoom, socket_color)
            };
            painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
            // Inner hole (Dark)
            painter.circle_filled(
                socket_pos,
                socket_radius - 2.0 * self.zoom,
                Color32::from_gray(20),
            );
            // Inner dot (Connector contact)
            painter.circle_filled(
                socket_pos,
                2.0 * self.zoom,
                if is_hovered {
                    socket_color
                } else {
                    Color32::from_gray(100)
                },
            );

            // Socket label
            painter.text(
                Pos2::new(rect.max.x - 14.0 * self.zoom, socket_y),
                egui::Align2::RIGHT_CENTER,
                &socket.name,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::from_gray(230), // Brighter text
            );

            // Draw live value meter for output sockets
            if let Some(value) = self.get_socket_live_value(part, i) {
                let meter_width = 30.0 * self.zoom;
                let meter_height = 8.0 * self.zoom;
                let meter_x = rect.max.x - 12.0 * self.zoom - meter_width;

                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, socket_y - meter_height / 2.0),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_gray(40));

                let value_width = (value.clamp(0.0, 1.0) * meter_width).max(1.0);
                let value_bar = Rect::from_min_size(
                    Pos2::new(meter_x, socket_y - meter_height / 2.0),
                    Vec2::new(value_width, meter_height),
                );
                painter.rect_filled(value_bar, 2.0, Color32::from_rgb(100, 180, 220));
            }
        }
    }

    pub fn get_part_style(
        part_type: &ModulePartType,
    ) -> (Color32, Color32, &'static str, &'static str) {
        match part_type {
            ModulePartType::Trigger(trigger) => {
                let name = match trigger {
                    TriggerType::AudioFFT { .. } => "Audio FFT",
                    TriggerType::Beat => "Beat",
                    TriggerType::Midi { .. } => "MIDI",
                    TriggerType::Osc { .. } => "OSC",
                    TriggerType::Shortcut { .. } => "Shortcut",
                    TriggerType::Random { .. } => "Random",
                    TriggerType::Fixed { .. } => "Fixed Timer",
                };
                (
                    Color32::from_rgb(60, 50, 70),
                    Color32::from_rgb(130, 80, 180),
                    "\u{26A1}",
                    name,
                )
            }
            ModulePartType::Source(SourceType::BevyAtmosphere { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "â˜ ï¸ ",
                "Atmosphere",
            ),
            ModulePartType::Source(SourceType::BevyHexGrid { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "\u{1F6D1}",
                "Hex Grid",
            ),
            ModulePartType::Source(SourceType::BevyParticles { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "\u{2728}",
                "Particles",
            ),
            ModulePartType::Source(SourceType::Bevy3DText { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 220, 180),
                "T",
                "3D Text",
            ),
            ModulePartType::Source(SourceType::BevyCamera { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(180, 100, 220),
                "\u{1F3A5}",
                "Camera",
            ),
            ModulePartType::Source(SourceType::Bevy3DShape { .. }) => (
                Color32::from_rgb(40, 60, 80),
                Color32::from_rgb(100, 180, 220),
                "\u{1F9CA}",
                "3D Shape",
            ),
            ModulePartType::Source(source) => {
                let name = match source {
                    SourceType::MediaFile { .. } => "Media File",
                    SourceType::Shader { .. } => "Shader",
                    SourceType::LiveInput { .. } => "Live Input",
                    SourceType::NdiInput { .. } => "NDI Input",
                    #[cfg(target_os = "windows")]
                    SourceType::SpoutInput { .. } => "Spout Input",
                    SourceType::VideoUni { .. } => "Video (Uni)",
                    SourceType::ImageUni { .. } => "Image (Uni)",
                    SourceType::VideoMulti { .. } => "Video (Multi)",
                    SourceType::ImageMulti { .. } => "Image (Multi)",
                    SourceType::Bevy => "Bevy Scene",
                    SourceType::BevyAtmosphere { .. } => "Atmosphere",
                    SourceType::BevyHexGrid { .. } => "Hex Grid",
                    SourceType::BevyParticles { .. } => "Particles",
                    SourceType::Bevy3DText { .. } => "3D Text",
                    SourceType::BevyCamera { .. } => "Camera",
                    SourceType::Bevy3DShape { .. } => "3D Shape",
                    SourceType::Bevy3DModel { .. } => "3D Model",
                };
                (
                    Color32::from_rgb(50, 60, 70),
                    Color32::from_rgb(80, 140, 180),
                    "\u{1F3AC}",
                    name,
                )
            }

            ModulePartType::Mask(mask) => {
                let name = match mask {
                    MaskType::File { .. } => "File Mask",
                    MaskType::Shape(shape) => match shape {
                        MaskShape::Circle => "Circle",
                        MaskShape::Rectangle => "Rectangle",
                        MaskShape::Triangle => "Triangle",
                        MaskShape::Star => "Star",
                        MaskShape::Ellipse => "Ellipse",
                    },
                    MaskType::Gradient { .. } => "Gradient",
                };
                (
                    Color32::from_rgb(60, 55, 70),
                    Color32::from_rgb(160, 100, 180),
                    "\u{1F3AD}",
                    name,
                )
            }
            ModulePartType::Modulizer(mod_type) => {
                let name = match mod_type {
                    ModulizerType::Effect {
                        effect_type: effect,
                        ..
                    } => match effect {
                        EffectType::Blur => "Blur",
                        EffectType::Sharpen => "Sharpen",
                        EffectType::Invert => "Invert",
                        EffectType::Threshold => "Threshold",
                        EffectType::Brightness => "Brightness",
                        EffectType::Contrast => "Contrast",
                        EffectType::Saturation => "Saturation",
                        EffectType::HueShift => "Hue Shift",
                        EffectType::Colorize => "Colorize",
                        EffectType::Wave => "Wave",
                        EffectType::Spiral => "Spiral",
                        EffectType::Pinch => "Pinch",
                        EffectType::Mirror => "Mirror",
                        EffectType::Kaleidoscope => "Kaleidoscope",
                        EffectType::Pixelate => "Pixelate",
                        EffectType::Halftone => "Halftone",
                        EffectType::EdgeDetect => "Edge Detect",
                        EffectType::Posterize => "Posterize",
                        EffectType::Glitch => "Glitch",
                        EffectType::RgbSplit => "RGB Split",
                        EffectType::ChromaticAberration => "Chromatic",
                        EffectType::VHS => "VHS",
                        EffectType::FilmGrain => "Film Grain",
                        EffectType::Vignette => "Vignette",
                        EffectType::ShaderGraph(_) => "Custom Graph",
                    },
                    ModulizerType::BlendMode(blend) => match blend {
                        BlendModeType::Normal => "Normal",
                        BlendModeType::Add => "Add",
                        BlendModeType::Multiply => "Multiply",
                        BlendModeType::Screen => "Screen",
                        BlendModeType::Overlay => "Overlay",
                        BlendModeType::Difference => "Difference",
                        BlendModeType::Exclusion => "Exclusion",
                    },
                    ModulizerType::AudioReactive { .. } => "Audio Reactive",
                };
                (
                    egui::Color32::from_rgb(60, 60, 50),
                    egui::Color32::from_rgb(180, 140, 60),
                    "ã€°ï¸ ",
                    name,
                )
            }
            ModulePartType::Mesh(_) => (
                egui::Color32::from_rgb(60, 60, 80),
                egui::Color32::from_rgb(100, 100, 200),
                "🕸️ï¸ ",
                "Mesh",
            ),
            ModulePartType::Layer(layer) => {
                let name = match layer {
                    LayerType::Single { .. } => "Single Layer",
                    LayerType::Group { .. } => "Layer Group",
                    LayerType::All { .. } => "All Layers",
                };
                (
                    Color32::from_rgb(50, 70, 60),
                    Color32::from_rgb(80, 180, 120),
                    "\u{1F4D1}",
                    name,
                )
            }
            ModulePartType::Output(output) => {
                let name = match output {
                    OutputType::Projector { .. } => "Projector",
                    OutputType::NdiOutput { .. } => "NDI Output",
                    #[cfg(target_os = "windows")]
                    OutputType::Spout { .. } => "Spout Output",
                    OutputType::Hue { .. } => "Philips Hue",
                };
                (
                    Color32::from_rgb(70, 50, 50),
                    Color32::from_rgb(180, 80, 80),
                    "\u{1F4FA}",
                    name,
                )
            }
            ModulePartType::Hue(hue) => {
                let name = match hue {
                    HueNodeType::SingleLamp { .. } => "Single Lamp",
                    HueNodeType::MultiLamp { .. } => "Multi Lamp",
                    HueNodeType::EntertainmentGroup { .. } => {
                        "Entertainment Group"
                    }
                };
                (
                    Color32::from_rgb(60, 60, 40),
                    Color32::from_rgb(200, 200, 100),
                    "\u{1F4A1}",
                    name,
                )
            }
        }
    }

    pub fn get_part_category(part_type: &ModulePartType) -> &'static str {
        match part_type {
            ModulePartType::Trigger(_) => "Trigger",
            ModulePartType::Source(_) => "Source",
            ModulePartType::Mask(_) => "Mask",
            ModulePartType::Modulizer(_) => "Modulator",
            ModulePartType::Mesh(_) => "Mesh",
            ModulePartType::Layer(_) => "Layer",
            ModulePartType::Output(_) => "Output",
            ModulePartType::Hue(_) => "Hue",
        }
    }

    pub fn get_part_property_text(part_type: &ModulePartType) -> String {
        match part_type {
            ModulePartType::Trigger(trigger_type) => match trigger_type {
                TriggerType::AudioFFT { band, .. } => format!("\u{1F50A} Audio: {:?}", band),
                TriggerType::Random { .. } => "\u{1F3B2} Random".to_string(),
                TriggerType::Fixed { interval_ms, .. } => format!("⏱️ï¸  {}ms", interval_ms),
                TriggerType::Midi { channel, note, .. } => {
                    format!("\u{1F3B9} Ch{} N{}", channel, note)
                }
                TriggerType::Osc { address } => format!("\u{1F4E1} {}", address),
                TriggerType::Shortcut { key_code, .. } => format!("âŒ¨ï¸  {}", key_code),
                TriggerType::Beat => "🥁 Beat".to_string(),
            },
            ModulePartType::Source(source_type) => match source_type {
                SourceType::MediaFile { path, .. } => {
                    if path.is_empty() {
                        "📁 Select file...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                SourceType::Shader { name, .. } => format!("\u{1F3A8} {}", name),
                SourceType::LiveInput { device_id } => format!("\u{1F4F9} Device {}", device_id),
                SourceType::NdiInput { source_name } => {
                    format!("\u{1F4E1} {}", source_name.as_deref().unwrap_or("None"))
                }
                SourceType::Bevy => "\u{1F3AE} Bevy Scene".to_string(),
                #[cfg(target_os = "windows")]
                SourceType::SpoutInput { sender_name } => format!("\u{1F6B0} {}", sender_name),
                SourceType::VideoUni { path, .. } => {
                    if path.is_empty() {
                        "📁 Select video...".to_string()
                    } else {
                        format!(
                            "\u{1F4F9} {}",
                            path.split(['/', '\\']).next_back().unwrap_or(path)
                        )
                    }
                }
                SourceType::ImageUni { path, .. } => {
                    if path.is_empty() {
                        "\u{1F5BC} Select image...".to_string()
                    } else {
                        format!(
                            "\u{1F5BC} {}",
                            path.split(['/', '\\']).next_back().unwrap_or(path)
                        )
                    }
                }
                SourceType::VideoMulti { shared_id, .. } => {
                    format!("\u{1F4F9} Shared: {}", shared_id)
                }
                SourceType::ImageMulti { shared_id, .. } => {
                    format!("\u{1F5BC} Shared: {}", shared_id)
                }
                SourceType::BevyAtmosphere { .. } => "â˜ ï¸  Atmosphere".to_string(),
                SourceType::BevyHexGrid { .. } => "\u{1F6D1} Hex Grid".to_string(),
                SourceType::BevyParticles { .. } => "\u{2728} Particles".to_string(),
                SourceType::Bevy3DText { text, .. } => {
                    format!("T: {}", text.chars().take(10).collect::<String>())
                }
                SourceType::BevyCamera { mode, .. } => match mode {
                    mapmap_core::module::BevyCameraMode::Orbit { .. } => "\u{1F3A5} Orbit".to_string(),
                    mapmap_core::module::BevyCameraMode::Fly { .. } => "\u{1F3A5} Fly".to_string(),
                    mapmap_core::module::BevyCameraMode::Static { .. } => "\u{1F3A5} Static".to_string(),
                },
                SourceType::Bevy3DShape { shape_type, .. } => format!("\u{1F9CA} {:?}", shape_type),
                SourceType::Bevy3DModel { path, .. } => format!("\u{1F3AE} Model: {}", path),
            },
            ModulePartType::Mask(mask_type) => match mask_type {
                MaskType::File { path } => {
                    if path.is_empty() {
                        "📁 Select mask...".to_string()
                    } else {
                        format!("📁 {}", path.split(['/', '\\']).next_back().unwrap_or(path))
                    }
                }
                MaskType::Shape(shape) => format!("\u{1F537} {:?}", shape),
                MaskType::Gradient { angle, .. } => {
                    format!("\u{1F308} Gradient {}Â°", *angle as i32)
                }
            },
            ModulePartType::Modulizer(modulizer_type) => match modulizer_type {
                ModulizerType::Effect {
                    effect_type: effect,
                    ..
                } => format!("\u{2728} {}", effect.name()),
                ModulizerType::BlendMode(blend) => format!("🔄 {}", blend.name()),
                ModulizerType::AudioReactive { source } => format!("\u{1F50A} {}", source),
            },
            ModulePartType::Mesh(_) => "🕸️ï¸  Mesh".to_string(),
            ModulePartType::Layer(layer_type) => {
                match layer_type {
                    LayerType::Single { name, .. } => format!("\u{1F4D1} {}", name),
                    LayerType::Group { name, .. } => format!("📁 {}", name),
                    LayerType::All { .. } => "\u{1F4D1} All Layers".to_string(),
                }
            }
            ModulePartType::Output(output_type) => match output_type {
                OutputType::Projector { name, .. } => format!("\u{1F4FA} {}", name),
                OutputType::NdiOutput { name } => format!("\u{1F4E1} {}", name),
                #[cfg(target_os = "windows")]
                OutputType::Spout { name } => format!("\u{1F6B0} {}", name),
                OutputType::Hue { bridge_ip, .. } => {
                    if bridge_ip.is_empty() {
                        "\u{1F4A1} Not Connected".to_string()
                    } else {
                        format!("\u{1F4A1} {}", bridge_ip)
                    }
                }
            },
            ModulePartType::Hue(hue) => match hue {
                HueNodeType::SingleLamp { name, .. } => {
                    format!("\u{1F4A1} {}", name)
                }
                HueNodeType::MultiLamp { name, .. } => {
                    format!("\u{1F4A1}\u{1F4A1} {}", name)
                }
                HueNodeType::EntertainmentGroup { name, .. } => {
                    format!("\u{1F3AD} {}", name)
                }
            },
        }
    }

    /// Get the live value of a specific output socket on a part.
    fn get_socket_live_value(&self, part: &ModulePart, socket_idx: usize) -> Option<f32> {
        if let ModulePartType::Trigger(TriggerType::AudioFFT { .. }) = &part.part_type {
            // The 9 frequency bands are the first 9 outputs
            if socket_idx < 9 {
                return Some(self.audio_trigger_data.band_energies[socket_idx]);
            }
            // After the bands, we have RMS, Peak, Beat, BPM
            match socket_idx {
                9 => return Some(self.audio_trigger_data.rms_volume),
                10 => return Some(self.audio_trigger_data.peak_volume),
                11 => return Some(self.audio_trigger_data.beat_strength),
                12 => return self.audio_trigger_data.bpm,
                _ => return None,
            }
        }
        None
    }

    /// Get current RMS volume
    pub fn get_rms_volume(&self) -> f32 {
        self.audio_trigger_data.rms_volume
    }

    /// Get beat detection status
    pub fn is_beat_detected(&self) -> bool {
        self.audio_trigger_data.beat_detected
    }

    /// Get audio trigger state for a part type
    fn get_audio_trigger_state(
        &self,
        part_type: &ModulePartType,
    ) -> (bool, f32, f32, bool) {
        match part_type {
            ModulePartType::Trigger(TriggerType::AudioFFT {
                band, threshold, ..
            }) => {
                let value = match band {
                    AudioBand::SubBass => self.audio_trigger_data.band_energies.first().copied().unwrap_or(0.0),
                    AudioBand::Bass => self.audio_trigger_data.band_energies.get(1).copied().unwrap_or(0.0),
                    AudioBand::LowMid => self.audio_trigger_data.band_energies.get(2).copied().unwrap_or(0.0),
                    AudioBand::Mid => self.audio_trigger_data.band_energies.get(3).copied().unwrap_or(0.0),
                    AudioBand::HighMid => self.audio_trigger_data.band_energies.get(4).copied().unwrap_or(0.0),
                    AudioBand::UpperMid => self.audio_trigger_data.band_energies.get(5).copied().unwrap_or(0.0),
                    AudioBand::Presence => self.audio_trigger_data.band_energies.get(6).copied().unwrap_or(0.0),
                    AudioBand::Brilliance => self.audio_trigger_data.band_energies.get(7).copied().unwrap_or(0.0),
                    AudioBand::Air => self.audio_trigger_data.band_energies.get(8).copied().unwrap_or(0.0),
                    AudioBand::Peak => self.audio_trigger_data.peak_volume,
                    AudioBand::BPM => self.audio_trigger_data.bpm.unwrap_or(0.0) / 200.0,
                };
                let is_active = value > *threshold;
                (true, value, *threshold, is_active)
            }
            ModulePartType::Trigger(TriggerType::Beat) => {
                let is_active = self.audio_trigger_data.beat_detected;
                let value = self.audio_trigger_data.beat_strength;
                (true, value, 0.5, is_active)
            }
            _ => (false, 0.0, 0.0, false),
        }
    }

    /// Auto-layout parts in a grid by type
    pub fn auto_layout_parts(parts: &mut [ModulePart]) {
        // Sort parts by type category for left-to-right flow
        let type_order = |pt: &ModulePartType| -> usize {
            match pt {
                ModulePartType::Trigger(_) => 0,
                ModulePartType::Source(_) => 1,
                ModulePartType::Mask(_) => 2,
                ModulePartType::Modulizer(_) => 3,
                ModulePartType::Mesh(_) => 4,
                ModulePartType::Layer(_) => 5,
                ModulePartType::Output(_) => 6,
                ModulePartType::Hue(_) => 7,
            }
        };

        // Group parts by type
        let mut columns: [Vec<usize>; 8] = Default::default();
        for (i, part) in parts.iter().enumerate() {
            let col = type_order(&part.part_type);
            columns[col].push(i);
        }

        // Layout parameters
        let node_width = 200.0;
        let node_height = 120.0;
        let h_spacing = 100.0;
        let v_spacing = 60.0;
        let start_x = 50.0;
        let start_y = 50.0;

        // Position each column
        let mut x = start_x;
        for col in &columns {
            if col.is_empty() {
                continue;
            }

            let mut y = start_y;
            for &part_idx in col {
                parts[part_idx].position = (x, y);
                y += node_height + v_spacing;
            }

            x += node_width + h_spacing;
        }
    }

    /// Find a free position for a new node
    pub fn find_free_position(
        parts: &[ModulePart],
        preferred: (f32, f32),
    ) -> (f32, f32) {
        let node_width = 200.0;
        let node_height = 130.0;
        let grid_step = 30.0;

        let mut pos = preferred;
        let mut attempts = 0;

        loop {
            let new_rect =
                Rect::from_min_size(Pos2::new(pos.0, pos.1), Vec2::new(node_width, node_height));

            let has_collision = parts.iter().any(|part| {
                let part_height = 80.0 + (part.inputs.len().max(part.outputs.len()) as f32) * 20.0;
                let part_rect = Rect::from_min_size(
                    Pos2::new(part.position.0, part.position.1),
                    Vec2::new(node_width, part_height),
                );
                new_rect.intersects(part_rect)
            });

            if !has_collision {
                return pos;
            }

            attempts += 1;
            if attempts > 100 {
                return (preferred.0, preferred.1 + (parts.len() as f32) * 150.0);
            }

            pos.1 += grid_step;
            if pos.1 > preferred.1 + 500.0 {
                pos.1 = preferred.1;
                pos.0 += node_width + 20.0;
            }
        }
    }
}
