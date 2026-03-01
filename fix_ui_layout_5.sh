sed -i 's/\.show(ctx, |_ui| {/.show(ctx, |ui| {/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/use crate::ui;//' crates/mapmap/src/app/ui_layout.rs
