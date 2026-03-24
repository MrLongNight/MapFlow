#![allow(deprecated)]

use crate::app::core::app_struct::App;
use anyhow::Result;
use crossbeam_channel::Sender;
use vorce_core::module::{ModulePartType, SourceType};
use vorce_render::TexturePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Handle to a background media player.
pub struct MediaPlayerHandle {
    /// Resolved media source path used to create this player.
    pub source_path: String,
    /// Playback speed currently configured for the player.
    pub playback_speed: f32,
    /// Loop mode currently configured for the player.
    pub loop_enabled: bool,
    /// Command channel to control the player
    pub command_tx: Sender<vorce_media::PlaybackCommand>,
    /// Multi-threaded frame pipeline
    #[allow(deprecated)]
    pub pipeline: vorce_media::FramePipeline,
}

#[derive(Debug, Clone)]
struct DesiredMediaPlayer {
    module_id: u64,
    part_id: u64,
    path: String,
    playback_speed: f32,
    loop_enabled: bool,
}

const VIDEO_LOG_THROTTLE: Duration = Duration::from_secs(5);

fn should_log_video_issue(
    log_times: &mut HashMap<String, Instant>,
    key: impl Into<String>,
) -> bool {
    let key = key.into();
    let now = Instant::now();
    match log_times.get(&key) {
        Some(last_logged) if now.duration_since(*last_logged) < VIDEO_LOG_THROTTLE => false,
        _ => {
            log_times.insert(key, now);
            true
        }
    }
}

fn clear_video_issue(log_times: &mut HashMap<String, Instant>, key: impl AsRef<str>) {
    log_times.remove(key.as_ref());
}

fn desired_media_players(app: &App) -> Vec<DesiredMediaPlayer> {
    let shared_media = &app.state.module_manager.shared_media;

    app.state
        .module_manager
        .modules()
        .iter()
        .flat_map(|module| {
            module.parts.iter().filter_map(move |part| {
                let (path, playback_speed, loop_enabled) = match &part.part_type {
                    ModulePartType::Source(SourceType::MediaFile {
                        path,
                        speed,
                        loop_enabled,
                        ..
                    })
                    | ModulePartType::Source(SourceType::VideoUni {
                        path,
                        speed,
                        loop_enabled,
                        ..
                    }) => (path.clone(), *speed, *loop_enabled),
                    ModulePartType::Source(SourceType::ImageUni { path, .. }) => {
                        (path.clone(), 1.0, true)
                    }
                    ModulePartType::Source(SourceType::VideoMulti { shared_id, .. })
                    | ModulePartType::Source(SourceType::ImageMulti { shared_id, .. }) => (
                        shared_media
                            .get(shared_id)
                            .map(|item| item.path.clone())
                            .unwrap_or_default(),
                        1.0,
                        true,
                    ),
                    _ => return None,
                };

                let path = path.trim().to_string();
                if path.is_empty() {
                    return None;
                }

                Some(DesiredMediaPlayer {
                    module_id: module.id,
                    part_id: part.id,
                    path,
                    playback_speed,
                    loop_enabled,
                })
            })
        })
        .collect()
}

/// Creates a new media player handle.
#[allow(clippy::too_many_arguments)]
pub fn create_player_handle(
    pool: Arc<TexturePool>,
    _device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    path: &str,
    texture_name: &str,
    playback_speed: f32,
    loop_enabled: bool,
    start_playing: bool,
) -> Result<MediaPlayerHandle> {
    let path_buf = std::path::PathBuf::from(path);
    let name = texture_name.to_string();
    let mut player = vorce_media::open_path(&path_buf).map_err(anyhow::Error::from)?;
    player
        .set_speed(playback_speed)
        .map_err(anyhow::Error::from)?;
    player
        .set_loop_mode(if loop_enabled {
            vorce_media::LoopMode::Loop
        } else {
            vorce_media::LoopMode::PlayOnce
        })
        .map_err(anyhow::Error::from)?;

    // Prime the first frame immediately so a paused source still has a visible preview.
    player.play().map_err(anyhow::Error::from)?;
    if let Some(frame) = player.update(Duration::ZERO) {
        if let vorce_io::format::FrameData::Cpu(ref data) = frame.data {
            pool.upload_data(&queue, &name, data, frame.format.width, frame.format.height);
        } else {
            warn!(
                "Fehler in Videoausgabe: Erste Frame-Vorschau fuer '{}' konnte nicht hochgeladen werden, weil das Frame-Format nicht CPU-basiert ist.",
                path
            );
        }
    }
    if !start_playing {
        player.pause().map_err(anyhow::Error::from)?;
    }

    let cmd_tx = player.command_sender();

    // Create and start the pipeline
    #[allow(deprecated)]
    let mut pipeline = vorce_media::FramePipeline::new();

    // Start upload thread first to be ready for decoded frames
    let pool_clone = pool.clone();
    let queue_clone = queue.clone();
    let name_clone = name.clone();
    let path_display = path_buf.display().to_string();

    pipeline.start_upload_thread(move |pipeline_frame| {
        let frame = &pipeline_frame.frame;
        if let vorce_io::format::FrameData::Cpu(ref data) = frame.data {
            pool_clone.upload_data(
                &queue_clone,
                &name_clone,
                data,
                frame.format.width,
                frame.format.height,
            );
            Ok(())
        } else {
            Err(format!(
                "Fehler in Videoausgabe: Frame fuer '{}' konnte nicht hochgeladen werden, weil das Frame-Format nicht CPU-basiert ist.",
                path_display
            ).into())
        }
    });

    // Start decode thread with the player
    pipeline.start_decode_thread(player);

    Ok(MediaPlayerHandle {
        source_path: path.to_string(),
        playback_speed,
        loop_enabled,
        command_tx: cmd_tx,
        pipeline,
    })
}

