#![allow(clippy::too_many_arguments)]
use super::super::{state::ModuleCanvas, utils};
use crate::theme::colors;
use crate::UIAction;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::module::{ModuleId, ModulePart, ModulePartType, TriggerType};

pub fn draw_part_with_delete(
    canvas: &ModuleCanvas,
    ui: &Ui,
    painter: &egui::Painter,
    part: &ModulePart,
    rect: Rect,
    actions: &mut Vec<UIAction>,
    module_id: ModuleId,
    meter_style: crate::config::AudioMeterStyle,
    node_animations_enabled: bool,
    animation_profile: crate::config::AnimationProfile,
) {
    let (_bg_color, title_color, icon, name) = utils::get_part_style(&part.part_type);
    let category = utils::get_part_category(&part.part_type);

    let is_audio_trigger = matches!(
        part.part_type,
        ModulePartType::Trigger(TriggerType::AudioFFT { .. })
    );
    let mut audio_trigger_value = 0.0;
    let mut threshold = 0.0;
    let mut is_audio_active = false;

    if let ModulePartType::Trigger(TriggerType::AudioFFT {
        band, threshold: t, ..
    }) = &part.part_type
    {
        threshold = *t;
        let index = *band as usize;
        if let Some(val) = canvas.audio_trigger_data.band_energies.get(index) {
            audio_trigger_value = *val;
            is_audio_active = audio_trigger_value > threshold;
        }
    }

    let generic_trigger_value = canvas
        .last_trigger_values
        .get(&part.id)
        .copied()
        .unwrap_or(0.0);
    let is_generic_active = generic_trigger_value > 0.1;

    let trigger_value = if is_generic_active {
        generic_trigger_value
    } else {
        audio_trigger_value
    };
    let is_active = is_audio_active || is_generic_active;

    if node_animations_enabled
        && animation_profile != crate::config::AnimationProfile::Off
        && is_active
    {
        let glow_intensity = (trigger_value * 2.0).min(1.0);
        let base_color =
            Color32::from_rgba_unmultiplied(255, (160.0 * glow_intensity) as u8, 0, 255);

        for i in 1..=4 {
            let expansion = i as f32 * 1.5 * canvas.zoom;
            let alpha = (100.0 / (i as f32)).min(255.0) as u8;
            let color = base_color
                .linear_multiply(glow_intensity)
                .gamma_multiply(alpha as f32 / 255.0);

            painter.rect_stroke(
                rect.expand(expansion),
                0.0,
                Stroke::new(1.0 * canvas.zoom, color),
                egui::StrokeKind::Middle,
            );
        }

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(
                2.0 * canvas.zoom,
                Color32::WHITE.gamma_multiply(180.0 * glow_intensity / 255.0),
            ),
            egui::StrokeKind::Middle,
        );
    }

    let is_midi_learn = canvas.midi_learn_part_id == Some(part.id);
    if is_midi_learn {
        let time = ui.input(|i| i.time);
        let pulse = (time * 8.0).sin().abs() as f32;
        let learn_color = Color32::from_rgb(0, 200, 255).linear_multiply(pulse);

        painter.rect_stroke(
            rect.expand(4.0 * canvas.zoom),
            0.0,
            Stroke::new(2.0 * canvas.zoom, learn_color),
            egui::StrokeKind::Middle,
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "WAITING FOR MIDI...",
            egui::FontId::proportional(12.0 * canvas.zoom),
            Color32::WHITE.gamma_multiply(200.0 * pulse / 255.0),
        );
    }

    let neutral_bg = colors::DARK_GREY;
    painter.rect_filled(rect, 0.0, neutral_bg);

    if node_animations_enabled && animation_profile != crate::config::AnimationProfile::Off {
        let time = ui.input(|i| i.time) as f32;
        let profile_scale = match animation_profile {
            crate::config::AnimationProfile::Subtle => 0.8,
            crate::config::AnimationProfile::Cinematic => 1.35,
            crate::config::AnimationProfile::Off => 0.0,
        };
        let (anim_speed, anim_color) = match &part.part_type {
            ModulePartType::Source(_) => (0.9, Color32::from_rgba_unmultiplied(0, 210, 255, 32)),
            ModulePartType::Modulizer(_) => {
                (1.6, Color32::from_rgba_unmultiplied(255, 100, 220, 28))
            }
            ModulePartType::Trigger(_) => (2.3, Color32::from_rgba_unmultiplied(255, 170, 80, 38)),
            ModulePartType::Output(_) => (1.2, Color32::from_rgba_unmultiplied(140, 255, 140, 24)),
            ModulePartType::Layer(_) | ModulePartType::Mask(_) => {
                (1.35, Color32::from_rgba_unmultiplied(190, 170, 255, 24))
            }
            _ => (1.0, Color32::from_rgba_unmultiplied(180, 200, 255, 20)),
        };
        let phase = (time * (anim_speed * profile_scale) + part.id as f32 * 0.11)
            .sin()
            .abs();
        let pulse_w = 1.2 * canvas.zoom + phase * (2.4 * profile_scale) * canvas.zoom;
        painter.rect_stroke(
            rect.expand(1.5 * canvas.zoom),
            0.0,
            Stroke::new(pulse_w, anim_color.gamma_multiply(0.45 + phase * 0.55)),
            egui::StrokeKind::Middle,
        );
    }

    if let mapmap_core::module::ModulePartType::Source(
        mapmap_core::module::SourceType::MediaFile { .. },
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

    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.5 * canvas.zoom, title_color.linear_multiply(0.8)),
        egui::StrokeKind::Middle,
    );

    let title_height = 28.0 * canvas.zoom;
    let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), title_height));

    painter.rect_filled(title_rect, 0.0, colors::LIGHTER_GREY);

    let stripe_height = 3.0 * canvas.zoom;
    let stripe_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), stripe_height));
    painter.rect_filled(stripe_rect, 0.0, title_color);

    painter.line_segment(
        [
            Pos2::new(rect.min.x, rect.min.y + title_height),
            Pos2::new(rect.max.x, rect.min.y + title_height),
        ],
        Stroke::new(1.0, colors::STROKE_GREY),
    );

    let preview_rect = Rect::from_min_max(
        Pos2::new(
            rect.min.x + 2.0 * canvas.zoom,
            rect.min.y + title_height + 2.0 * canvas.zoom,
        ),
        Pos2::new(
            rect.max.x - 2.0 * canvas.zoom,
            rect.max.y - 2.0 * canvas.zoom,
        ),
    );

    if let Some(&texture_id) = canvas.node_previews.get(&(module_id, part.id)) {
        painter.image(
            texture_id,
            preview_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(preview_rect, 0.0, Color32::from_gray(15));
    }

    let mut cursor_x = rect.min.x + 8.0 * canvas.zoom;
    let center_y = title_rect.center().y;

    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(16.0 * canvas.zoom),
        Color32::WHITE,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - icon_galley.size().y / 2.0),
        icon_galley.clone(),
        Color32::WHITE,
    );
    cursor_x += icon_galley.size().x + 6.0 * canvas.zoom;

    let category_text = category.to_uppercase();
    let category_color = Color32::from_white_alpha(160);
    let category_galley = ui.painter().layout_no_wrap(
        category_text,
        egui::FontId::proportional(10.0 * canvas.zoom),
        category_color,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - category_galley.size().y / 2.0),
        category_galley.clone(),
        category_color,
    );
    cursor_x += category_galley.size().x + 6.0 * canvas.zoom;

    let name_galley = ui.painter().layout_no_wrap(
        name.to_string(),
        egui::FontId::proportional(14.0 * canvas.zoom),
        Color32::WHITE,
    );
    painter.galley(
        Pos2::new(cursor_x, center_y - name_galley.size().y / 2.0),
        name_galley,
        Color32::WHITE,
    );

    let delete_button_rect = get_delete_button_rect(canvas, rect);

    let delete_id = egui::Id::new((part.id, "delete"));
    let progress = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(delete_id.with("progress")))
        .unwrap_or(0.0);

    crate::widgets::custom::draw_safety_radial_fill(
        painter,
        delete_button_rect.center(),
        10.0 * canvas.zoom,
        progress,
        Color32::from_rgb(255, 50, 50),
    );

    painter.text(
        delete_button_rect.center(),
        egui::Align2::CENTER_CENTER,
        "x",
        egui::FontId::proportional(16.0 * canvas.zoom),
        Color32::from_rgba_unmultiplied(255, 100, 100, 200),
    );

    let property_text = utils::get_part_property_text(&part.part_type);
    let has_property_text = !property_text.is_empty();

    if has_property_text {
        let property_y = rect.max.y - 10.0 * canvas.zoom;
        painter.text(
            Pos2::new(rect.center().x, property_y),
            egui::Align2::CENTER_CENTER,
            property_text,
            egui::FontId::proportional(10.0 * canvas.zoom),
            Color32::from_gray(180),
        );
    }

    if let mapmap_core::module::ModulePartType::Source(
        mapmap_core::module::SourceType::MediaFile { .. },
    ) = &part.part_type
    {
        if let Some(info) = canvas.player_info.get(&part.id) {
            let duration = info.duration.max(0.001);
            let progress = (info.current_time / duration).clamp(0.0, 1.0) as f32;
            let is_playing = info.is_playing;

            let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };
            let bar_height = 4.0 * canvas.zoom;
            let bar_y = rect.max.y - (offset_from_bottom * canvas.zoom) - bar_height;
            let bar_width = rect.width() - 20.0 * canvas.zoom;
            let bar_x = rect.min.x + 10.0 * canvas.zoom;

            let bar_bg =
                Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));
            painter.rect_filled(bar_bg, 2.0 * canvas.zoom, Color32::from_gray(30));

            let progress_width = (progress * bar_width).max(2.0 * canvas.zoom);
            let progress_rect = Rect::from_min_size(
                Pos2::new(bar_x, bar_y),
                Vec2::new(progress_width, bar_height),
            );

            let color = if is_playing {
                Color32::from_rgb(100, 255, 100)
            } else {
                Color32::from_rgb(255, 200, 50)
            };

            painter.rect_filled(progress_rect, 2.0 * canvas.zoom, color);

            let interact_rect = bar_bg.expand(6.0 * canvas.zoom);
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
                        crate::editors::module_canvas::types::MediaPlaybackCommand::Seek(seek_s),
                    ));
                }
            }
        }
    }

    if is_audio_trigger {
        let offset_from_bottom = if has_property_text { 28.0 } else { 12.0 };

        let meter_height = match meter_style {
            crate::config::AudioMeterStyle::Retro => 12.0 * canvas.zoom,
            crate::config::AudioMeterStyle::Digital => 4.0 * canvas.zoom,
        };

        let meter_y = rect.max.y - (offset_from_bottom * canvas.zoom) - meter_height;
        let meter_width = rect.width() - 20.0 * canvas.zoom;
        let meter_x = rect.min.x + 10.0 * canvas.zoom;

        match meter_style {
            crate::config::AudioMeterStyle::Retro => {
                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, meter_y),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_rgb(230, 225, 210));

                let arc_rect = meter_bg.shrink(2.0 * canvas.zoom);
                let clamped_val = trigger_value.clamp(0.0, 1.0);
                let pivot = Pos2::new(meter_bg.center().x, meter_bg.max.y + meter_height * 0.5);
                let radius = meter_height * 1.5;

                let start_angle = -40.0_f32.to_radians();
                let end_angle = 40.0_f32.to_radians();
                let needle_angle = start_angle + (end_angle - start_angle) * clamped_val;

                let needle_tip =
                    pivot + Vec2::new(needle_angle.sin() * radius, -needle_angle.cos() * radius);

                let bounded_tip = Pos2::new(
                    needle_tip.x.clamp(arc_rect.min.x, arc_rect.max.x),
                    needle_tip.y.max(arc_rect.min.y),
                );

                painter.line_segment(
                    [
                        Pos2::new(meter_x + meter_width * 0.8, meter_y + meter_height * 0.5),
                        Pos2::new(meter_x + meter_width * 0.95, meter_y + meter_height * 0.5),
                    ],
                    Stroke::new(1.0 * canvas.zoom, Color32::from_rgb(200, 50, 50)),
                );

                let visible_base = Pos2::new(pivot.x, meter_bg.max.y);
                painter.line_segment(
                    [visible_base, bounded_tip],
                    Stroke::new(1.5 * canvas.zoom, Color32::from_rgb(180, 40, 40)),
                );

                painter.rect_stroke(
                    meter_bg,
                    2.0,
                    Stroke::new(1.0, Color32::from_white_alpha(40)),
                    egui::StrokeKind::Inside,
                );
            }
            crate::config::AudioMeterStyle::Digital => {
                let meter_bg = Rect::from_min_size(
                    Pos2::new(meter_x, meter_y),
                    Vec2::new(meter_width, meter_height),
                );
                painter.rect_filled(meter_bg, 2.0, Color32::from_gray(20));

                let num_segments = 20;
                let segment_spacing = 1.0 * canvas.zoom;
                let segment_width = (meter_width - (num_segments as f32 - 1.0) * segment_spacing)
                    / num_segments as f32;

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
                        Color32::from_rgb(0, 255, 100)
                    } else if t < 0.85 {
                        Color32::from_rgb(255, 180, 0)
                    } else {
                        Color32::from_rgb(255, 50, 50)
                    };

                    painter.rect_filled(seg_rect, 1.0, seg_color);
                }

                let threshold_x = meter_x + threshold * meter_width;
                painter.line_segment(
                    [
                        Pos2::new(threshold_x, meter_y - 2.0),
                        Pos2::new(threshold_x, meter_y + meter_height + 2.0),
                    ],
                    Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 50, 50, 200)),
                );
            }
        }
    }

    let socket_start_y = rect.min.y + title_height + 10.0 * canvas.zoom;
    for (i, socket) in part.inputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.min.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        let socket_color = utils::get_socket_color(&socket.socket_type);

        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        let type_name = socket.socket_type.name();
        let display_name = if socket
            .name
            .to_lowercase()
            .contains(&type_name.to_lowercase())
        {
            socket.name.clone()
        } else {
            format!("{} ({})", socket.name, type_name)
        };

        painter.text(
            Pos2::new(rect.min.x + 14.0 * canvas.zoom, socket_y),
            egui::Align2::LEFT_CENTER,
            &display_name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );
    }

    for (i, socket) in part.outputs.iter().enumerate() {
        let socket_y = socket_start_y + i as f32 * 22.0 * canvas.zoom;
        let socket_pos = Pos2::new(rect.max.x, socket_y);
        let socket_radius = 7.0 * canvas.zoom;

        let socket_color = utils::get_socket_color(&socket.socket_type);

        let is_hovered = if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            socket_pos.distance(pointer_pos) < socket_radius * 1.5
        } else {
            false
        };

        let ring_stroke = if is_hovered {
            let pulse = (ui.input(|i| i.time) * 10.0).sin() as f32 * 0.2 + 0.8;
            Stroke::new(3.0 * canvas.zoom, Color32::WHITE.linear_multiply(pulse))
        } else {
            Stroke::new(2.0 * canvas.zoom, socket_color)
        };
        painter.circle_stroke(socket_pos, socket_radius, ring_stroke);
        painter.circle_filled(
            socket_pos,
            socket_radius - 2.0 * canvas.zoom,
            Color32::from_gray(20),
        );
        painter.circle_filled(
            socket_pos,
            2.0 * canvas.zoom,
            if is_hovered {
                socket_color
            } else {
                Color32::from_gray(100)
            },
        );

        let type_name = socket.socket_type.name();
        let display_name = if socket
            .name
            .to_lowercase()
            .contains(&type_name.to_lowercase())
        {
            socket.name.clone()
        } else {
            format!("{} ({})", socket.name, type_name)
        };

        painter.text(
            Pos2::new(rect.max.x - 14.0 * canvas.zoom, socket_y),
            egui::Align2::RIGHT_CENTER,
            &display_name,
            egui::FontId::proportional(11.0 * canvas.zoom),
            Color32::from_gray(230),
        );
    }
}

pub fn get_delete_button_rect(canvas: &ModuleCanvas, part_rect: Rect) -> Rect {
    let title_height = 28.0 * canvas.zoom;
    Rect::from_center_size(
        Pos2::new(
            part_rect.max.x - 10.0 * canvas.zoom,
            part_rect.min.y + title_height * 0.5,
        ),
        Vec2::splat(20.0 * canvas.zoom),
    )
}
