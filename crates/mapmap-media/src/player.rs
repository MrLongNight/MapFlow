//! Video playback control

use crate::{DecodedFrame, VideoDecoder};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::time::Duration;
use thiserror::Error;

/// Player errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum PlayerError {
    #[error("Decoder error: {0}")]
    Decode(String),
    #[error("Seek error: {0}")]
    Seek(String),
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: String, to: String },
    #[error("Invalid command for current state {state:?}: {command:?}")]
    InvalidCommand { state: String, command: String },
}

/// Playback state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Error(PlayerError),
}

/// Loop mode for playback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    On,
    Off,
}

/// Commands to control video playback
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetSpeed(f32),
    SetLoopMode(LoopMode),
    SetPlaybackMode(PlaybackMode),
}

/// Playback direction (Phase 1, Month 5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackDirection {
    /// Play forward (default)
    #[default]
    Forward,
    /// Play backward (reverse)
    Backward,
}

/// Playback mode (Phase 1, Month 5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// Loop - repeat indefinitely (existing behavior)
    #[default]
    Loop,
    /// Ping Pong - bounce forward and backward
    PingPong,
    /// Play Once and Eject - stop and unload after completion
    PlayOnceAndEject,
    /// Play Once and Hold - stop on last frame
    PlayOnceAndHold,
}

/// Video player with playback control
pub struct VideoPlayer {
    decoder: Box<dyn VideoDecoder>,
    state: PlaybackState,
    current_time: Duration,
    playback_speed: f32,
    /// Legacy looping flag (deprecated - use playback_mode instead)
    looping: bool,
    /// Playback direction - forward or backward (Phase 1, Month 5)
    direction: PlaybackDirection,
    /// Playback mode - loop, ping pong, play once variants (Phase 1, Month 5)
    playback_mode: PlaybackMode,
    last_frame: Option<DecodedFrame>,
    /// Track whether we should eject after completing playback
    should_eject: bool,
    command_sender: Sender<PlaybackCommand>,
    command_receiver: Receiver<PlaybackCommand>,
}

impl VideoPlayer {
    /// Create a new video player with a decoder
    pub fn new(decoder: impl VideoDecoder + 'static) -> Self {
        let (command_sender, command_receiver) = unbounded();
        Self {
            decoder: Box::new(decoder),
            state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            playback_speed: 1.0,
            looping: false,
            direction: PlaybackDirection::default(),
            playback_mode: PlaybackMode::default(),
            last_frame: None,
            should_eject: false,
            command_sender,
            command_receiver,
        }
    }

    /// Get a sender to send commands to the player
    pub fn command_sender(&self) -> Sender<PlaybackCommand> {
        self.command_sender.clone()
    }

    /// Update the player (call every frame)
    pub fn update(&mut self, dt: Duration) -> Option<DecodedFrame> {
        self.process_commands();

        if self.state != PlaybackState::Playing {
            return self.last_frame.clone();
        }

        let duration = self.decoder.duration();

        // Advance playback time based on direction
        match self.direction {
            PlaybackDirection::Forward => {
                self.current_time += dt.mul_f32(self.playback_speed);

                // Check if we've reached the end
                if self.current_time >= duration {
                    if let Err(e) = self.handle_end_of_playback() {
                        self.state = PlaybackState::Error(e);
                    }
                    if self.should_eject {
                        return None; // Eject - return no frame
                    }
                }
            }
            PlaybackDirection::Backward => {
                // Go backward in time
                let delta = dt.mul_f32(self.playback_speed);
                if self.current_time > delta {
                    self.current_time -= delta;
                } else {
                    // Reached the beginning
                    self.current_time = Duration::ZERO;
                    if let Err(e) = self.handle_beginning_of_playback() {
                        self.state = PlaybackState::Error(e);
                    }
                    if self.should_eject {
                        return None; // Eject - return no frame
                    }
                }
            }
        }

        // For backward playback, we need to seek to current_time each frame
        // For forward playback, just get next frame sequentially
        if self.direction == PlaybackDirection::Backward {
            // Seek to current position for backward playback
            if self.decoder.seek(self.current_time).is_ok() {
                match self.decoder.next_frame() {
                    Ok(frame) => {
                        self.last_frame = Some(frame.clone());
                        return Some(frame);
                    }
                    Err(_) => {
                        return self.last_frame.clone();
                    }
                }
            } else {
                return self.last_frame.clone();
            }
        }

        // Forward playback - get next frame sequentially
        match self.decoder.next_frame() {
            Ok(frame) => {
                self.last_frame = Some(frame.clone());
                Some(frame)
            }
            Err(e) => {
                self.state = PlaybackState::Error(PlayerError::Decode(e.to_string()));
                self.last_frame.clone()
            }
        }
    }

