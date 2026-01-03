//! Robust Media Playback State Machine
//!
//! This module implements a fault-tolerant state machine for video/audio playback.
//! It replaces legacy implementations with a clean, command-driven architecture.

use crate::VideoDecoder;
use crossbeam_channel::{unbounded, Receiver, Sender};
use mapmap_io::VideoFrame;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, info, warn};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Loop, // Repeat indefinitely
    PlayOnce, // Stop at end
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
}

/// Status notifications from the player
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackStatus {
    StateChanged(PlaybackState),
    Error(PlayerError),
    ReachedEnd,
    Looped,
}

/// Video player with robust state machine
pub struct VideoPlayer {
    decoder: Box<dyn VideoDecoder>,
    state: PlaybackState,
    current_time: Duration,
    playback_speed: f32,
    loop_mode: LoopMode,
    last_frame: Option<VideoFrame>,

    // Command and Status channels
    command_sender: Sender<PlaybackCommand>,
    command_receiver: Receiver<PlaybackCommand>,
    status_sender: Sender<PlaybackStatus>,
    status_receiver: Receiver<PlaybackStatus>,
}

impl VideoPlayer {
    /// Create a new video player with a decoder
    pub fn new(decoder: impl VideoDecoder + 'static) -> Self {
        Self::new_with_box(Box::new(decoder))
    }

    /// Create a new video player with a boxed decoder trait object
    pub fn new_with_box(decoder: Box<dyn VideoDecoder>) -> Self {
        let (command_sender, command_receiver) = unbounded();
        let (status_sender, status_receiver) = unbounded();

        Self {
            decoder,
            state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            playback_speed: 1.0,
            loop_mode: LoopMode::default(),
            last_frame: None,
            command_sender,
            command_receiver,
            status_sender,
            status_receiver,
        }
    }

    /// Get a sender to send commands to the player
    pub fn command_sender(&self) -> Sender<PlaybackCommand> {
        self.command_sender.clone()
    }

    /// Get a receiver for status updates
    pub fn status_receiver(&self) -> Receiver<PlaybackStatus> {
        self.status_receiver.clone()
    }

    /// Update the player (call every frame)
    pub fn update(&mut self, dt: Duration) -> Option<VideoFrame> {
        self.process_commands();

        // If not playing, return last frame or None
        if self.state != PlaybackState::Playing {
            return self.last_frame.clone();
        }

        let duration = self.decoder.duration();

        // Advance time
        self.current_time += dt.mul_f32(self.playback_speed);

        // Check for end of stream
        // Only check if duration is known (non-zero)
        if duration > Duration::ZERO && self.current_time >= duration {
            match self.loop_mode {
                LoopMode::Loop => {
                    self.current_time = Duration::ZERO;
                    if let Err(e) = self.seek_internal(Duration::ZERO) {
                        self.transition_to_error(e);
                        return self.last_frame.clone();
                    }
                    let _ = self.status_sender.send(PlaybackStatus::Looped);
                }
                LoopMode::PlayOnce => {
                    self.current_time = duration; // Clamp to end
                    let _ = self.transition_state(PlaybackState::Stopped); // Auto-stop
                    let _ = self.status_sender.send(PlaybackStatus::ReachedEnd);
                    return self.last_frame.clone();
                }
            }
        }

        match self.decoder.next_frame() {
            Ok(frame) => {
                self.last_frame = Some(frame.clone());
                Some(frame)
            }
            Err(e) => {
                if matches!(e, crate::MediaError::EndOfStream) {
                    // Handle EOF loop logic
                    match self.loop_mode {
                        LoopMode::Loop => {
                            self.current_time = Duration::ZERO;
                            if let Err(seek_err) = self.seek_internal(Duration::ZERO) {
                                self.transition_to_error(seek_err);
                            } else {
                                let _ = self.status_sender.send(PlaybackStatus::Looped);
                                // Try to get the first frame again immediately so we don't drop a frame
                                if let Ok(frame) = self.decoder.next_frame() {
                                    self.last_frame = Some(frame.clone());
                                    return Some(frame);
                                }
                            }
                        }
                        LoopMode::PlayOnce => {
                            self.current_time = duration;
                            let _ = self.transition_state(PlaybackState::Stopped);
                            let _ = self.status_sender.send(PlaybackStatus::ReachedEnd);
                        }
                    }
                } else {
                    warn!("Decoder error during playback: {}", e);
                    // Only transition to error if it's a critical decode error,
                    // otherwise keep trying next frame
                }

                // Return last frame to avoid visual glitches
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
                PlaybackCommand::Seek(time) => self.seek(time),
                PlaybackCommand::SetSpeed(speed) => self.set_speed(speed),
                PlaybackCommand::SetLoopMode(mode) => self.set_loop_mode(mode),
            };

            if let Err(e) = result {
                warn!("Command execution failed: {}", e);
                // Send error status but don't necessarily change state to Error unless critical
                let _ = self.status_sender.send(PlaybackStatus::Error(e));
            }
        }
    }

