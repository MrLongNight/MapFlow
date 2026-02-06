//! NDI (Network Device Interface) support.
//!
//! This module provides NDI input (receiving) and output (sending) capabilities
//! using the grafton-ndi crate which wraps the official NDI SDK.

#![allow(dead_code, unused_variables)] // TODO: Remove during implementation

#[cfg(feature = "ndi")]
use crate::error::{IoError, Result};
#[cfg(feature = "ndi")]
use crate::format::{FrameData, PixelFormat, VideoFormat, VideoFrame};

#[cfg(feature = "ndi")]
use grafton_ndi::{Find, Finder, FrameType, Receiver, Recv, RecvBandwidth, RecvColorFormat, NDI};
#[cfg(feature = "ndi")]
use std::time::Duration;
#[cfg(feature = "ndi")]
use tracing::{error, info, warn};

/// Re-export Source type for external use
#[cfg(feature = "ndi")]
pub use grafton_ndi::Source;

/// Wrapper for NDI Source that is Send + Sync safe
#[cfg(feature = "ndi")]
#[derive(Debug, Clone)]
pub struct NdiSource {
    /// The name of the NDI source
    pub name: String,
    /// The URL/address of the source (if available)
    pub url_address: Option<String>,
}

#[cfg(feature = "ndi")]
impl From<grafton_ndi::Source> for NdiSource {
    fn from(source: grafton_ndi::Source) -> Self {
        Self {
            name: source.name.clone(),
            url_address: source.url_address.clone(),
        }
    }
}

/// NDI library handle - ensures NDI is initialized
#[cfg(feature = "ndi")]
struct NdiHandle {
    _ndi: NDI,
}

#[cfg(feature = "ndi")]
impl NdiHandle {
    fn new() -> Result<Self> {
        let ndi = NDI::new().map_err(|e| {
            error!("Failed to initialize NDI library: {}", e);
            IoError::NdiError(format!("Failed to initialize NDI: {}", e))
        })?;
        info!("NDI library initialized successfully");
        Ok(Self { _ndi: ndi })
    }
}

/// NDI receiver for capturing video from NDI sources.
#[cfg(feature = "ndi")]
pub struct NdiReceiver {
    /// NDI library handle
    _handle: NdiHandle,
    /// Current source info
    source_info: Option<NdiSource>,
    /// Video format
    format: VideoFormat,
    /// Frame counter
    frame_count: u64,
    /// NDI receiver instance
    recv: Option<Recv>,
}

#[cfg(feature = "ndi")]
impl NdiReceiver {
    /// Creates a new NDI receiver.
    pub fn new() -> Result<Self> {
        info!("Creating NDI Receiver");
        let handle = NdiHandle::new()?;

        Ok(Self {
            _handle: handle,
            source_info: None,
            format: VideoFormat::hd_1080p30_rgba(),
            frame_count: 0,
            recv: None,
        })
    }

    /// Discovers available NDI sources on the network.
    ///
    /// This is a blocking call that waits for the specified timeout.
    pub fn discover_sources(timeout_ms: u32) -> Result<Vec<NdiSource>> {
        info!("Starting NDI source discovery for {}ms", timeout_ms);

        // Initialize NDI temporarily for discovery
        let _ndi = NDI::new().map_err(|e| {
            error!("Failed to initialize NDI for discovery: {}", e);
            IoError::NdiError(format!("Failed to initialize NDI: {}", e))
        })?;

        let finder = Finder::default();
        let ndi_find = Find::new(finder).map_err(|e| {
            error!("Failed to create NDI finder: {}", e);
            IoError::NdiError(format!("Failed to create NDI finder: {}", e))
        })?;

        // Wait for sources
        if !ndi_find.wait_for_sources(timeout_ms) {
            info!("No NDI sources found within timeout");
            return Ok(vec![]);
        }

        let sources = ndi_find.get_sources(timeout_ms);
        let ndi_sources: Vec<NdiSource> = sources.into_iter().map(|s| s.into()).collect();

        info!("Found {} NDI sources", ndi_sources.len());
        for source in &ndi_sources {
            info!("  - {}", source.name);
        }

        Ok(ndi_sources)
    }

