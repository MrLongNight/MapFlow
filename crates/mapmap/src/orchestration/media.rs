use crate::app::core::app_struct::App;
use anyhow::Result;
use crossbeam_channel::Sender;
use mapmap_render::TexturePool;
use std::sync::Arc;

/// Handle to a background media player.
pub struct MediaPlayerHandle {
    /// Command channel to control the underlying media player.
    pub command_tx: Sender<mapmap_media::PlaybackCommand>,
}

/// Creates a new media player handle.
pub fn create_player_handle(
    _pool: Arc<TexturePool>,
    _device: Arc<wgpu::Device>,
    _queue: Arc<wgpu::Queue>,
    _path: &str,
    _texture_name: &str,
) -> Result<MediaPlayerHandle> {
    // Placeholder until I find the correct run_player equivalent
    let (cmd_tx, _) = crossbeam_channel::unbounded();
    Ok(MediaPlayerHandle { command_tx: cmd_tx })
}

/// Synchronizes media players with the current module graph.
pub fn sync_media_players(_app: &mut App) {}

/// Updates all active media players.
pub fn update_media_players(_app: &mut App, _dt: f32) {}
