//! UI and Node action processing.

use crate::app::core::app_struct::App;
use crate::orchestration::node_logic::load_project_file;
use anyhow::Result;
use mapmap_io::save_project;
use mapmap_mcp::McpAction;
use mapmap_ui::{NodeEditorAction, UIAction};
use rfd::FileDialog;
use std::path::PathBuf;
use tracing::{error, info};

/// Handle global UI actions
pub fn handle_ui_actions(app: &mut App) -> Result<bool> {
    let actions = app.ui_state.take_actions();
    let mut needs_sync = false;

    for action in actions {
        match action {
            UIAction::NodeAction(node_action) => {
                app.ui_state
                    .node_editor_panel
                    .handle_action(node_action.clone());
                if let Err(e) = handle_node_action(app, node_action) {
                    eprintln!("Error handling node action: {}", e);
                }
            }

            // Global Fullscreen Setting
            UIAction::SetGlobalFullscreen(is_fullscreen) => {
                needs_sync = true;
                // Update config
                app.ui_state.user_config.global_fullscreen = is_fullscreen;
                let _ = app.ui_state.user_config.save();
                info!("Global fullscreen set to: {}", is_fullscreen);
            }
            UIAction::OpenShaderGraph(graph_id) => {
                app.ui_state.show_shader_graph = true;
                if let Some(graph) = app.state.shader_graphs.get(&graph_id) {
                    app.ui_state.node_editor_panel.load_graph(graph);
                } else {
                    // Create if not exists
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        app.state.shader_graphs.entry(graph_id)
                    {
                        let new_graph = mapmap_core::shader_graph::ShaderGraph::new(
                            graph_id,
                            "New Graph".to_string(),
                        );
                        e.insert(new_graph.clone());
                        app.ui_state.node_editor_panel.load_graph(&new_graph);
                    }
                }
            }
            UIAction::ToggleModuleCanvas => {
                app.ui_state.show_module_canvas = !app.ui_state.show_module_canvas;
            }
            UIAction::ToggleFullscreen => {
                app.ui_state.user_config.window_maximized =
                    !app.ui_state.user_config.window_maximized;
                let _ = app.ui_state.user_config.save();
            }
            UIAction::ToggleControllerOverlay => {
                app.ui_state.show_controller_overlay = !app.ui_state.show_controller_overlay;
            }
            UIAction::ResetLayout => {
                app.ui_state.show_left_sidebar = true;
                app.ui_state.show_timeline = true;
                app.ui_state.show_inspector = true;
                app.ui_state.show_media_browser = true;
                app.ui_state.show_module_canvas = false;
            }
            UIAction::Play => app.state.effect_animator.play(),
            UIAction::Pause => app.state.effect_animator.pause(),
            UIAction::Stop => app.state.effect_animator.stop(),
            UIAction::SetSpeed(s) => app.state.effect_animator.set_speed(s),
            UIAction::ToggleMediaManager => {
                app.media_manager_ui.visible = !app.media_manager_ui.visible;
            }

            UIAction::SaveProjectAs => {
                if let Some(path) = FileDialog::new()
                    .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                    .set_file_name("project.mflow")
                    .save_file()
                {
                    if let Err(e) = save_project(&app.state, &path) {
                        error!("Failed to save project: {}", e);
                    } else {
                        info!("Project saved to {:?}", path);
                    }
                }
            }
            UIAction::SaveProject(path_str) => {
                let path = if path_str.is_empty() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                        .set_file_name("project.mflow")
                        .save_file()
                    {
                        path
                    } else {
                        PathBuf::new()
                    }
                } else {
                    PathBuf::from(path_str)
                };

                if !path.as_os_str().is_empty() {
                    if let Err(e) = save_project(&app.state, &path) {
                        error!("Failed to save project: {}", e);
                    } else {
                        info!("Project saved to {:?}", path);
                    }
                }
            }
            UIAction::PickMediaFile(module_id, part_id, path_str) => {
                app.ui_state.module_canvas.active_module_id = Some(module_id);
                app.ui_state.module_canvas.editing_part_id = Some(part_id);
                if !path_str.is_empty() {
                    let _ = app.action_sender.send(McpAction::SetModuleSourcePath(
                        module_id,
                        part_id,
                        std::path::PathBuf::from(path_str),
                    ));
                } else {
                    let sender = app.action_sender.clone();
                    app.tokio_runtime.spawn(async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .add_filter(
                                "Media",
                                &[
                                    "mp4", "mov", "avi", "mkv", "webm", "gif", "png", "jpg", "jpeg",
                                ],
                            )
                            .pick_file()
                            .await
                        {
                            let path = handle.path().to_path_buf();
                            let _ = sender
                                .send(McpAction::SetModuleSourcePath(module_id, part_id, path));
                        }
                    });
                }
            }
            UIAction::SetMediaFile(module_id, part_id, path) => {
                let _ = app.action_sender.send(McpAction::SetModuleSourcePath(
                    module_id,
                    part_id,
                    PathBuf::from(path),
                ));
            }

            UIAction::LoadProject(path_str) => {
                let path = if path_str.is_empty() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                        .pick_file()
                    {
                        path
                    } else {
                        PathBuf::new()
                    }
                } else {
                    PathBuf::from(path_str)
                };

                if !path.as_os_str().is_empty() {
                    load_project_file(app, &path);
                }
            }
            UIAction::LoadRecentProject(path_str) => {
                let path = PathBuf::from(path_str);
                load_project_file(app, &path);
            }
            UIAction::SetLanguage(lang_code) => {
                app.state.settings.language = lang_code.clone();
                app.state.dirty = true;
                app.ui_state.i18n.set_locale(&lang_code);
                info!("Language switched to: {}", lang_code);
            }
            UIAction::Exit => {
                info!("Exit requested via menu");
                app.exit_requested = true;
            }
            UIAction::OpenSettings => {
                info!("Settings requested");
                app.ui_state.show_settings = true;
            }
            #[cfg(feature = "ndi")]
            UIAction::ConnectNdiSource { part_id, source } => {
                let receiver = app.ndi_receivers.entry(part_id).or_insert_with(|| {
                    info!("Creating new NdiReceiver for part {}", part_id);
                    mapmap_io::ndi::NdiReceiver::new().expect("Failed to create NDI receiver")
                });
                info!(
                    "Connecting part {} to NDI source '{}'",
                    part_id, source.name
                );
                if let Err(e) = receiver.connect(&source) {
                    error!("Failed to connect to NDI source: {}", e);
                }
            }
            UIAction::SetMidiAssignment(element_id, target_id) => {
                #[cfg(feature = "midi")]
                {
                    use mapmap_ui::config::MidiAssignmentTarget;
                    app.ui_state.user_config.set_midi_assignment(
                        &element_id,
                        MidiAssignmentTarget::MapFlow(target_id.clone()),
                    );
                    tracing::info!(
                        "MIDI Assignment set via Global Learn: {} -> {}",
                        element_id,
                        target_id
                    );
                }
                #[cfg(not(feature = "midi"))]
                {
                    let _ = element_id;
                    let _ = target_id;
                }
            }
            UIAction::RegisterHue => {
                info!("Linking with Philips Hue Bridge...");
                let ip = app.ui_state.user_config.hue_config.bridge_ip.clone();
                if ip.is_empty() {
                    error!("Cannot link: No Bridge IP specified.");
                } else {
                    match app.tokio_runtime.block_on(app.hue_controller.register(&ip)) {
                        Ok(new_config) => {
                            info!("Successfully linked with Hue Bridge!");
                            app.ui_state.user_config.hue_config.username = new_config.username;
                            app.ui_state.user_config.hue_config.client_key = new_config.client_key;
                            let _ = app.ui_state.user_config.save();
                        }
                        Err(e) => {
                            error!("Failed to link with Hue Bridge: {}", e);
                        }
                    }
                }
            }
            UIAction::FetchHueGroups => {
                info!("Fetching Hue Entertainment Groups...");
                let bridge_ip = app.ui_state.user_config.hue_config.bridge_ip.clone();
                let username = app.ui_state.user_config.hue_config.username.clone();

                if bridge_ip.is_empty() || username.is_empty() {
                    error!("Cannot fetch groups: Bridge IP or Username missing");
                } else {
                    // Construct a temp config to fetch groups
                    let config = mapmap_control::hue::models::HueConfig {
                        bridge_ip: bridge_ip.clone(),
                        username: username.clone(),
                        ..Default::default()
                    };

                    info!("Calling get_entertainment_groups API...");
                    // Blocking call
                    match app.tokio_runtime.block_on(
                        mapmap_control::hue::api::groups::get_entertainment_groups(&config),
                    ) {
                        Ok(groups) => {
                            info!("Successfully fetched {} entertainment groups", groups.len());
                            for g in &groups {
                                info!("  - Group: id='{}', name='{}'", g.id, g.name);
                            }
                            app.ui_state.available_hue_groups =
                                groups.into_iter().map(|g| (g.id, g.name)).collect();
                        }
                        Err(e) => {
                            error!("Failed to fetch groups: {:?}", e);
                        }
                    }
                }
            }
            UIAction::ConnectHue => {
                info!("Connecting to Philips Hue Bridge...");
                let ui_hue = &app.ui_state.user_config.hue_config;
                let control_hue = mapmap_control::hue::models::HueConfig {
                    bridge_ip: ui_hue.bridge_ip.clone(),
                    username: ui_hue.username.clone(),
                    client_key: ui_hue.client_key.clone(),
                    application_id: String::new(),
                    entertainment_group_id: ui_hue.entertainment_area.clone(),
                };
                app.hue_controller.update_config(control_hue);

                if let Err(e) = app.tokio_runtime.block_on(app.hue_controller.connect()) {
                    error!("Failed to connect to Hue Bridge: {}", e);
                } else {
                    info!("Successfully connected to Hue Bridge");
                }
            }
            UIAction::DisconnectHue => {
                info!("Disconnecting from Philips Hue Bridge...");
                app.tokio_runtime.block_on(app.hue_controller.disconnect());
            }
            UIAction::DiscoverHueBridges => {
                info!("Discovering Philips Hue Bridges...");
                match app
                    .tokio_runtime
                    .block_on(mapmap_control::hue::api::discovery::discover_bridges())
                {
                    Ok(bridges) => {
                        info!("Discovered {} bridges", bridges.len());
                        app.ui_state.discovered_hue_bridges = bridges;
                    }
                    Err(e) => {
                        error!("Bridge discovery failed: {}", e);
                    }
                }
            }
            UIAction::SetLayerOpacity(id, opacity) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.opacity = opacity;
                    app.state.dirty = true;
                }
            }
            UIAction::SetLayerBlendMode(id, mode) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.blend_mode = mode;
                    app.state.dirty = true;
                }
            }
            UIAction::SetLayerVisibility(id, visible) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.visible = visible;
                    app.state.dirty = true;
                }
            }
            UIAction::AddLayer => {
                let count = app.state.layer_manager.len();
                app.state
                    .layer_manager
                    .create_layer(format!("Layer {}", count + 1));
                app.state.dirty = true;
            }
            UIAction::CreateGroup => {
                let count = app.state.layer_manager.len();
                app.state
                    .layer_manager
                    .create_group(format!("Group {}", count + 1));
                app.state.dirty = true;
            }
            UIAction::ReparentLayer(id, parent_id) => {
                app.state.layer_manager.reparent_layer(id, parent_id);
                app.state.dirty = true;
            }
            UIAction::SwapLayers(id1, id2) => {
                app.state.layer_manager.swap_layers(id1, id2);
                app.state.dirty = true;
            }
            UIAction::ToggleGroupCollapsed(id) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.collapsed = !layer.collapsed;
                    app.state.dirty = true;
                }
            }
            UIAction::RemoveLayer(id) => {
                app.state.layer_manager.remove_layer(id);
                app.state.dirty = true;
                if app.ui_state.selected_layer_id == Some(id) {
                    app.ui_state.selected_layer_id = None;
                }
            }
            UIAction::DuplicateLayer(id) => {
                if let Some(new_id) = app.state.layer_manager.duplicate_layer(id) {
                    app.ui_state.selected_layer_id = Some(new_id);
                    app.state.dirty = true;
                }
            }
            UIAction::RenameLayer(id, name) => {
                if app.state.layer_manager.rename_layer(id, name) {
                    app.state.dirty = true;
                }
            }
            UIAction::ToggleLayerSolo(id) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.toggle_solo();
                    app.state.dirty = true;
                }
            }
            UIAction::ToggleLayerBypass(id) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.toggle_bypass();
                    app.state.dirty = true;
                }
            }
            UIAction::EjectAllLayers => {
                app.state.layer_manager.eject_all();
                app.state.dirty = true;
            }
            UIAction::SetLayerTransform(id, transform) => {
                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.transform = transform;
                    app.state.dirty = true;
                }
            }
            UIAction::ApplyResizeMode(id, mode) => {
                let target_size = mapmap_core::Vec2::new(
                    app.state.layer_manager.composition.size.0 as f32,
                    app.state.layer_manager.composition.size.1 as f32,
                );

                let mut source_size = mapmap_core::Vec2::ONE;
                if let Some(layer) = app.state.layer_manager.get_layer(id) {
                    if let Some(paint_id) = layer.paint_id {
                        if let Some(_paint) = app.state.paint_manager.get_paint(paint_id) {
                            source_size = target_size;
                        }
                    }
                }

                if let Some(layer) = app.state.layer_manager.get_layer_mut(id) {
                    layer.set_transform_with_resize(mode, source_size, target_size);
                    app.state.dirty = true;
                }
            }
            UIAction::SetMasterOpacity(val) => {
                app.state.layer_manager.composition.set_master_opacity(val);
                app.state.dirty = true;
            }
            UIAction::SetMasterSpeed(val) => {
                app.state.layer_manager.composition.set_master_speed(val);
                app.state.dirty = true;
            }
            UIAction::SetCompositionName(name) => {
                app.state.layer_manager.composition.name = name;
                app.state.dirty = true;
            }
            UIAction::ConfigureOutput(id, config) => {
                let fs = config.fullscreen;
                app.state.output_manager.update_output(id, config.clone());

                let all_ids: Vec<_> = app
                    .state
                    .output_manager
                    .list_outputs()
                    .iter()
                    .map(|o| o.id)
                    .collect();
                for oid in all_ids {
                    if let Some(other) = app.state.output_manager.get_output_mut(oid) {
                        if other.fullscreen != fs {
                            other.fullscreen = fs;
                            info!("Syncing fullscreen state for output {} -> {}", oid, fs);
                        }
                    }
                }

                app.state.dirty = true;
            }
            UIAction::MediaCommand(part_id, command) => {
                app.ui_state
                    .module_canvas
                    .pending_playback_commands
                    .push((part_id, command));
            }
            _ => {
                // Other actions
            }
        }
    }

    // Handle MCP Actions
    handle_mcp_actions(app);
    // Also process implicit MCP actions that were sent via UI actions?
    // The loop above already handles UI -> MCP Sender calls, and handle_mcp_actions pulls from Receiver.
    // So this is correct.

    Ok(needs_sync)
}

