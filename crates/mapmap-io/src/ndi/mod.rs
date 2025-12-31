//! NDI (Network Device Interface) support.
#![allow(dead_code, unused_variables)] // TODO: Remove during implementation

#[cfg(feature = "ndi")]
use crate::error::{IoError, Result};
#[cfg(feature = "ndi")]
use crate::format::{VideoFormat, VideoFrame};
#[cfg(feature = "ndi")]
use crate::sink::VideoSink;
#[cfg(feature = "ndi")]
use crate::source::VideoSource;

#[cfg(feature = "ndi")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ndi")]
use std::time::Duration;
#[cfg(feature = "ndi")]
use tokio::runtime::Runtime;
#[cfg(feature = "ndi")]
use tokio::sync::{watch, oneshot};
#[cfg(feature = "ndi")]
use tracing::{error, info, warn};
#[cfg(feature = "ndi")]
use once_cell::sync::Lazy;

#[cfg(feature = "ndi")]
static NDI_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create NDI Tokio runtime")
});

/// Information about an NDI source on the network.
#[cfg(feature = "ndi")]
pub use grafton_ndi::Source;

/// NDI receiver for capturing video from NDI sources.
#[cfg(feature = "ndi")]
pub struct NdiReceiver {
    source_name: String,
    format: VideoFormat,
    frame_count: u64,
    // NDI receiver instance
    _receiver: Option<grafton_ndi::Recv>, // Becomes managed by the async task
    // Watch channel to get the latest frame
    frame_rx: Option<watch::Receiver<Option<Arc<VideoFrame>>>>,
    // Channel to stop the receiver task
    stop_tx: Option<oneshot::Sender<()>>,
}

#[cfg(feature = "ndi")]
impl NdiReceiver {
    /// Creates a new NDI receiver.
    pub fn new() -> Result<Self> {
        info!("Initializing NDI Receiver");
        grafton_ndi::initialize().map_err(|e| {
            error!("Failed to initialize NDI: {}", e);
            IoError::NdiError(format!("Failed to initialize NDI: {}", e))
        })?;

        Ok(Self {
            source_name: "Not Connected".to_string(),
            // Default to a common format, will be updated on connection
            format: VideoFormat::hd_1080p30_rgba(),
            frame_count: 0,
            _receiver: None,
            frame_rx: None,
            stop_tx: None,
        })
    }

    /// Discovers available NDI sources on the network asynchronously.
    pub fn discover_sources_async(sender: std::sync::mpsc::Sender<Vec<Source>>) {
        NDI_RUNTIME.spawn(async move {
            info!("Starting NDI source discovery for 2000ms");
            let result = async {
                let finder = grafton_ndi::Finder::new(false, None, None).await.map_err(|e| {
                    error!("Failed to create NDI finder: {}", e);
                    IoError::NdiError(format!("Failed to create NDI finder: {}", e))
                })?;

                tokio::time::sleep(Duration::from_millis(2000)).await;

                let sources = finder.sources().await.map_err(|e| {
                    error!("Failed to get NDI sources: {}", e);
                    IoError::NdiError(format!("Failed to get NDI sources: {}", e))
                })?;
                info!("Found {} NDI sources", sources.len());
                Ok(sources)
            }.await;

            match result {
                Ok(sources) => {
                    let _ = sender.send(sources);
                }
                Err(e) => {
                    error!("NDI discovery failed: {}", e);
                    let _ = sender.send(vec![]);
                }
            }
        });
    }

    /// Discovers available NDI sources on the network asynchronously.
    pub async fn discover_sources(timeout_ms: u32) -> Result<Vec<Source>> {
        info!("Starting NDI source discovery for {}ms", timeout_ms);
        let finder = grafton_ndi::Finder::new(false, None, None).await.map_err(|e| {
            error!("Failed to create NDI finder: {}", e);
            IoError::NdiError(format!("Failed to create NDI finder: {}", e))
        })?;

        tokio::time::sleep(Duration::from_millis(timeout_ms as u64)).await;

        let sources = finder.sources().await.map_err(|e| {
            error!("Failed to get NDI sources: {}", e);
            IoError::NdiError(format!("Failed to get NDI sources: {}", e))
        })?;
        info!("Found {} NDI sources", sources.len());
        Ok(sources)
    }

