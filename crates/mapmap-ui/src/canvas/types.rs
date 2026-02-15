/// Playback commands for media players
#[derive(Debug, Clone, PartialEq)]
pub enum MediaPlaybackCommand {
    Play,
    Pause,
    Stop,
    /// Reload the media from disk (used when path changes)
    Reload,
    /// Set playback speed (1.0 = normal)
    SetSpeed(f32),
    /// Set loop mode
    SetLoop(bool),
    /// Seek to position (seconds from start)
    Seek(f64),
    /// Set reverse playback
    SetReverse(bool),
}

/// Information about a media player's current state
#[derive(Debug, Clone, Default)]
pub struct MediaPlayerInfo {
    /// Current playback position in seconds
    pub current_time: f64,
    /// Total duration in seconds
    pub duration: f64,
    /// Whether the player is currently playing
    pub is_playing: bool,
}