    /// Discovers sources asynchronously via a channel
    pub fn discover_sources_async(sender: std::sync::mpsc::Sender<Vec<NdiSource>>) {
        std::thread::spawn(move || match Self::discover_sources(3000) {
            Ok(sources) => {
                let _ = sender.send(sources);
            }
            Err(e) => {
                error!("NDI discovery failed: {}", e);
                let _ = sender.send(vec![]);
            }
        });
    }

    /// Connects to a specific NDI source by name.
    pub fn connect(&mut self, source: &NdiSource) -> Result<()> {
        info!("Connecting to NDI source: {}", source.name);

        // We need a Source object for the receiver
        // First, discover to find the matching source
        let _ndi = NDI::new()
            .map_err(|e| IoError::NdiError(format!("Failed to initialize NDI: {}", e)))?;

        let finder = Finder::default();
        let ndi_find = Find::new(finder)
            .map_err(|e| IoError::NdiError(format!("Failed to create finder: {}", e)))?;

        // Wait briefly for sources
        ndi_find.wait_for_sources(2000);
        let sources = ndi_find.get_sources(1000);

        let matching_source = sources
            .into_iter()
            .find(|s| s.name == source.name)
            .ok_or_else(|| {
                IoError::NdiError(format!("Source '{}' not found on network", source.name))
            })?;

        // Create receiver
        let receiver = Receiver::new(
            matching_source,
            RecvColorFormat::UYVY_BGRA, // BGRA for easy GPU upload
            RecvBandwidth::Highest,
            false, // No interlaced fields
            Some(format!("MapFlow-{}", source.name)),
        );

        let recv = Recv::new(receiver).map_err(|e| {
            error!("Failed to create NDI receiver: {}", e);
            IoError::NdiError(format!("Failed to create receiver: {}", e))
        })?;

        self.recv = Some(recv);
        self.source_info = Some(source.clone());

        info!("Successfully connected to NDI source: {}", source.name);
        Ok(())
    }

    /// Receives a frame with a timeout.
    /// Returns Ok(Some(frame)) if a frame was received,
    /// Ok(None) if no frame was available within the timeout.
    pub fn receive(&mut self, timeout: Duration) -> Result<Option<VideoFrame>> {
        let recv = self
            .recv
            .as_ref()
            .ok_or_else(|| IoError::NdiError("Not connected to any source".to_string()))?;

        let timeout_ms = timeout.as_millis() as u32;

        match recv.capture(timeout_ms) {
            Ok(FrameType::Video(video_frame)) => {
                // Extract frame data from fields
                let width = video_frame.xres as u32;
                let height = video_frame.yres as u32;
                let frame_rate =
                    video_frame.frame_rate_n as f32 / video_frame.frame_rate_d.max(1) as f32;

                // Calculate data size (BGRA = 4 bytes per pixel)
                let data_size = (width * height * 4) as usize;

                // Extract data from raw pointer (unsafe but necessary for NDI)
                let data = if !video_frame.p_data.is_null() && data_size > 0 {
                    unsafe { std::slice::from_raw_parts(video_frame.p_data, data_size).to_vec() }
                } else {
                    warn!("NDI frame has null data pointer");
                    return Ok(None);
                };

                let format = VideoFormat {
                    width,
                    height,
                    pixel_format: PixelFormat::BGRA8,
                    frame_rate,
                };

                self.format = format.clone();
                self.frame_count += 1;

                let frame = VideoFrame {
                    data: FrameData::Cpu(data),
                    format,
                    timestamp: Duration::from_nanos(video_frame.timestamp.max(0) as u64),
                    metadata: Default::default(),
                };

                Ok(Some(frame))
            }
            Ok(FrameType::None) => Ok(None),
            Ok(FrameType::Audio(_)) => Ok(None), // Ignore audio frames
            Ok(FrameType::Metadata(_)) => Ok(None), // Ignore metadata
            Err(e) => {
                warn!("NDI capture error: {:?}", e);
                Ok(None)
            }
        }
    }

    /// Returns the name of the connected source, if any.
    pub fn source_name(&self) -> Option<&str> {
        self.source_info.as_ref().map(|s| s.name.as_str())
    }

    /// Returns whether a source is connected.
    pub fn is_connected(&self) -> bool {
        self.recv.is_some()
    }
}