/// Handle Node Editor actions
fn handle_node_action(app: &mut App, action: NodeEditorAction) -> Result<()> {
    if let Some(graph_id) = app.ui_state.node_editor_panel.graph_id {
        if let Some(graph) = app.state.shader_graphs.get_mut(&graph_id) {
            let mut needs_update = false;

            match action {
                NodeEditorAction::AddNode(node_type, pos) => {
                    let _id = graph.add_node(node_type);
                    if let Some(node) = graph.nodes.get_mut(&_id) {
                        node.position = (pos.x, pos.y);
                    }
                    needs_update = true;
                }
                NodeEditorAction::RemoveNode(node_id) => {
                    graph.remove_node(node_id);
                    needs_update = true;
                }
                NodeEditorAction::SelectNode(_) => {
                    // Selection is handled in UI state mostly.
                }
                NodeEditorAction::AddConnection(_from, from_socket, to, to_socket) => {
                    if let Err(e) = graph.connect(_from, &from_socket, to, &to_socket) {
                        tracing::warn!("Failed to connect nodes: {}", e);
                    } else {
                        needs_update = true;
                    }
                }
                NodeEditorAction::RemoveConnection(_from, _sub_idx, to, to_socket) => {
                    if let Err(e) = graph.disconnect(to, &to_socket) {
                        tracing::warn!("Failed to disconnect nodes: {}", e);
                    } else {
                        needs_update = true;
                    }
                }
                NodeEditorAction::UpdateGraph(_) => {
                    needs_update = true;
                }
            }

            if needs_update {
                app.ui_state.node_editor_panel.load_graph(graph);
                app.state.dirty = true;

                // Compile Graph
                if let Err(e) = app
                    .effect_chain_renderer
                    .update_shader_graph(&mut app.shader_graph_manager, graph_id)
                {
                    tracing::error!("Shader Compile Error: {}", e);
                } else {
                    tracing::info!("Shader Graph {} compiled successfully", graph_id);
                }
            }
        }
    }
    Ok(())
}

/// Process pending MCP actions
pub fn handle_mcp_actions(app: &mut App) {
    while let Ok(action) = app.mcp_receiver.try_recv() {
        if let mapmap_mcp::McpAction::SetModuleSourcePath(mod_id, part_id, path) = action {
            info!(
                "MCP: SetModuleSourcePath({}, {}, {:?})",
                mod_id, part_id, path
            );
            if let Some(module) = app.state.module_manager.get_module_mut(mod_id) {
                if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                    if let mapmap_core::module::ModulePartType::Source(
                        mapmap_core::module::SourceType::MediaFile {
                            path: ref mut current_path,
                            ..
                        },
                    ) = &mut part.part_type
                    {
                        let new_path_str = path.to_string_lossy().to_string();
                        if *current_path != new_path_str {
                            *current_path = new_path_str;
                            app.state.dirty = true;

                            // Force player reload by removing existing instance
                            // sync_media_players will recreate it with new path
                            if app.media_players.remove(&(mod_id, part_id)).is_some() {
                                info!("Removed player for {} to force reload", part_id);
                            }
                        }
                    }
                }
            }
        }
    }
}
