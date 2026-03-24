use egui::{Color32, Id};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: Id,
    pub message: String,
    pub toast_type: ToastType,
    pub expires_at: Instant,
    pub duration: Duration,
}

#[derive(Default)]
pub struct ToastManager {
    pub toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn add(&mut self, message: String, toast_type: ToastType, duration: Duration) {
        let id = Id::new(message.clone()).with(Instant::now());
        self.toasts.push(Toast {
            id,
            message,
            toast_type,
            expires_at: Instant::now() + duration,
            duration,
        });
    }

    pub fn info(&mut self, message: String) {
        self.add(message, ToastType::Info, Duration::from_secs(3));
    }

    pub fn success(&mut self, message: String) {
        self.add(message, ToastType::Success, Duration::from_secs(3));
    }

    pub fn warning(&mut self, message: String) {
        self.add(message, ToastType::Warning, Duration::from_secs(5));
    }

    pub fn error(&mut self, message: String) {
        self.add(message, ToastType::Error, Duration::from_secs(8));
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|t| t.expires_at > now);
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.update();

        if self.toasts.is_empty() {
            return;
        }

        #[allow(deprecated)]
        let screen_rect = ctx.screen_rect();
        let mut current_y = screen_rect.max.y - 20.0;

        for toast in self.toasts.iter().rev() {
            let color = match toast.toast_type {
                ToastType::Info => Color32::from_rgb(100, 100, 255),
                ToastType::Success => Color32::from_rgb(100, 255, 100),
                ToastType::Warning => Color32::from_rgb(255, 200, 100),
                ToastType::Error => Color32::from_rgb(255, 100, 100),
            };

            let remaining = toast.expires_at.duration_since(Instant::now()).as_secs_f32();
            let progress = remaining / toast.duration.as_secs_f32();
            let alpha = if progress < 0.1 { progress * 10.0 } else { 1.0 };

            egui::Area::new(toast.id)
                .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -(screen_rect.max.y - current_y)])
                .show(ctx, |ui| {
                    egui::Frame::window(ui.style())
                        .fill(Color32::from_black_alpha((200.0 * alpha) as u8))
                        .stroke(egui::Stroke::new(1.0, color.linear_multiply(alpha)))
                        .inner_margin(egui::Margin::symmetric(16, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let icon = match toast.toast_type {
                                    ToastType::Info => "ℹ",
                                    ToastType::Success => "✔",
                                    ToastType::Warning => "⚠",
                                    ToastType::Error => "✖",
                                };
                                ui.label(egui::RichText::new(icon).color(color).strong());
                                ui.label(egui::RichText::new(&toast.message).color(Color32::WHITE.linear_multiply(alpha)));
                            });
                        });
                });

            current_y -= 50.0; // Estimate height
        }
    }
}
