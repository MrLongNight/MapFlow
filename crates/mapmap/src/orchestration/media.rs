use crate::app::core::app_struct::App;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_render::TexturePool;
use std::sync::Arc;

/// Handle to a background media player.
pub struct MediaPlayerHandle {
    /// Command channel to control the player
    pub command_tx: Sender<mapmap_media::PlaybackCommand>,
    /// Update channel to send delta time
    pub update_tx: Sender<f32>,
}

/// Creates a new media player handle.
pub fn create_player_handle(
    pool: Arc<TexturePool>,
    _device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    path: &str,
    texture_name: &str,
) -> Result<MediaPlayerHandle> {
    let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
    let (upd_tx, upd_rx) = crossbeam_channel::unbounded();

    let path_buf = std::path::PathBuf::from(path);
    let name = texture_name.to_string();

    std::thread::spawn(move || {
        let mut player = match mapmap_media::open_path(&path_buf) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to open media {:?}: {}", path_buf, e);
                return;
            }
        };

        let mut last_update = std::time::Instant::now();
        loop {
            // Process commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                let _ = player.command_sender().send(cmd);
            }

            // Get dt from channel
            if let Ok(dt) = upd_rx.try_recv() {
                player.update(std::time::Duration::from_secs_f32(dt));

                if let Some(frame) = player.last_frame() {
                    // Extract byte slice from FrameData::Cpu
                    if let mapmap_io::format::FrameData::Cpu(ref data) = frame.data {
                        pool.upload_data(
                            &queue,
                            &name,
                            data,
                            frame.format.width,
                            frame.format.height,
                        );
                    }
                }
            }

            if last_update.elapsed().as_millis() < 5 {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            last_update = std::time::Instant::now();
        }
    });

    Ok(MediaPlayerHandle {
        command_tx: cmd_tx,
        update_tx: upd_tx,
    })
}

/// Synchronizes media players with the current module graph.
pub fn sync_media_players(_app: &mut App) {}

/// Updates all active media players.
pub fn update_media_players(app: &mut App, dt: f32) {
    for handle in app.media_players.values_mut() {
        let _ = handle.update_tx.send(dt);
    }
}