#[cfg(feature = "ndi")]
impl Default for NdiReceiver {
    fn default() -> Self {
        Self::new().expect("Failed to create default NdiReceiver")
    }
}

// Note: NdiReceiver cannot implement VideoSource trait because Recv is not Send.
// The receiver must be used on the same thread where it was created.
// For cross-thread usage, wrap in a dedicated thread with channels.

/// NDI sender for broadcasting video to the network.
/// Note: Sender implementation is a placeholder - grafton-ndi 0.2.4 Send API needs verification.
#[cfg(feature = "ndi")]
pub struct NdiSender {
    /// NDI library handle
    _handle: NdiHandle,
    /// Sender name
    name: String,
    /// Video format
    format: VideoFormat,
    /// Frame counter
    frame_count: u64,
    /// NDI send instance
    send: Option<grafton_ndi::Send>,
}

#[cfg(feature = "ndi")]
impl NdiSender {
    /// Creates a new NDI sender with the given name.
    pub fn new(name: impl Into<String>, format: VideoFormat) -> Result<Self> {
        let name = name.into();
        info!("Creating NDI Sender: {}", name);

        let handle = NdiHandle::new()?;

        // Create sender - using Sender struct directly
        // Note: The exact API for Send creation in grafton-ndi 0.2.4 may vary
        let sender = grafton_ndi::Sender {
            name: name.clone(),
            groups: None,
            clock_video: false,
            clock_audio: false,
        };

        let send = grafton_ndi::Send::new(sender).map_err(|e| {
            error!("Failed to create NDI sender: {}", e);
            IoError::NdiSenderFailed(format!("Failed to create sender: {}", e))
        })?;

        info!("NDI Sender '{}' created successfully", name);

        Ok(Self {
            _handle: handle,
            name,
            format,
            frame_count: 0,
            send: Some(send),
        })
    }

    /// Sends a video frame.
    pub fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        let send = self
            .send
            .as_ref()
            .ok_or_else(|| IoError::NdiSenderFailed("Sender not initialized".to_string()))?;

        // Extract CPU data
        let data = match &frame.data {
            FrameData::Cpu(data) => data,
            FrameData::Gpu(_) => {
                return Err(IoError::NdiSenderFailed(
                    "GPU frames must be downloaded to CPU before sending".to_string(),
                ));
            }
        };

        // Create NDI video frame and send
        // Note: The exact VideoFrame creation API may differ
        self.frame_count += 1;

        // TODO: Implement actual frame sending when Send API is clarified
        warn!("NdiSender::send_frame not fully implemented yet");

        Ok(())
    }

    /// Returns the sender name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of frames sent.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

// Stub implementations when NDI feature is disabled
/// NDI receiver stub (NDI feature disabled)
#[cfg(not(feature = "ndi"))]
#[derive(Debug)]
pub struct NdiReceiver;

#[cfg(not(feature = "ndi"))]
impl NdiReceiver {
    /// Create a new NDI receiver stub
    pub fn new() -> std::result::Result<Self, String> {
        Err("NDI feature not enabled".to_string())
    }
}

/// NDI sender stub (NDI feature disabled)
#[cfg(not(feature = "ndi"))]
#[derive(Debug)]
pub struct NdiSender;

#[cfg(not(feature = "ndi"))]
impl NdiSender {
    /// Create a new NDI sender stub
    pub fn new(
        _name: impl Into<String>,
        _format: crate::format::VideoFormat,
    ) -> std::result::Result<Self, String> {
        Err("NDI feature not enabled".to_string())
    }
}

#[cfg(all(test, not(feature = "ndi")))]
mod tests {
    use super::*;

    #[test]
    fn test_ndi_receiver_stub_error() {
        let result = NdiReceiver::new();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "NDI feature not enabled");
    }

    #[test]
    fn test_ndi_sender_stub_error() {
        let result = NdiSender::new(
            "test",
            crate::format::VideoFormat::new(1920, 1080, crate::format::PixelFormat::RGBA8, 60.0),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "NDI feature not enabled");
    }
}

/// NDI source stub (NDI feature disabled)
#[cfg(not(feature = "ndi"))]
#[derive(Debug, Clone)]
pub struct NdiSource {
    /// Source name
    pub name: String,
    /// Source URL
    pub url_address: Option<String>,
}
