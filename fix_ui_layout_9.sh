sed -i 's/\.show(ctx, |ui| {/.show(ctx, |_ui| {/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/_ui\.heading("Timeline (Work in Progress)");/ui.heading("Timeline (Work in Progress)");/' crates/mapmap/src/app/ui_layout.rs
