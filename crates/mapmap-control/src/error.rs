//! Error types for the control system
use thiserror::Error;

/// Control system errors
#[derive(Error, Debug)]
pub enum ControlError {
    /// Generic MIDI error
    #[error("MIDI error: {0}")]
    MidiError(String),

    /// MIDI connection error
    #[error("MIDI connection error: {0}")]
    #[cfg(feature = "midi")]
    MidiConnectionError(#[from] midir::ConnectError<midir::MidiInput>),

    /// MIDI initialization error
    #[error("MIDI init error: {0}")]
    #[cfg(feature = "midi")]
    MidiInitError(#[from] midir::InitError),

    /// MIDI transmission error
    #[error("MIDI send error: {0}")]
    #[cfg(feature = "midi")]
    MidiSendError(#[from] midir::SendError),

    /// OSC error
    #[error("OSC error: {0}")]
    OscError(String),

    /// DMX error
    #[error("DMX error: {0}")]
    DmxError(String),

    /// HTTP API error
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// I/O error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid parameter value
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Control target not found
    #[error("Target not found: {0}")]
    TargetNotFound(String),

    /// Invalid message format
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// Result type for control operations
pub type Result<T> = std::result::Result<T, ControlError>;