    /// Connects to a specific NDI source.
    pub fn connect(&mut self, source: &Source) -> Result<()> {
        info!("Connecting to NDI source: {}", source.name);
        let recv_builder = grafton_ndi::RecvBuilder::new()
            .source(source.clone())
            .allow_video_fields(false)
            .color_format(grafton_ndi::ColorFormat::UYVY_RGBA)
            .build();

        let receiver = NDI_RUNTIME.block_on(recv_builder).map_err(|e| {
            error!("Failed to create NDI receiver for source {}: {}", source.name, e);
            IoError::NdiError(format!("Failed to create NDI receiver: {}", e))
        })?;

        // Stop any existing receiver task
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        let (stop_tx, mut stop_rx) = oneshot::channel();
        let (frame_tx, frame_rx) = watch::channel(None);

        self.source_name = source.name.clone();
        self.frame_rx = Some(frame_rx);
        self.stop_tx = Some(stop_tx);

        NDI_RUNTIME.spawn(async move {
            loop {
                tokio::select! {
                    frame_result = receiver.recv_video() => {
                        match frame_result {
                            Ok(Some(frame)) => {
                                let video_frame = VideoFrame {
                                    format: VideoFormat {
                                        width: frame.width() as u32,
                                        height: frame.height() as u32,
                                        frame_rate: frame.frame_rate() as f32,
                                        ..Default::default()
                                    },
                                    data: frame.data().to_vec(),
                                    timestamp: Duration::from_nanos(frame.timestamp().unwrap_or(0) as u64),
                                };
                                if frame_tx.send(Some(Arc::new(video_frame))).is_err() {
                                    // Receiver was dropped, stop the task
                                    break;
                                }
                            }
                            Ok(None) => continue, // No frame available yet
                            Err(e) => {
                                error!("Error receiving NDI frame: {}", e);
                                break;
                            }
                        }
                    }
                    _ = &mut stop_rx => {
                        info!("NDI receiver task stopped for source {}", source.name);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(feature = "ndi")]
impl Drop for NdiReceiver {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
}

#[cfg(feature = "ndi")]
impl VideoSource for NdiReceiver {
    fn name(&self) -> &str {
        &self.source_name
    }

    fn format(&self) -> VideoFormat {
        self.format.clone()
    }

    fn receive_frame(&mut self) -> Result<VideoFrame> {
        let frame_rx = self.frame_rx.as_mut().ok_or(IoError::NdiError("Not connected to a source".to_string()))?;

        // Check if there is a new frame
        if frame_rx.has_changed().unwrap_or(false) {
             let received = frame_rx.borrow_and_update();
             if let Some(frame_arc) = received.as_ref() {
                let frame = (*frame_arc).clone();
                // Update format if it has changed
                if self.format.width != frame.format.width || self.format.height != frame.format.height {
                    self.format = frame.format.clone();
                }
                self.frame_count += 1;
                return Ok(frame);
             }
        }

        Err(IoError::NoFrameAvailable)
    }

    fn is_available(&self) -> bool {
        self.frame_rx.is_some()
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// NDI sender for broadcasting video to the network.
#[cfg(feature = "ndi")]
pub struct NdiSender {
    name: String,
    format: VideoFormat,
    frame_count: u64,
    frame_tx: mpsc::Sender<Arc<VideoFrame>>,
    stop_tx: Option<oneshot::Sender<()>>,
}

#[cfg(feature = "ndi")]
impl NdiSender {
    /// Creates a new NDI sender.
    pub fn new(name: impl Into<String>, format: VideoFormat) -> Result<Self> {
        let name = name.into();
        info!("Initializing NDI Sender with name: {}", name);
        grafton_ndi::initialize().map_err(|e| {
            error!("Failed to initialize NDI: {}", e);
            IoError::NdiError(format!("Failed to initialize NDI: {}", e))
        })?;

        let send_builder = grafton_ndi::SendBuilder::new()
            .name(name.clone())
            .build();

        let sender = NDI_RUNTIME.block_on(send_builder).map_err(|e| {
            error!("Failed to create NDI sender: {}", e);
            IoError::NdiSenderFailed(format!("Failed to create NDI sender: {}", e))
        })?;

        let (stop_tx, mut stop_rx) = oneshot::channel();
        let (frame_tx, mut frame_rx) = mpsc::channel(2); // Small buffer

        NDI_RUNTIME.spawn(async move {
            loop {
                tokio::select! {
                    Some(frame) = frame_rx.recv() => {
                        let ndi_frame = grafton_ndi::VideoFrame::new(
                            frame.format.width as _,
                            frame.format.height as _,
                            grafton_ndi::FourCC::RGBA,
                            &frame.data,
                        )
                        .frame_rate(frame.format.frame_rate as _)
                        .timestamp(frame.timestamp.as_nanos() as i64)
                        .build();

                        match ndi_frame {
                            Ok(ndi_frame) => {
                                if let Err(e) = sender.send_video(&ndi_frame).await {
                                    error!("Failed to send NDI frame: {}", e);
                                }
                            }
                            Err(e) => error!("Failed to build NDI frame: {}", e),
                        }
                    }
                    _ = &mut stop_rx => {
                        info!("NDI sender task stopped for {}", name);
                        break;
                    }
                    else => break, // Channel closed
                }
            }
        });

        Ok(Self {
            name,
            format,
            frame_count: 0,
            frame_tx,
            stop_tx: Some(stop_tx),
        })
    }
}

#[cfg(feature = "ndi")]
impl Drop for NdiSender {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
}

#[cfg(feature = "ndi")]
impl VideoSink for NdiSender {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> VideoFormat {
        self.format.clone()
    }

    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        // This is now a non-blocking send to the background task
        match self.frame_tx.try_send(Arc::new(frame.clone())) {
            Ok(_) => {
                self.frame_count += 1;
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("NDI sender channel is full, dropping frame.");
                // Not a fatal error, just indicates the network can't keep up
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("NDI sender task has panicked or closed.");
                Err(IoError::NdiSenderFailed("Sender task has closed".to_string()))
            }
        }
    }

    fn is_available(&self) -> bool {
        true // Sender is always "available" once created
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

// Stub types when NDI feature is disabled
#[cfg(not(feature = "ndi"))]
pub struct NdiReceiver;
#[cfg(not(feature = "ndi"))]
impl NdiReceiver {
    pub fn new() -> crate::error::Result<Self> {
        Err(crate::error::IoError::feature_not_enabled("NDI", "ndi"))
    }
    pub async fn discover_sources(_timeout_ms: u32) -> crate::error::Result<Vec<Source>> {
        Err(crate::error::IoError::feature_not_enabled("NDI", "ndi"))
    }
}
#[cfg(not(feature = "ndi"))]
pub struct NdiSender;
#[cfg(not(feature = "ndi"))]
impl NdiSender {
    pub fn new(_name: impl Into<String>, _format: crate::format::VideoFormat) -> crate::error::Result<Self> {
        Err(crate::error::IoError::feature_not_enabled("NDI", "ndi"))
    }
}
#[cfg(not(feature = "ndi"))]
#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    // Note: NDI tests are hard to run in a headless CI environment.
    // These tests primarily check that the API doesn't panic and that
    // stubs work as expected.
    use super::*;

    #[test]
    fn test_ndi_receiver_creation() {
        if cfg!(feature = "ndi") {
            // This test requires the NDI runtime to be installed.
            // It might fail in environments without it.
            let result = NdiReceiver::new();
            if let Err(e) = &result {
                // It's acceptable for this to fail if the NDI libs aren't found
                println!("NDI Receiver creation failed (as expected in some envs): {:?}", e);
            }
        } else {
            let result = NdiReceiver::new();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_ndi_sender_creation() {
        if cfg!(feature = "ndi") {
            let format = crate::format::VideoFormat::sd_480p30_rgba();
            let result = NdiSender::new("Test Sender", format);
             if let Err(e) = &result {
                println!("NDI Sender creation failed (as expected in some envs): {:?}", e);
            }
        } else {
            let format = crate::format::VideoFormat::sd_480p30_rgba();
            let result = NdiSender::new("Test Sender", format);
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_ndi_discover_sources() {
        if cfg!(feature = "ndi") {
             let result = NdiReceiver::discover_sources(500).await;
             if let Err(e) = &result {
                println!("NDI discovery failed (as expected in some envs): {:?}", e);
            }
        } else {
            let result = NdiReceiver::discover_sources(500).await;
            assert!(result.is_err());
        }
    }
}