    /// Internal transition helper
    fn transition_state(&mut self, new_state: PlaybackState) -> Result<(), PlayerError> {
        if self.state == new_state {
            return Ok(());
        }

        // Validate transition
        match (&self.state, &new_state) {
            (PlaybackState::Idle, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Idle, PlaybackState::Loading) => Ok(()), // E.g. loading a file
            (PlaybackState::Stopped, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Stopped, PlaybackState::Idle) => Ok(()), // Reset
            (PlaybackState::Loading, PlaybackState::Playing) => Ok(()),
            (PlaybackState::Loading, PlaybackState::Stopped) => Ok(()),
            (PlaybackState::Playing, PlaybackState::Paused) => Ok(()),
            (PlaybackState::Paused, PlaybackState::Playing) => Ok(()),
            (_, PlaybackState::Stopped) => Ok(()), // Stop is always valid
            (_, PlaybackState::Error(_)) => Ok(()), // Error can happen anytime
            (PlaybackState::Error(_), PlaybackState::Idle) => Ok(()), // Reset from error
            _ => Err(PlayerError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", new_state),
            }),
        }?;

        info!("State transition: {:?} -> {:?}", self.state, new_state);
        self.state = new_state.clone();
        let _ = self
            .status_sender
            .send(PlaybackStatus::StateChanged(new_state));
        Ok(())
    }

    fn transition_to_error(&mut self, error: PlayerError) {
        error!("Player error: {}", error);
        self.state = PlaybackState::Error(error.clone());
        let _ = self
            .status_sender
            .send(PlaybackStatus::StateChanged(self.state.clone()));
    }

    // --- Command Implementations ---

    pub fn play(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Playing)
    }

    pub fn pause(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Paused)
    }

    pub fn stop(&mut self) -> Result<(), PlayerError> {
        self.transition_state(PlaybackState::Stopped)?;
        self.seek_internal(Duration::ZERO)
    }

    pub fn seek(&mut self, time: Duration) -> Result<(), PlayerError> {
        self.seek_internal(time)
    }

    fn seek_internal(&mut self, time: Duration) -> Result<(), PlayerError> {
        let duration = self.decoder.duration();
        let target_time = if time > duration { duration } else { time };

        match self.decoder.seek(target_time) {
            Ok(_) => {
                self.current_time = target_time;
                Ok(())
            }
            Err(e) => Err(PlayerError::Seek(e.to_string())),
        }
    }

    pub fn set_speed(&mut self, speed: f32) -> Result<(), PlayerError> {
        self.playback_speed = speed.max(0.0); // No negative speed for now
        Ok(())
    }

    pub fn set_loop_mode(&mut self, mode: LoopMode) -> Result<(), PlayerError> {
        self.loop_mode = mode;
        Ok(())
    }

    // --- Accessors ---

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn current_time(&self) -> Duration {
        self.current_time
    }

    pub fn duration(&self) -> Duration {
        self.decoder.duration()
    }

    pub fn speed(&self) -> f32 {
        self.playback_speed
    }

    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    pub fn resolution(&self) -> (u32, u32) {
        self.decoder.resolution()
    }

    pub fn fps(&self) -> f64 {
        self.decoder.fps()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::TestPatternDecoder;
    use crate::MediaError;

    // A mock decoder that can be configured to fail.
    #[derive(Clone)]
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

        fn next_frame(&mut self) -> Result<VideoFrame, MediaError> {
            if self.fail_next_frame {
                Err(MediaError::DecoderError("Decode failed".to_string()))
            } else {
                Ok(VideoFrame::empty(mapmap_io::VideoFormat {
                    width: 1920,
                    height: 1080,
                    pixel_format: mapmap_io::PixelFormat::RGBA8,
                    frame_rate: 30.0,
                }))
            }
        }

        fn clone_decoder(&self) -> crate::Result<Box<dyn VideoDecoder>> {
            Ok(Box::new(self.clone()))
        }
    }

    #[test]
    fn test_player_creation() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let player = VideoPlayer::new(decoder);

        assert_eq!(*player.state(), PlaybackState::Idle);
        assert_eq!(player.speed(), 1.0);
        assert_eq!(player.loop_mode(), LoopMode::Loop);
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
        assert_eq!(player.current_time(), Duration::ZERO);
    }

    #[test]
    fn test_player_speed_control() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert!(player.set_speed(2.0).is_ok());
        assert_eq!(player.speed(), 2.0);

        // Test clamping (negative becomes 0)
        assert!(player.set_speed(-1.0).is_ok());
        assert_eq!(player.speed(), 0.0);
    }

    #[test]
    fn test_loop_mode() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        assert_eq!(player.loop_mode(), LoopMode::Loop);

        assert!(player.set_loop_mode(LoopMode::PlayOnce).is_ok());
        assert_eq!(player.loop_mode(), LoopMode::PlayOnce);
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
    }

    #[test]
    fn test_invalid_state_transitions() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);

        // Idle -> Paused (Invalid)
        assert!(player.transition_state(PlaybackState::Paused).is_err());
    }

    #[test]
    fn test_command_channel() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);
        let tx = player.command_sender();

        tx.send(PlaybackCommand::Play).unwrap();
        player.process_commands();
        assert_eq!(*player.state(), PlaybackState::Playing);

        tx.send(PlaybackCommand::Pause).unwrap();
        player.process_commands();
        assert_eq!(*player.state(), PlaybackState::Paused);
    }

    #[test]
    fn test_status_channel() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(60), 30.0);
        let mut player = VideoPlayer::new(decoder);
        let rx = player.status_receiver();

        player.play().unwrap();

        let status = rx.try_recv();
        assert!(status.is_ok());
        match status.unwrap() {
            PlaybackStatus::StateChanged(s) => assert_eq!(s, PlaybackState::Playing),
            _ => panic!("Expected StateChanged"),
        }
    }

    #[test]
    fn test_playback_loop() {
        // Duration 1 sec, speed 1.0
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(1), 30.0);
        let mut player = VideoPlayer::new(decoder);
        let rx = player.status_receiver();

        player.set_loop_mode(LoopMode::Loop).unwrap();
        player.play().unwrap();

        // Drain status
        while rx.try_recv().is_ok() {}

        // Advance 1.1s
        player.update(Duration::from_millis(1100));

        // Should have looped
        assert_eq!(*player.state(), PlaybackState::Playing);
        assert!(player.current_time() < Duration::from_secs(1));

        // Check for Looped status
        let mut looped = false;
        while let Ok(s) = rx.try_recv() {
            if s == PlaybackStatus::Looped {
                looped = true;
            }
        }
        assert!(looped);
    }

    #[test]
    fn test_playback_play_once() {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(1), 30.0);
        let mut player = VideoPlayer::new(decoder);

        player.set_loop_mode(LoopMode::PlayOnce).unwrap();
        player.play().unwrap();

        // Advance 1.1s
        player.update(Duration::from_millis(1100));

        // Should have stopped
        assert_eq!(*player.state(), PlaybackState::Stopped);
        assert_eq!(player.current_time(), Duration::from_secs(1));
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
        // Not easily testable with current API as update() logs warning but doesn't transition to Error immediately
        // unless loop seek fails.
        // But we can check that it handles it gracefully (no panic).
        let mut decoder = MockDecoder::new();
        decoder.fail_next_frame = true;
        let mut player = VideoPlayer::new(decoder);

        player.play().unwrap();
        let frame = player.update(Duration::from_millis(33));

        // Should return None or last frame, but not panic
        assert!(frame.is_none());
    }
}