/// Synchronizes media players with the current module graph.
pub fn sync_media_players(app: &mut App) {
    let desired_players = desired_media_players(app);

    // Bolt Optimization: Removing intermediate `HashSet` allocation (`desired_keys`)
    // and `.contains()` lookup, since we can just do a direct `.find()` over `desired_players`.
    let stale_keys: Vec<(u64, u64)> = app
        .media_players
        .iter()
        .filter_map(|(key, handle)| {
            let desired = match desired_players
                .iter()
                .find(|player| (player.module_id, player.part_id) == *key)
            {
                Some(d) => d,
                None => return Some(*key),
            };

            let same_path = handle.source_path == desired.path;
            let same_speed = (handle.playback_speed - desired.playback_speed).abs() < f32::EPSILON;
            let same_loop = handle.loop_enabled == desired.loop_enabled;

            if same_path && same_speed && same_loop {
                None
            } else {
                Some(*key)
            }
        })
        .collect();

    for (module_id, part_id) in stale_keys {
        let texture_name = format!("part_{}_{}", module_id, part_id);
        if app.media_players.remove(&(module_id, part_id)).is_some() {
            info!(
                "Media-Sync: entferne veralteten Player fuer Modul {} / Part {}",
                module_id, part_id
            );
        }
        app.texture_pool.release(&texture_name);
        clear_video_issue(
            &mut app.video_diagnostic_log_times,
            format!("video-output-missing-player:{module_id}:{part_id}"),
        );
        clear_video_issue(
            &mut app.video_diagnostic_log_times,
            format!("video-output-open-failed:{module_id}:{part_id}"),
        );
    }

    let start_playing = app.state.effect_animator.is_playing();

    for desired in desired_players {
        let key = (desired.module_id, desired.part_id);
        if app.media_players.contains_key(&key) {
            clear_video_issue(
                &mut app.video_diagnostic_log_times,
                format!(
                    "video-output-missing-player:{}:{}",
                    desired.module_id, desired.part_id
                ),
            );
            clear_video_issue(
                &mut app.video_diagnostic_log_times,
                format!(
                    "video-output-open-failed:{}:{}",
                    desired.module_id, desired.part_id
                ),
            );
            continue;
        }

        let source_path = std::path::PathBuf::from(&desired.path);
        if !source_path.exists() {
            let issue_key = format!(
                "video-output-missing-player:{}:{}",
                desired.module_id, desired.part_id
            );
            if should_log_video_issue(&mut app.video_diagnostic_log_times, issue_key) {
                warn!(
                    "Fehler in Videoausgabe: Modul {} / Part {} kann '{}' nicht laden, weil die Datei oder der Ordner nicht existiert.",
                    desired.module_id,
                    desired.part_id,
                    desired.path
                );
            }
            app.texture_pool
                .release(&format!("part_{}_{}", desired.module_id, desired.part_id));
            continue;
        }

        let texture_name = format!("part_{}_{}", desired.module_id, desired.part_id);
        match create_player_handle(
            app.texture_pool.clone(),
            app.backend.device.clone(),
            app.backend.queue.clone(),
            &desired.path,
            &texture_name,
            desired.playback_speed,
            desired.loop_enabled,
            start_playing,
        ) {
            Ok(handle) => {
                info!(
                    "Media-Sync: Player aktiv fuer Modul {} / Part {} -> '{}'",
                    desired.module_id, desired.part_id, desired.path
                );
                app.media_players.insert(key, handle);
                clear_video_issue(
                    &mut app.video_diagnostic_log_times,
                    format!(
                        "video-output-missing-player:{}:{}",
                        desired.module_id, desired.part_id
                    ),
                );
                clear_video_issue(
                    &mut app.video_diagnostic_log_times,
                    format!(
                        "video-output-open-failed:{}:{}",
                        desired.module_id, desired.part_id
                    ),
                );
            }
            Err(err) => {
                let issue_key = format!(
                    "video-output-open-failed:{}:{}",
                    desired.module_id, desired.part_id
                );
                if should_log_video_issue(&mut app.video_diagnostic_log_times, issue_key) {
                    error!(
                        "Fehler in Videoausgabe: Modul {} / Part {} konnte '{}' nicht initialisieren, weil {}.",
                        desired.module_id,
                        desired.part_id,
                        desired.path,
                        err
                    );
                }
                app.texture_pool.release(&texture_name);
            }
        }
    }
}

/// Updates all active media players.
#[allow(deprecated)]
pub fn update_media_players(app: &mut App, _dt: f32) {
    for handle in app.media_players.values_mut() {
        // Drain upload channel to keep the pipeline moving and update stats
        while let Ok(_frame) = handle.pipeline.upload_rx.try_recv() {
            // Frame is already uploaded to the texture pool by the upload thread.
            // We just need to drain the channel here.
        }
    }
}
