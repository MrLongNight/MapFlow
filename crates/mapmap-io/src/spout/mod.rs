//! Spout texture sharing support (Windows only).
//!
//! Spout is a Windows-only system for sharing textures between applications
//! using DirectX shared resources.
//!
//! # Note
//!
//! Spout support requires the Spout SDK and DirectX 11, which are not included.
//! This is currently a stub implementation showing the intended API.
//!
//! To enable full Spout support:
//! 1. Download the Spout SDK from https://spout.zeal.co/
//! 2. Create FFI bindings for the Spout library
//! 3. Integrate with wgpu's DirectX 11 backend
//! 4. Implement sender and receiver

#[cfg(all(feature = "spout", target_os = "windows"))]
use crate::error::{IoError, Result};
#[cfg(all(feature = "spout", target_os = "windows"))]
use crate::format::{FrameData, VideoFormat, VideoFrame};
#[cfg(all(feature = "spout", target_os = "windows"))]
use crate::sink::VideoSink;
#[cfg(all(feature = "spout", target_os = "windows"))]
use crate::source::VideoSource;
#[cfg(all(feature = "spout", target_os = "windows"))]
use rusty_spout::RustySpout;
#[cfg(all(feature = "spout", target_os = "windows"))]
use std::{ffi::CString, sync::Arc};
#[cfg(all(feature = "spout", target_os = "windows"))]
use mapmap_render::wgpu;

/// Information about a Spout sender.
#[cfg(all(feature = "spout", target_os = "windows"))]
#[derive(Debug, Clone)]
pub struct SpoutSenderInfo {
    /// Sender name
    pub name: String,
    /// Video format
    pub format: VideoFormat,
}

/// Spout receiver for receiving textures from other applications.
///
/// # Note
///
/// This is a stub implementation. Full implementation requires the Spout SDK.
#[cfg(all(feature = "spout", target_os = "windows"))]
pub struct SpoutReceiver {
    spout: RustySpout,
    name: String,
    format: VideoFormat,
    frame_count: u64,
    device: Arc<wgpu::Device>,
}

#[cfg(all(feature = "spout", target_os = "windows"))]
impl SpoutReceiver {
    /// Creates a new Spout receiver.
    pub fn new(device: Arc<wgpu::Device>) -> Result<Self> {
        let mut spout = RustySpout::new();
        spout.get_spout().map_err(|_| IoError::SpoutInitFailed)?;
        Ok(Self {
            spout,
            name: String::new(),
            format: VideoFormat::default(),
            frame_count: 0,
            device,
        })
    }

    /// Lists available Spout senders.
    pub fn list_senders() -> Result<Vec<SpoutSenderInfo>> {
        let mut spout = RustySpout::new();
        spout.get_spout().map_err(|_| IoError::SpoutInitFailed)?;
        let count = spout.get_sender_count().map_err(|e| IoError::SpoutError(e.to_string()))?;
        let mut senders = Vec::new();
        for i in 0..count {
            let (_, name) = spout.get_sender(i, 256).map_err(|e| IoError::SpoutError(e.to_string()))?;
            let mut width = 0;
            let mut height = 0;
            let mut handle = std::ptr::null_mut();
            let mut format = 0;
            let success = spout.get_sender_info(&name, &mut width, &mut height, &mut handle, &mut format).map_err(|e| IoError::SpoutError(e.to_string()))?;
            if success {
                senders.push(SpoutSenderInfo {
                    name,
                    format: VideoFormat {
                        width,
                        height,
                        framerate: 0.0,
                        fourcc: 0,
                    },
                });
            }
        }
        Ok(senders)
    }

    /// Connects to a specific Spout sender.
    pub fn connect(&mut self, sender_name: &str) -> Result<()> {
        self.spout.set_receiver_name(sender_name).map_err(|e| IoError::SpoutError(e.to_string()))?;
        let width = self.spout.get_sender_width().map_err(|e| IoError::SpoutError(e.to_string()))?;
        let height = self.spout.get_sender_height().map_err(|e| IoError::SpoutError(e.to_string()))?;
        self.name = sender_name.to_string();
        self.format = VideoFormat {
            width,
            height,
            framerate: 0.0,
            fourcc: 0,
        };
        Ok(())
    }
}

