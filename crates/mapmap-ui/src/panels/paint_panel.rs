// crates/mapmap-ui/src/paint_panel.rs

use crate::core::responsive::ResponsiveLayout;
use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
use egui::Context;
use mapmap_core::{PaintId, PaintManager, PaintType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintPanelAction {
    AddPaint,
    RemovePaint(PaintId),
}

#[derive(Default)]
pub struct PaintPanel {
    pub visible: bool,
    action: Option<PaintPanelAction>,
}

impl PaintPanel {
    pub fn take_action(&mut self) -> Option<PaintPanelAction> {
        self.action.take()
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        i18n: &LocaleManager,
        paint_manager: &mut PaintManager,
        icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(350.0, 450.0);

        egui::Window::new(i18n.t("panel-paints"))
            .open(&mut self.visible)
            .default_size(window_size)
            .resizable(true)
            .scroll([false, true])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(
                    ui,
                    &i18n.t("panel-paints"),
                    |_| {},
                );

                ui.add_space(8.0);

                ui.heading(i18n.t_args(
                    "label-total-paints",
                    &[("count", &paint_manager.paints().len().to_string())],
                ));
                ui.separator();

                let paint_ids: Vec<_> = paint_manager.paints().iter().map(|p| p.id).collect();

                for paint_id in paint_ids {
                    if let Some(paint) = paint_manager.get_paint_mut(paint_id) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(mgr) = icon_manager {
                                    let icon = match paint.paint_type {
                                        PaintType::Video => AppIcon::VideoFile,
                                        PaintType::Color => AppIcon::PaintBucket,
                                        _ => AppIcon::Pencil, // Fallback
                                    };
                                    mgr.show(ui, icon, 16.0);
                                }
                                ui.heading(&paint.name);
                            });

                            // Opacity slider
                            ui.add(
                                egui::Slider::new(&mut paint.opacity, 0.0..=1.0)
                                    .text(i18n.t("label-master-opacity")),
                            );

                            // Playback controls for video
                            if paint.paint_type == PaintType::Video {
                                ui.checkbox(&mut paint.is_playing, i18n.t("check-playing"));
                                ui.checkbox(&mut paint.loop_playback, i18n.t("mode-loop"));
                                ui.add(
                                    egui::Slider::new(&mut paint.rate, 0.1..=2.0)
                                        .text(i18n.t("label-speed")),
                                );
                            }

                            // Color picker for color type
                            if paint.paint_type == PaintType::Color {
                                ui.horizontal(|ui| {
                                    ui.label(i18n.t("paints-color"));
                                    ui.color_edit_button_rgba_unmultiplied(&mut paint.color);
                                });
                            }

                            if let Some(mgr) = icon_manager {
                                if let Some(img) = mgr.image(AppIcon::Remove, 16.0) {
                                    if ui
                                        .add(egui::Button::image(img))
                                        .clone()
                                        .on_hover_text(i18n.t("btn-remove"))
                                        .clicked()
                                    {
                                        self.action = Some(PaintPanelAction::RemovePaint(paint.id));
                                    }
                                }
                            } else if ui.button(i18n.t("btn-remove")).clicked() {
                                self.action = Some(PaintPanelAction::RemovePaint(paint.id));
                            }
                        });
                    }
                }

                ui.separator();

                if let Some(mgr) = icon_manager {
                    if let Some(img) = mgr.image(AppIcon::Add, 16.0) {
                        if ui
                            .add(egui::Button::image(img))
                            .clone()
                            .on_hover_text(i18n.t("btn-add-paint"))
                            .clicked()
                        {
                            self.action = Some(PaintPanelAction::AddPaint);
                        }
                    }
                } else if ui.button(i18n.t("btn-add-paint")).clicked() {
                    self.action = Some(PaintPanelAction::AddPaint);
                }
            });
    }
}
