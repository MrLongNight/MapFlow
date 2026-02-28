//! Error types for video I/O operations.
//!
//! This module defines comprehensive error types for all video I/O operations
//! including NDI, DeckLink, Spout, Syphon, streaming, and format conversion.

/// Result type alias for video I/O operations.
pub type Result<T> = std::result::Result<T, IoError>;

/// Comprehensive error type for video I/O operations.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    /// Generic I/O error
    #[error("I/O error: {0}")]
/// Error: I/O error.
/// Error: I/O error.
    /// Error: I/O error.
    Io(#[from] std::io::Error),

    /// RON serialization error
    #[error("RON Serialization error: {0}")]
/// Error: RON Serialization error.
/// Error: RON Serialization error.
    /// Error: RON Serialization error.
    RonSerialization(#[from] ron::Error),

    /// RON deserialization error
    #[error("RON Deserialization error: {0}")]
/// Error: RON Deserialization error.
/// Error: RON Deserialization error.
    /// Error: RON Deserialization error.
    RonDeserialization(#[from] ron::error::SpannedError),

    /// JSON serialization error
    #[error("JSON Serialization error: {0}")]
/// Error: JSON Serialization error.
/// Error: JSON Serialization error.
    /// Error: JSON Serialization error.
    JsonSerialization(#[from] serde_json::Error),

    /// Unsupported file format for project files or other I/O.
    #[error("Format not supported: {0}")]
/// Error: Format not supported.
/// Error: Format not supported.
    /// Error: Format not supported.
    UnsupportedFormat(String),

    /// File too large
    #[error("File too large: {size} bytes (limit: {limit} bytes)")]
/// Error: File too large.
/// Error: File too large.
    /// Error: File too large.
    FileTooLarge {
        /// File size in bytes
        size: u64,
        /// Size limit in bytes
        limit: u64,
    },

    /// Project file version mismatch
    #[error("Project file version mismatch. Expected {expected}, got {found}.")]
/// Error: Project file version mismatch. Expected {expected}, got {found}..
/// Error: Project file version mismatch. Expected {expected}, got {found}..
    /// Error: Project file version mismatch. Expected {expected}, got {found}..
    VersionMismatch {
        /// The version expected by the application.
        expected: String,
        /// The version found in the project file.
        found: String,
    },

    /// NDI-related errors
    #[error("NDI error: {0}")]
/// Error: NDI error.
/// Error: NDI error.
    /// Error: NDI error.
    NdiError(String),

    /// NDI initialization failed
    #[error("Failed to initialize NDI runtime")]
/// Error: Failed to initialize NDI runtime.
/// Error: Failed to initialize NDI runtime.
    /// Error: Failed to initialize NDI runtime.
    NdiInitFailed,

    /// NDI source not found
    #[error("NDI source not found: {0}")]
/// Error: NDI source not found.
/// Error: NDI source not found.
    /// Error: NDI source not found.
    NdiSourceNotFound(String),

    /// NDI receiver creation failed
    #[error("Failed to create NDI receiver")]
/// Error: Failed to create NDI receiver.
/// Error: Failed to create NDI receiver.
    /// Error: Failed to create NDI receiver.
    NdiReceiverFailed,

    /// NDI sender creation failed
    #[error("Failed to create NDI sender: {0}")]
/// Error: Failed to create NDI sender.
/// Error: Failed to create NDI sender.
    /// Error: Failed to create NDI sender.
    NdiSenderFailed(String),

    /// DeckLink-related errors
    #[error("DeckLink error: {0}")]
/// Error: DeckLink error.
/// Error: DeckLink error.
    /// Error: DeckLink error.
    DeckLinkError(String),

    /// DeckLink device not found
    #[error("DeckLink device not found")]
/// Error: DeckLink device not found.
/// Error: DeckLink device not found.
    /// Error: DeckLink device not found.
    DeckLinkDeviceNotFound,

    /// DeckLink SDK not available
    #[error("DeckLink SDK not available or not installed")]
/// Error: DeckLink SDK not available or not installed.
/// Error: DeckLink SDK not available or not installed.
    /// Error: DeckLink SDK not available or not installed.
    DeckLinkSdkNotAvailable,

    /// Spout-related errors (Windows only)
    #[cfg(target_os = "windows")]
    #[error("Spout error: {0}")]
/// Error: Spout error.
/// Error: Spout error.
    /// Error: Spout error.
    SpoutError(String),

    /// Spout initialization failed
    #[cfg(target_os = "windows")]
    #[error("Failed to initialize Spout")]
/// Error: Failed to initialize Spout.
/// Error: Failed to initialize Spout.
    /// Error: Failed to initialize Spout.
    SpoutInitFailed,

    /// Spout sender/receiver not found
    #[cfg(target_os = "windows")]
    #[error("Spout sender not found: {0}")]
/// Error: Spout sender not found.
/// Error: Spout sender not found.
    /// Error: Spout sender not found.
    SpoutNotFound(String),

    /// Syphon-related errors (macOS only)
    #[cfg(target_os = "macos")]
    #[error("Syphon error: {0}")]
/// Error: Syphon error.
/// Error: Syphon error.
    /// Error: Syphon error.
    SyphonError(String),

    /// Syphon initialization failed
    #[cfg(target_os = "macos")]
    #[error("Failed to initialize Syphon")]
/// Error: Failed to initialize Syphon.
/// Error: Failed to initialize Syphon.
    /// Error: Failed to initialize Syphon.
    SyphonInitFailed,

    /// Syphon server/client not found
    #[cfg(target_os = "macos")]
    #[error("Syphon server not found: {0}")]
/// Error: Syphon server not found.
/// Error: Syphon server not found.
    /// Error: Syphon server not found.
    SyphonNotFound(String),

    /// Streaming errors
    #[error("Stream error: {0}")]
/// Error: Stream error.
/// Error: Stream error.
    /// Error: Stream error.
    StreamError(String),

    /// Failed to initialize encoder
    #[error("Failed to initialize encoder: {0}")]
/// Error: Failed to initialize encoder.
/// Error: Failed to initialize encoder.
    /// Error: Failed to initialize encoder.
    EncoderInitFailed(String),

    /// Failed to encode frame
    #[error("Failed to encode frame: {0}")]
/// Error: Failed to encode frame.
/// Error: Failed to encode frame.
    /// Error: Failed to encode frame.
    EncodeFailed(String),

    /// Failed to connect to streaming server
    #[error("Failed to connect to streaming server: {0}")]
/// Error: Failed to connect to streaming server.
/// Error: Failed to connect to streaming server.
    /// Error: Failed to connect to streaming server.
    StreamConnectionFailed(String),

    /// Stream disconnected
    #[error("Stream disconnected")]
/// Error: Stream disconnected.
/// Error: Stream disconnected.
    /// Error: Stream disconnected.
    StreamDisconnected,

    /// RTMP-specific errors
    #[error("RTMP error: {0}")]
/// Error: RTMP error.
/// Error: RTMP error.
    /// Error: RTMP error.
    RtmpError(String),

    /// SRT-specific errors
    #[error("SRT error: {0}")]
/// Error: SRT error.
/// Error: SRT error.
    /// Error: SRT error.
    SrtError(String),

    /// Format conversion errors
    #[error("Format conversion error: {0}")]
/// Error: Format conversion error.
/// Error: Format conversion error.
    /// Error: Format conversion error.
    ConversionError(String),

    /// Unsupported pixel format
    #[error("Unsupported pixel format: {0}")]
/// Error: Unsupported pixel format.
/// Error: Unsupported pixel format.
    /// Error: Unsupported pixel format.
    UnsupportedPixelFormat(String),

    /// Unsupported video format
    #[error("Unsupported video format: {width}x{height} @ {fps}fps")]
/// Error: Unsupported video format.
/// Error: Unsupported video format.
    /// Error: Unsupported video format.
    UnsupportedVideoFormat {
        /// Video width in pixels
        width: u32,
        /// Video height in pixels
        height: u32,
        /// Frames per second
        fps: f32,
    },

    /// Invalid frame data
    #[error("Invalid frame data: {0}")]
/// Error: Invalid frame data.
/// Error: Invalid frame data.
    /// Error: Invalid frame data.
    InvalidFrameData(String),

    /// Frame size mismatch
    #[error("Frame size mismatch: expected {expected} bytes, got {actual} bytes")]
/// Error: Frame size mismatch.
/// Error: Frame size mismatch.
    /// Error: Frame size mismatch.
    FrameSizeMismatch {
        /// Expected frame size in bytes
        expected: usize,
        /// Actual frame size in bytes
        actual: usize,
    },

    /// Virtual camera errors
    #[error("Virtual camera error: {0}")]
/// Error: Virtual camera error.
/// Error: Virtual camera error.
    /// Error: Virtual camera error.
    VirtualCameraError(String),

    /// Virtual camera not available
    #[error("Virtual camera not available on this platform")]
/// Error: Virtual camera not available on this platform.
/// Error: Virtual camera not available on this platform.
    /// Error: Virtual camera not available on this platform.
    VirtualCameraNotAvailable,

    /// No frame available
    #[error("No frame available")]
/// Error: No frame available.
/// Error: No frame available.
    /// Error: No frame available.
    NoFrameAvailable,

    /// Timeout waiting for frame
    #[error("Timeout waiting for frame")]
/// Error: Timeout waiting for frame.
/// Error: Timeout waiting for frame.
    /// Error: Timeout waiting for frame.
    FrameTimeout,

    /// Device not available
    #[error("Device not available: {0}")]
/// Error: Device not available.
/// Error: Device not available.
    /// Error: Device not available.
    DeviceNotAvailable(String),

    /// Feature not enabled
    #[error("Feature not enabled: {0}. Enable the '{1}' feature flag to use this functionality")]
/// Error: Feature not enabled.
/// Error: Feature not enabled.
    /// Error: Feature not enabled.
    FeatureNotEnabled(String, String),

    /// Platform not supported
    #[error("Operation not supported on this platform: {0}")]
/// Error: Operation not supported on this platform.
/// Error: Operation not supported on this platform.
    /// Error: Operation not supported on this platform.
    PlatformNotSupported(String),

    /// Resource allocation failed
    #[error("Failed to allocate resource: {0}")]
/// Error: Failed to allocate resource.
/// Error: Failed to allocate resource.
    /// Error: Failed to allocate resource.
    AllocationFailed(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
/// Error: Invalid parameter.
/// Error: Invalid parameter.
    /// Error: Invalid parameter.
    InvalidParameter(String),

    /// Other errors
    #[error("{0}")]
/// Error: {0}.
/// Error: {0}.
    /// Error: {0}.
    Other(String),
}

impl IoError {
    /// Creates a new generic error with a custom message.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Creates a feature not enabled error.
    pub fn feature_not_enabled(feature: &str, feature_flag: &str) -> Self {
        Self::FeatureNotEnabled(feature.to_string(), feature_flag.to_string())
    }

    /// Creates a platform not supported error.
    pub fn platform_not_supported(operation: &str) -> Self {
        Self::PlatformNotSupported(operation.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IoError::NdiError("test error".to_string());
        assert_eq!(err.to_string(), "NDI error: test error");
    }

    #[test]
    fn test_feature_not_enabled() {
        let err = IoError::feature_not_enabled("NDI", "ndi");
        assert!(err.to_string().contains("ndi"));
    }

    #[test]
    fn test_frame_size_mismatch() {
        let err = IoError::FrameSizeMismatch {
            expected: 1920 * 1080 * 4,
            actual: 1000,
        };
        let err_str = err.to_string();
        assert!(err_str.contains("expected"));
        assert!(err_str.contains("got"));
        assert!(err_str.contains("8294400"));
        assert!(err_str.contains("1000"));
    }
}