#[cfg(all(feature = "spout", target_os = "windows"))]
impl VideoSource for SpoutReceiver {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> VideoFormat {
        self.format.clone()
    }

    fn receive_frame(&mut self) -> Result<VideoFrame> {
        if !self.spout.is_connected().unwrap_or(false) {
            return Err(IoError::SpoutError("Not connected to a sender".to_string()));
        }

        if !self.spout.is_frame_new().unwrap_or(false) {
            // Return an empty frame or a special status? For now, an error.
            return Err(IoError::SpoutError("No new frame available".to_string()));
        }

        let handle = self.spout.get_sender_handle().map_err(|e| IoError::SpoutError(e.to_string()))?;
        let non_null_handle = std::ptr::NonNull::new(handle).ok_or(IoError::SpoutError("Received null handle".to_string()))?;
        let width = self.spout.get_sender_width().map_err(|e| IoError::SpoutError(e.to_string()))?;
        let height = self.spout.get_sender_height().map_err(|e| IoError::SpoutError(e.to_string()))?;
        let format = self.spout.get_sender_format().map_err(|e| IoError::SpoutError(e.to_string()))?;

        // Convert the DXGI format to a wgpu::TextureFormat
        let texture_format = match format {
            28 => wgpu::TextureFormat::Rgba8UnormSrgb, // DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
            _ => wgpu::TextureFormat::Bgra8UnormSrgb, // Default to BGRA
        };

        let texture = unsafe { mapmap_render::spout::texture_from_shared_handle(&self.device, non_null_handle, width, height, texture_format) }.map_err(|e| IoError::SpoutError(e.to_string()))?;

        Ok(VideoFrame {
            data: FrameData::Gpu(texture),
            format: self.format.clone(),
            timestamp: std::time::Duration::from_secs(0),
            metadata: Default::default(),
        })
    }

