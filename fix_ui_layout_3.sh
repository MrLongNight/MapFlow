sed -i 's/ui::panels::dashboard::show(ctx, app);/\/\/ ui::panels::dashboard::show(ctx, app);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::view::media_browser::show(ctx, app);/\/\/ ui::view::media_browser::show(ctx, app);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::transform_panel::show(ui, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::transform_panel::show(ui, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::edge_blend_panel::show(ui, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::edge_blend_panel::show(ui, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::effect_chain_panel::show(ui, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::effect_chain_panel::show(ui, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::audio_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::audio_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::controller_overlay_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::controller_overlay_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::assignment_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::assignment_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
sed -i 's/ui::panels::shortcuts_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/\/\/ ui::panels::shortcuts_panel::show(ctx, \&mut app.ui_state, \&mut app.state);/' crates/mapmap/src/app/ui_layout.rs
