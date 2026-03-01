//! SRT streaming output (stub implementation).
//!
//! Secure Reliable Transport (SRT) is a protocol for low-latency video streaming.
//! This is currently a stub implementation.

#[cfg(feature = "stream")]
use crate::error::{IoError, Result};
#[cfg(feature = "stream")]
use crate::format::{FrameData, VideoFormat, VideoFrame};
#[cfg(feature = "stream")]
use crate::sink::{SinkStatistics, VideoSink};
#[cfg(feature = "stream")]
use crate::stream::encoder::{EncoderPreset, VideoCodec, VideoEncoder};

/// SRT streamer for sending video via SRT protocol.
///
/// SRT (Secure Reliable Transport) provides low-latency streaming with
/// error correction and encryption. This implementation uses FFmpeg
/// built with libsrt (`--enable-libsrt`).
///
/// # Example
///
/// ```ignore
/// use mapmap_io::stream::SrtStreamer;
/// use mapmap_io::format::VideoFormat;
///
/// let format = VideoFormat::hd_1080p60_rgba();
/// let mut streamer = SrtStreamer::new(
///     "srt://localhost:9000",
///     format,
///     6_000_000, // 6 Mbps
/// ).unwrap();
/// ```
#[cfg(feature = "stream")]
pub struct SrtStreamer {
    /// URL of the SRT destination
    url: String,
    /// Video encoder mapping frames to the stream
    encoder: VideoEncoder,
    /// Stream video format
    format: VideoFormat,
    /// Total frames sent successfully
    frame_count: u64,
    /// Network connection state flag
    connected: bool,
}

#[cfg(feature = "stream")]
impl SrtStreamer {
    /// Creates a new SRT streamer.
    ///
    /// # Parameters
    ///
    /// - `url` - SRT URL (e.g., "srt://host:port")
    /// - `format` - Video format to stream
    /// - `bitrate` - Target bitrate in bits per second
    pub fn new(url: impl Into<String>, format: VideoFormat, bitrate: u64) -> Result<Self> {
        let url = url.into();

        // Validate SRT URL
        if !url.starts_with("srt://") {
            return Err(IoError::InvalidParameter(
                "SRT URL must start with srt://".to_string(),
            ));
        }

        let encoder = VideoEncoder::new(
            &url,
            format.clone(),
            VideoCodec::H264,
            bitrate,
            EncoderPreset::UltraFast,
        )?;

        tracing::info!("Created SRT streamer for {}", url);

        Ok(Self {
            url,
            encoder,
            format,
            connected: false,
            frame_count: 0,
        })
    }

    /// Creates a default SRT streamer for 1080p60.
    pub fn default_1080p60(url: impl Into<String>) -> Result<Self> {
        Self::new(url, VideoFormat::hd_1080p60_rgba(), 6_000_000)
    }

    /// Connects to the SRT server.
    pub fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        tracing::info!("Connecting to SRT server: {}", self.url);

        // Encoder connects when the first frame is sent, or when initialized.
        // It's initialized in `new()`, so we just set state to connected.
        self.connected = true;

        Ok(())
    }

    /// Disconnects from the SRT server.
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        tracing::info!("Disconnecting from SRT server");
        self.encoder.flush()?;
        self.connected = false;
        Ok(())
    }

    /// Checks if the streamer is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Returns the SRT URL.
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(feature = "stream")]
impl VideoSink for SrtStreamer {
    fn name(&self) -> &str {
        "SRT Streamer"
    }

    fn format(&self) -> VideoFormat {
        self.format.clone()
    }

    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        if !self.connected {
            self.connect()?;
        }

        // Handle empty frames directly if there's no data
        if let FrameData::Empty = frame.data {
            return Ok(());
        }

        self.encoder.encode(frame)?;
        self.frame_count += 1;
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }

    fn flush(&mut self) -> Result<()> {
        self.encoder.flush()
    }

    fn reconnect(&mut self) -> Result<()> {
        self.disconnect()?;

        // Need to recreate encoder for reconnection
        self.encoder = VideoEncoder::new(
            &self.url,
            self.format.clone(),
            VideoCodec::H264,
            self.encoder.bitrate(),
            EncoderPreset::UltraFast,
        )?;

        self.connect()
    }

    fn statistics(&self) -> Option<SinkStatistics> {
        let stats = self.encoder.statistics();
        Some(SinkStatistics {
            frames_sent: stats.frames_encoded,
            frames_dropped: stats.frames_dropped,
            bitrate: Some(self.encoder.bitrate()),
            average_latency_ms: None,
        })
    }
}

// Stub implementation when stream feature is disabled
/// SRT streamer (stub implementation when feature is disabled)
#[cfg(not(feature = "stream"))]
pub struct SrtStreamer;

#[cfg(not(feature = "stream"))]
impl SrtStreamer {
    /// Create a new SRT streamer (returns error when feature is disabled)
    pub fn new(
        _url: impl Into<String>,
        _format: crate::format::VideoFormat,
        _bitrate: u64,
    ) -> crate::error::Result<Self> {
        Err(crate::error::IoError::feature_not_enabled(
            "SRT streaming",
            "stream",
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "stream")]
mod tests {
    use super::*;

    #[test]
    fn test_srt_streamer_creation() {
        let format = VideoFormat::hd_1080p60_rgba();
        let streamer = SrtStreamer::new("srt://localhost:9000", format, 6_000_000);
        assert!(streamer.is_ok());
    }

    #[test]
    fn test_srt_streamer_invalid_url() {
        let format = VideoFormat::hd_1080p60_rgba();
        let streamer = SrtStreamer::new("http://localhost/stream", format, 6_000_000);
        assert!(streamer.is_err());
    }

    #[test]
    fn test_srt_streamer_not_implemented() {
        let format = VideoFormat::hd_1080p60_rgba();
        let mut streamer = SrtStreamer::new("srt://localhost:9000", format, 6_000_000).unwrap();

        // Connect should fail with not implemented error
        assert!(streamer.connect().is_err());
        assert!(!streamer.is_connected());
    }
}