    fn is_available(&self) -> bool {
        self.spout.is_connected().unwrap_or(false)
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Spout sender for sharing textures with other applications.
///
/// # Note
///
/// This is a stub implementation. Full implementation requires the Spout SDK.
#[cfg(all(feature = "spout", target_os = "windows"))]
pub struct SpoutSender {
    spout: RustySpout,
    name: String,
    format: VideoFormat,
    frame_count: u64,
    device: Arc<wgpu::Device>,
}

#[cfg(all(feature = "spout", target_os = "windows"))]
impl SpoutSender {
    /// Creates a new Spout sender.
    ///
    /// # Parameters
    ///
    /// - `name` - Name of this Spout sender (visible to receivers)
    /// - `format` - Video format to share
    pub fn new(name: impl Into<String>, format: VideoFormat, device: Arc<wgpu::Device>) -> Result<Self> {
        let mut spout = RustySpout::new();
        spout.get_spout().map_err(|_| IoError::SpoutInitFailed)?;
        let name = name.into();
        spout.set_sender_name(&name).map_err(|e| IoError::SpoutError(e.to_string()))?;
        spout.create_sender(&name, format.width, format.height, 0).map_err(|e| IoError::SpoutError(e.to_string()))?;
        Ok(Self {
            spout,
            name,
            format,
            frame_count: 0,
            device,
        })
    }
}

#[cfg(all(feature = "spout", target_os = "windows"))]
impl VideoSink for SpoutSender {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> VideoFormat {
        self.format.clone()
    }

    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        match &frame.data {
            FrameData::Cpu(data) => {
                // This is the CPU fallback. It's slow but safe.
                self.spout.send_image(
                    data.as_ptr(),
                    frame.format.width,
                    frame.format.height,
                    0x1908, // GL_RGBA
                    false,
                ).map_err(|e| IoError::SpoutError(e.to_string()))?;
                self.frame_count += 1;
                Ok(())
            }
            FrameData::Gpu(texture) => {
                // This is the GPU-accelerated path.
                // It uses an FFI call to the Spout library to update the sender with a shared texture handle.
                let handle = unsafe {
                    mapmap_render::spout::shared_handle_from_texture(&self.device, texture)
                }
                .map_err(|e| IoError::SpoutError(format!("Failed to get shared handle: {}", e)))?;

                let dxgi_format = match texture.format() {
                    wgpu::TextureFormat::Rgba8UnormSrgb => 28, // DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
                    wgpu::TextureFormat::Bgra8UnormSrgb => 91, // DXGI_FORMAT_B8G8R8A8_UNORM_SRGB
                    _ => return Err(IoError::SpoutError(format!("Unsupported texture format for Spout sending: {:?}", texture.format()))),
                };

                let spout_ffi = self.spout.get_spout().map_err(|e| IoError::SpoutError(e.to_string()))?;
                let c_name = CString::new(self.name.clone()).unwrap();

                let success = unsafe {
                    spout_ffi.UpdateSender(
                        c_name.as_ptr(),
                        frame.format.width as i32,
                        frame.format.height as i32,
                        handle.0 as _,
                        dxgi_format,
                    )
                };

                if success {
                    self.frame_count += 1;
                    Ok(())
                } else {
                    Err(IoError::SpoutError("Failed to update Spout sender with GPU texture".to_string()))
                }
            }
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

// Stub types for non-Windows platforms
/// Spout receiver (stub implementation when feature is disabled or on non-Windows platforms)
#[cfg(not(all(feature = "spout", target_os = "windows")))]
pub struct SpoutReceiver;

#[cfg(not(all(feature = "spout", target_os = "windows")))]
impl SpoutReceiver {
    /// Create a new Spout receiver (returns error when feature is disabled or on non-Windows platforms)
    pub fn new() -> crate::error::Result<Self> {
        #[cfg(not(target_os = "windows"))]
        return Err(crate::error::IoError::platform_not_supported(
            "Spout is only available on Windows",
        ));

        #[cfg(target_os = "windows")]
        Err(crate::error::IoError::feature_not_enabled("Spout", "spout"))
    }

    /// List available Spout senders (returns error when feature is disabled or on non-Windows platforms)
    pub fn list_senders() -> crate::error::Result<Vec<SpoutSenderInfo>> {
        #[cfg(not(target_os = "windows"))]
        return Err(crate::error::IoError::platform_not_supported(
            "Spout is only available on Windows",
        ));

        #[cfg(target_os = "windows")]
        Err(crate::error::IoError::feature_not_enabled("Spout", "spout"))
    }
}

/// Spout sender (stub implementation when feature is disabled or on non-Windows platforms)
#[cfg(not(all(feature = "spout", target_os = "windows")))]
pub struct SpoutSender;

#[cfg(not(all(feature = "spout", target_os = "windows")))]
impl SpoutSender {
    /// Create a new Spout sender (returns error when feature is disabled or on non-Windows platforms)
    pub fn new(
        _name: impl Into<String>,
        _format: crate::format::VideoFormat,
        _device: Arc<wgpu::Device>,
    ) -> crate::error::Result<Self> {
        #[cfg(not(target_os = "windows"))]
        return Err(crate::error::IoError::platform_not_supported(
            "Spout is only available on Windows",
        ));

        #[cfg(target_os = "windows")]
        Err(crate::error::IoError::feature_not_enabled("Spout", "spout"))
    }
}

/// Spout sender information
#[cfg(not(all(feature = "spout", target_os = "windows")))]
#[derive(Debug, Clone)]
pub struct SpoutSenderInfo {
    /// Sender name
    pub name: String,
    /// Video format
    pub format: crate::format::VideoFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spout_receiver_unavailable() {
        let result = SpoutReceiver::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_spout_sender_unavailable() {
        #[cfg(all(feature = "spout", target_os = "windows"))]
        {
            let format = VideoFormat::hd_1080p60_rgba();
            let result = SpoutSender::new("Test", format);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_spout_list_senders_unavailable() {
        let result = SpoutReceiver::list_senders();
        assert!(result.is_err());
    }
}
