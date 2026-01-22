//! Error types for the control system
use thiserror::Error;

/// Control system errors
#[derive(Error, Debug)]
pub enum ControlError {
    /// Generic MIDI error
    #[error("MIDI error: {0}")]
    MidiError(String),

    /// MIDI connection failure
    #[error("MIDI connection error: {0}")]
    #[cfg(feature = "midi")]
    MidiConnectionError(#[from] midir::ConnectError<midir::MidiInput>),

    /// MIDI initialization failure
    #[error("MIDI init error: {0}")]
    #[cfg(feature = "midi")]
    MidiInitError(#[from] midir::InitError),

    /// MIDI send failure
    #[error("MIDI send error: {0}")]
    #[cfg(feature = "midi")]
    MidiSendError(#[from] midir::SendError),

    /// OSC protocol error
    #[error("OSC error: {0}")]
    OscError(String),

    /// DMX/ArtNet/sACN error
    #[error("DMX error: {0}")]
    DmxError(String),

    /// Web API or HTTP server error
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// Standard IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/Deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid control parameter value
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Control target (e.g. layer or parameter) not found
    #[error("Target not found: {0}")]
    TargetNotFound(String),

    /// Malformed or invalid control message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// Result type for control operations
pub type Result<T> = std::result::Result<T, ControlError>;
