sed -i 's/use mapmap_ui as ui_crate;//' crates/mapmap/src/app/ui_layout.rs
sed -i 's/pub fn show/#[allow(missing_docs)]\npub fn show/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/\.show(ctx, |ui| {/.show(ctx, |_ui| {/' crates/mapmap/src/app/ui_layout.rs
