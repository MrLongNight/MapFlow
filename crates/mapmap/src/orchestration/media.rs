//! Video player and media orchestration.

use crate::app::core::app_struct::App;
use crossbeam_channel::{Receiver, Sender};
use mapmap_core::module::{ModulePartType, SourceType};
use mapmap_media::{FramePipeline, PlaybackCommand, PlaybackState, PlaybackStatus};
use mapmap_render::WgpuFrameUploader;
use std::time::Duration;
use tracing::{error, info};

/// Handle for a media player running in a separate thread
pub struct MediaPlayerHandle {
    /// Path to the media file
    pub path: String,
    /// The frame pipeline handling decoding and uploading
    pub pipeline: FramePipeline,
    /// Channel for sending playback commands
    pub command_tx: Sender<PlaybackCommand>,
    /// Channel for receiving status updates
    pub status_rx: Receiver<PlaybackStatus>,
    /// Current playback time (cached from frames)
    pub current_time: Duration,
    /// Total duration of the media
    pub duration: Duration,
    /// Current playback state
    pub state: PlaybackState,
}

/// Create a new media player handle, spawning decode and upload threads
pub fn create_player_handle(
    pool: std::sync::Arc<mapmap_render::TexturePool>,
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    path: &str,
    tex_name: &str,
) -> anyhow::Result<MediaPlayerHandle> {
    let mut player = mapmap_media::open_path(path)?;
    let duration = player.duration();
    let (width, height) = player.resolution();

    // Start playing immediately
    player.play().map_err(|e| anyhow::anyhow!("{}", e))?;

    let command_tx = player.command_sender();
    let status_rx = player.status_receiver();
    let state = player.state().clone();

    let mut pipeline = FramePipeline::new();

    // Start decode thread (consumes player)
    pipeline.start_decode_thread(player);

    let tex_name_owned = tex_name.to_string();
    let pool_clone = pool.clone();
    let uploader = std::sync::Arc::new(WgpuFrameUploader::new(device.clone(), queue.clone()));

    // Ensure texture exists initially
    pool.ensure_texture(
        &tex_name_owned,
        if width > 0 { width } else { 1280 },
        if height > 0 { height } else { 720 },
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );

    pipeline.start_upload_thread(move |frame: &mapmap_media::pipeline::PipelineFrame| {
        if let mapmap_io::format::FrameData::Cpu(data) = &frame.frame.data {
            let width = frame.frame.format.width;
            let height = frame.frame.format.height;

            // Ensure texture exists and is correct size
            pool_clone.ensure_texture(
                &tex_name_owned,
                width,
                height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );

            // Get texture and upload
            if let Some(texture) = pool_clone.get_texture(&tex_name_owned) {
                uploader.upload(&texture, data, width, height);
            }
        }
        Ok(())
    });

    Ok(MediaPlayerHandle {
        path: path.to_string(),
        pipeline,
        command_tx,
        status_rx,
        current_time: Duration::ZERO,
        duration,
        state,
    })
}

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
                    SourceType::ImageUni { path, .. } => Some(path.clone()),
                    SourceType::VideoMulti { shared_id, .. }
                    | SourceType::ImageMulti { shared_id, .. } => {
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
                        let tex_name = format!("part_{}_{}", module.id, part.id);

                        let pool = app.texture_pool.clone();
                        let device = app.backend.device.clone();
                        let queue = app.backend.queue.clone();

                        // Create player if not exists
                        match app.media_players.entry(key) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                match create_player_handle(
                                    pool.clone(),
                                    device.clone(),
                                    queue.clone(),
                                    &path,
                                    &tex_name,
                                ) {
                                    Ok(handle) => {
                                        info!(
                                            "Created media player for module={} part={} path='{}'",
                                            module.id, part.id, path
                                        );
                                        e.insert(handle);
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
                                let handle = e.get_mut();
                                if handle.path != path {
                                    info!(
                                        "Path changed for source {}:{} -> loading {}",
                                        module.id, part.id, path
                                    );
                                    // Load new media
                                    match create_player_handle(
                                        pool,
                                        device.clone(),
                                        queue,
                                        &path,
                                        &tex_name,
                                    ) {
                                        Ok(new_handle) => {
                                            *handle = new_handle;
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
pub fn update_media_players(app: &mut App, _dt: f32) {
    let ui_state = &mut app.ui_state;

    for ((mod_id, part_id), handle) in &mut app.media_players {
        // 1. Process Status Updates
        while let Ok(status) = handle.status_rx.try_recv() {
            match status {
                PlaybackStatus::StateChanged(new_state) => {
                    handle.state = new_state;
                }
                PlaybackStatus::Looped => {
                    handle.current_time = Duration::ZERO;
                }
                PlaybackStatus::ReachedEnd => {
                    handle.current_time = handle.duration;
                    handle.state = PlaybackState::Stopped;
                }
                PlaybackStatus::Error(e) => {
                    error!("Player error {}:{}: {}", mod_id, part_id, e);
                    handle.state = PlaybackState::Error(e);
                }
            }
        }

        // 2. Process Uploaded Frames (Update timestamp)
        // Drain the upload queue to get the latest frame info.
        // Frames are already uploaded to GPU by the upload thread.
        // We just need the timestamp for UI.
        while let Ok(frame) = handle.pipeline.upload_rx.try_recv() {
            handle.current_time = frame.frame.timestamp;
        }

        // 3. Sync player info to UI
        if let Some(active_id) = ui_state.module_canvas.active_module_id {
            if *mod_id == active_id {
                ui_state.module_canvas.player_info.insert(
                    *part_id,
                    mapmap_ui::types::MediaPlayerInfo {
                        current_time: handle.current_time.as_secs_f64(),
                        duration: handle.duration.as_secs_f64(),
                        is_playing: matches!(handle.state, PlaybackState::Playing),
                    },
                );

                // Update node preview texture for the active module
                let tex_name = format!("part_{}_{}", mod_id, part_id);
                if let Some(texture) = app.texture_pool.get_texture(&tex_name) {
                    let view = std::sync::Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));

                    // Register with egui if not already cached
                    let tex_id = if let Some((cached_id, _cached_view)) = app.preview_texture_cache.get(&(*mod_id, *part_id)) {
                        // Check if view changed? (e.g. resize)
                        *cached_id
                    } else {
                        let id = app.egui_renderer.register_native_texture(
                            &app.backend.device,
                            &view,
                            wgpu::FilterMode::Linear,
                        );
                        app.preview_texture_cache.insert((*mod_id, *part_id), (id, view.clone()));
                        id
                    };

                    ui_state.module_canvas.node_previews.insert((*mod_id, *part_id), tex_id);
                }
            }
        }
    }
}
