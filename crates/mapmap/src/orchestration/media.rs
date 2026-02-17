//! Video player and media orchestration.

use crate::app::core::app_struct::App;
use mapmap_core::module::{ModulePartType, SourceType};
use tracing::{error, info};

/// Synchronize media players with active source modules
pub fn sync_media_players(app: &mut App) {
    let mut active_sources = std::collections::HashSet::new();

    // Identify active media files across all source types
    for module in app.state.module_manager.modules() {
        for part in &module.parts {
            if let ModulePartType::Source(source_type) = &part.part_type {
                let path_result = match source_type {
                    SourceType::MediaFile { path, .. } => Some(path.clone()),
                    SourceType::VideoUni { path, .. } => Some(path.clone()),
                    SourceType::VideoMulti { shared_id, .. } => {
                        // Look up path in shared media
                        app.state
                            .module_manager
                            .shared_media
                            .get(shared_id)
                            .map(|item| item.path.clone())
                    }
                    _ => None,
                };

                if let Some(path) = path_result {
                    if !path.is_empty() {
                        let key = (module.id, part.id);
                        active_sources.insert(key);

                        // Create player if not exists
                        match app.media_players.entry(key) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                match mapmap_media::open_path(&path) {
                                    Ok(mut player) => {
                                        info!(
                                            "Created media player for module={} part={} path='{}'",
                                            module.id, part.id, path
                                        );
                                        if let Err(e) = player.play() {
                                            error!(
                                                "Failed to start playback for source {}:{} : {}",
                                                module.id, part.id, e
                                            );
                                        }
                                        e.insert((path.clone(), player));
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to create video player for source {}:{} : {}",
                                            module.id, part.id, e
                                        );
                                    }
                                }
                            }
                            std::collections::hash_map::Entry::Occupied(mut e) => {
                                // Check if path changed
                                let (current_path, player) = e.get_mut();
                                if *current_path != path {
                                    info!(
                                        "Path changed for source {}:{} -> loading {}",
                                        module.id, part.id, path
                                    );
                                    // Load new media
                                    match mapmap_media::open_path(&path) {
                                        Ok(mut new_player) => {
                                            if let Err(err) = new_player.play() {
                                                error!("Failed to start playback: {}", err);
                                            }
                                            *current_path = path.clone();
                                            *player = new_player;
                                        }
                                        Err(err) => {
                                            error!("Failed to load new media: {}", err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Cleanup removed players
    app.media_players
        .retain(|key, _| active_sources.contains(key));
}

/// Update all media players and upload frames to texture pool
#[allow(clippy::manual_is_multiple_of)]
pub fn update_media_players(app: &mut App, dt: f32) {
    static FRAME_LOG_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let num_frames = FRAME_LOG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    #[allow(clippy::manual_is_multiple_of)]
    let log_this_frame = num_frames % 60 == 0;

    let texture_pool = &mut app.texture_pool;
    let queue = &app.backend.queue;
    let ui_state = &mut app.ui_state;

    for ((mod_id, part_id), (_, player)) in &mut app.media_players {
        // Update player logic
        if let Some(frame) = player.update(std::time::Duration::from_secs_f32(dt)) {
            let tex_name = format!("part_{}_{}", mod_id, part_id);

            // Upload to GPU if data is on CPU
            if let mapmap_io::format::FrameData::Cpu(data) = &frame.data {
                if log_this_frame {
                    tracing::info!(
                        "Frame upload: mod={} part={} size={}x{}",
                        mod_id,
                        part_id,
                        frame.format.width,
                        frame.format.height
                    );
                }

                // CRITICAL: Ensure texture exists in pool with correct format and size
                texture_pool.ensure_texture(
                    &tex_name,
                    frame.format.width,
                    frame.format.height,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                );

                texture_pool.upload_data(
                    queue,
                    &tex_name,
                    data,
                    frame.format.width,
                    frame.format.height,
                );
            }
        }

        // Sync player info to UI for timeline display
        if let Some(active_id) = ui_state.module_canvas.active_module_id {
            if *mod_id == active_id {
                ui_state.module_canvas.player_info.insert(
                    *part_id,
                    mapmap_ui::types::MediaPlayerInfo {
                        current_time: player.current_time().as_secs_f64(),
                        duration: player.duration().as_secs_f64(),
                        is_playing: matches!(player.state(), mapmap_media::PlaybackState::Playing),
                    },
                );
            }
        }
    }
}





