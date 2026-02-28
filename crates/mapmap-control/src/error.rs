//! Error types for the control system
use thiserror::Error;

/// Control system errors
#[derive(Error, Debug)]
pub enum ControlError {
    #[error("MIDI error: {0}")]
/// Error: MIDI error.
    /// Error: MIDI error.
    MidiError(String),

    #[error("MIDI connection error: {0}")]
/// Error: MIDI connection error.
    /// Error: MIDI connection error.
    #[cfg(feature = "midi")]
    MidiConnectionError(#[from] midir::ConnectError<midir::MidiInput>),

    #[error("MIDI init error: {0}")]
/// Error: MIDI init error.
    /// Error: MIDI init error.
    #[cfg(feature = "midi")]
    MidiInitError(#[from] midir::InitError),

    #[error("MIDI send error: {0}")]
/// Error: MIDI send error.
    /// Error: MIDI send error.
    #[cfg(feature = "midi")]
    MidiSendError(#[from] midir::SendError),

    #[error("OSC error: {0}")]
/// Error: OSC error.
    /// Error: OSC error.
    OscError(String),

    #[error("DMX error: {0}")]
/// Error: DMX error.
    /// Error: DMX error.
    DmxError(String),

    #[error("HTTP error: {0}")]
/// Error: HTTP error.
    /// Error: HTTP error.
    HttpError(String),

    #[error("IO error: {0}")]
/// Error: IO error.
    /// Error: IO error.
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
/// Error: JSON error.
    /// Error: JSON error.
    JsonError(#[from] serde_json::Error),

    #[error("Invalid parameter: {0}")]
/// Error: Invalid parameter.
    /// Error: Invalid parameter.
    InvalidParameter(String),

    #[error("Target not found: {0}")]
/// Error: Target not found.
    /// Error: Target not found.
    TargetNotFound(String),

    #[error("Invalid message: {0}")]
/// Error: Invalid message.
    /// Error: Invalid message.
    InvalidMessage(String),
}

/// Result type for control operations
pub type Result<T> = std::result::Result<T, ControlError>;