    fn process_commands(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            let result = match command {
                PlaybackCommand::Play => self.play(),
                PlaybackCommand::Pause => self.pause(),
                PlaybackCommand::Stop => self.stop(),
                PlaybackCommand::Seek(duration) => self.seek(duration),
                PlaybackCommand::SetSpeed(speed) => self.set_speed(speed),
                PlaybackCommand::SetLoopMode(mode) => self.set_loop_mode(mode),
                PlaybackCommand::SetPlaybackMode(mode) => self.set_playback_mode(mode),
            };

            if let Err(e) = result {
                self.state = PlaybackState::Error(e);
            }
        }
    }

    fn transition_state(&mut self, new_state: PlaybackState) -> Result<(), PlayerError> {
        if self.state == new_state {
            return Ok(());
        }

        match (&self.state, &new_state) {
            (PlaybackState::Idle, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Stopped, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Loading, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Playing, PlaybackState::Paused) => Ok(()),
            (PlaybackState::Paused, PlaybackState::Playing) => Ok(()),
            (_, PlaybackState::Stopped) => Ok(()),
            (_, PlaybackState::Error(_)) => Ok(()),
            _ => Err(PlayerError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", new_state),
            }),
        }?;

        self.state = new_state;
        Ok(())
    }

    /// Handle reaching the end of playback (forward direction)
    fn handle_end_of_playback(&mut self) -> Result<(), PlayerError> {
        match self.playback_mode {
            PlaybackMode::Loop => {
                // Loop back to beginning
                self.seek(Duration::ZERO)?;
            }
            PlaybackMode::PingPong => {
                // Reverse direction
                self.direction = PlaybackDirection::Backward;
                self.current_time = self.decoder.duration();
            }
            PlaybackMode::PlayOnceAndEject => {
                // Stop and mark for ejection
                self.stop()?;
                self.should_eject = true;
            }
            PlaybackMode::PlayOnceAndHold => {
                // Stop and hold on last frame
                self.stop()?;
                self.current_time = self.decoder.duration();
            }
        }
        Ok(())
    }

    /// Handle reaching the beginning of playback (backward direction)
    fn handle_beginning_of_playback(&mut self) -> Result<(), PlayerError> {
        match self.playback_mode {
            PlaybackMode::Loop => {
                // Loop to end
                self.seek(self.decoder.duration())?;
            }
            PlaybackMode::PingPong => {
                // Reverse direction to forward
                self.direction = PlaybackDirection::Forward;
                self.current_time = Duration::ZERO;
            }
            PlaybackMode::PlayOnceAndEject => {
                // Stop and mark for ejection
                self.stop()?;
                self.should_eject = true;
            }
            PlaybackMode::PlayOnceAndHold => {
                // Stop and hold on first frame
                self.stop()?;
                self.current_time = Duration::ZERO;
            }
        }
        Ok(())
    }

    /// Start or resume playback
    pub fn play(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Playing)
    }

    /// Pause playback
    pub fn pause(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Paused)
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Stopped)?;
        self.seek(Duration::ZERO)
    }

    /// Seek to a specific timestamp
    pub fn seek(&mut self, timestamp: Duration) -> Result<(), PlayerError> {
        if self.decoder.seek(timestamp).is_ok() {
            self.current_time = timestamp;
            Ok(())
        } else {
            Err(PlayerError::Seek("Failed to seek".to_string()))
        }
    }

    /// Set playback speed (1.0 = normal, 0.5 = half speed, 2.0 = double speed)
    pub fn set_speed(&mut self, speed: f32) -> Result<(), PlayerError> {
        self.playback_speed = speed.clamp(0.0, 10.0);
        Ok(())
    }

    /// Enable or disable looping
    pub fn set_loop_mode(&mut self, mode: LoopMode) -> Result<(), PlayerError> {
        self.looping = mode == LoopMode::On;
        Ok(())
    }

    /// Get current playback state
    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    /// Get current playback time
    pub fn current_time(&self) -> Duration {
        self.current_time
    }

    /// Get total duration
    pub fn duration(&self) -> Duration {
        self.decoder.duration()
    }

    /// Get playback speed
    pub fn speed(&self) -> f32 {
        self.playback_speed
    }

    /// Check if looping is enabled
    pub fn is_looping(&self) -> bool {
        self.looping
    }

    /// Get video resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.decoder.resolution()
    }

    /// Get FPS
    pub fn fps(&self) -> f64 {
        self.decoder.fps()
    }

    /// Set playback direction (Phase 1, Month 5)
    pub fn set_direction(&mut self, direction: PlaybackDirection) {
        self.direction = direction;
    }

    /// Get current playback direction
    pub fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    /// Set playback mode (Phase 1, Month 5)
    pub fn set_playback_mode(&mut self, mode: PlaybackMode) -> Result<(), PlayerError> {
        self.playback_mode = mode;
        self.should_eject = false; // Reset eject flag when changing mode
        Ok(())
    }

    /// Get current playback mode
    pub fn playback_mode(&self) -> PlaybackMode {
        self.playback_mode
    }

    /// Check if the player should eject (for PlayOnceAndEject mode)
    pub fn should_eject(&self) -> bool {
        self.should_eject
    }

    /// Toggle direction between forward and backward
    pub fn toggle_direction(&mut self) {
        self.direction = match self.direction {
            PlaybackDirection::Forward => PlaybackDirection::Backward,
            PlaybackDirection::Backward => PlaybackDirection::Forward,
        };
    }

    /// Reset the eject flag (call after ejecting content)
    pub fn reset_eject(&mut self) {
        self.should_eject = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::TestPatternDecoder;
    use crate::{MediaError, PixelFormat};

    // A mock decoder that can be configured to fail.
    struct MockDecoder {
        fail_seek: bool,
        fail_next_frame: bool,
    }

    impl MockDecoder {
        fn new() -> Self {
            Self {
                fail_seek: false,
                fail_next_frame: false,
            }
        }
    }

    impl VideoDecoder for MockDecoder {
        fn duration(&self) -> Duration {
            Duration::from_secs(60)
        }

        fn resolution(&self) -> (u32, u32) {
            (1920, 1080)
        }

        fn fps(&self) -> f64 {
            30.0
        }

        fn seek(&mut self, _timestamp: Duration) -> Result<(), MediaError> {
            if self.fail_seek {
                Err(MediaError::SeekError("Seek failed".to_string()))
            } else {
                Ok(())
            }
        }

        fn next_frame(&mut self) -> Result<DecodedFrame, MediaError> {
            if self.fail_next_frame {
                Err(MediaError::DecoderError("Decode failed".to_string()))
            } else {
                Ok(DecodedFrame {
                    pts: Duration::ZERO,
                    data: vec![],
                    width: 0,
                    height: 0,
                    format: PixelFormat::RGBA8,
                })
            }
        }
    }

    #[test]
    fn test_player_creation() {
        // Use TestPatternDecoder for testing
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let player = VideoPlayer::new(decoder);

        assert_eq!(*player.state(), PlaybackState::Idle);
        assert_eq!(player.speed(), 1.0);
        assert!(!player.is_looping());
        assert_eq!(player.direction(), PlaybackDirection::Forward);
        assert_eq!(player.playback_mode(), PlaybackMode::Loop);
    }

    #[test]
    fn test_player_playback_control() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert!(player.play().is_ok());
        assert_eq!(*player.state(), PlaybackState::Playing);

        assert!(player.pause().is_ok());
        assert_eq!(*player.state(), PlaybackState::Paused);

        assert!(player.stop().is_ok());
        assert_eq!(*player.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_player_speed_control() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert!(player.set_speed(2.0).is_ok());
        assert_eq!(player.speed(), 2.0);

        assert!(player.set_speed(0.5).is_ok());
        assert_eq!(player.speed(), 0.5);

        // Test clamping
        assert!(player.set_speed(20.0).is_ok());
        assert_eq!(player.speed(), 10.0);

        assert!(player.set_speed(-1.0).is_ok());
        assert_eq!(player.speed(), 0.0);
    }

    #[test]
    fn test_playback_direction() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert_eq!(player.direction(), PlaybackDirection::Forward);

        player.set_direction(PlaybackDirection::Backward);
        assert_eq!(player.direction(), PlaybackDirection::Backward);

        player.toggle_direction();
        assert_eq!(player.direction(), PlaybackDirection::Forward);
    }

    #[test]
    fn test_playback_modes() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert_eq!(player.playback_mode(), PlaybackMode::Loop);

        assert!(player.set_playback_mode(PlaybackMode::PingPong).is_ok());
        assert_eq!(player.playback_mode(), PlaybackMode::PingPong);

        assert!(player
            .set_playback_mode(PlaybackMode::PlayOnceAndEject)
            .is_ok());
        assert_eq!(player.playback_mode(), PlaybackMode::PlayOnceAndEject);
        assert!(!player.should_eject());

        assert!(player
            .set_playback_mode(PlaybackMode::PlayOnceAndHold)
            .is_ok());
        assert_eq!(player.playback_mode(), PlaybackMode::PlayOnceAndHold);
    }

    #[test]
    fn test_state_transitions() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        // Idle -> Playing
        assert!(player.transition_state(PlaybackState::Playing).is_ok());
        assert_eq!(*player.state(), PlaybackState::Playing);

        // Playing -> Paused
        assert!(player.transition_state(PlaybackState::Paused).is_ok());
        assert_eq!(*player.state(), PlaybackState::Paused);

        // Paused -> Playing
        assert!(player.transition_state(PlaybackState::Playing).is_ok());
        assert_eq!(*player.state(), PlaybackState::Playing);

        // Playing -> Stopped
        assert!(player.transition_state(PlaybackState::Stopped).is_ok());
        assert_eq!(*player.state(), PlaybackState::Stopped);

        // Paused -> Stopped
        player.state = PlaybackState::Paused;
        assert!(player.transition_state(PlaybackState::Stopped).is_ok());
        assert_eq!(*player.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        // Idle -> Paused
        assert!(player.transition_state(PlaybackState::Paused).is_err());

        // Loading -> Paused
        player.state = PlaybackState::Loading;
        assert!(player.transition_state(PlaybackState::Paused).is_err());

        // Stopped -> Loading
        player.state = PlaybackState::Stopped;
        assert!(player.transition_state(PlaybackState::Loading).is_err());
    }

    #[test]
    fn test_command_processing() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);
        let command_sender = player.command_sender();

        // Send a Play command
        command_sender.send(PlaybackCommand::Play).unwrap();
        player.process_commands();
        assert_eq!(*player.state(), PlaybackState::Playing);

        // Send a Pause command
        command_sender.send(PlaybackCommand::Pause).unwrap();
        player.process_commands();
        assert_eq!(*player.state(), PlaybackState::Paused);

        // Send a Stop command
        command_sender.send(PlaybackCommand::Stop).unwrap();
        player.process_commands();
        assert_eq!(*player.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_seek_error() {
        let mut decoder = MockDecoder::new();
        decoder.fail_seek = true;
        let mut player = VideoPlayer::new(decoder);

        assert!(player.seek(Duration::from_secs(10)).is_err());
    }

    #[test]
    fn test_decode_error() {
        let mut decoder = MockDecoder::new();
        decoder.fail_next_frame = true;
        let mut player = VideoPlayer::new(decoder);

        player.play().unwrap();
        player.update(Duration::from_millis(33));

        assert!(matches!(player.state(), PlaybackState::Error(_)));
    }
}
