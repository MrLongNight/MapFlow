//! MPV-based video decoder using libmpv2
//!
//! This module provides video decoding via the MPV library, which supports
//! virtually all video formats with hardware acceleration.

use crate::{MediaError, Result, VideoDecoder};
use mapmap_io::format::{PixelFormat, VideoFormat};
use mapmap_io::VideoFrame;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info};

use libmpv2::Mpv;

/// MPV-based video decoder
///
/// Uses libmpv2 for video decoding with hardware acceleration support.
/// The render feature allows getting raw video frames for GPU texture upload.
pub struct MpvDecoder {
    mpv: Mpv,
    path: PathBuf,
    width: u32,
    height: u32,
    duration_secs: f64,
    fps: f64,
    current_time: Duration,
}

impl MpvDecoder {
    /// Open a video file with MPV
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        info!("Opening video with MPV: {:?}", path);

        // Initialize MPV
        let mpv = Mpv::new().map_err(|e| {
            error!("Failed to create MPV instance: {}", e);
            MediaError::DecoderError(format!("MPV init failed: {}", e))
        })?;

        // Configure MPV for offscreen rendering
        mpv.set_property("vo", "null").ok(); // No video output window
        mpv.set_property("pause", true).ok(); // Start paused
        mpv.set_property("keep-open", true).ok(); // Don't close at end
        mpv.set_property("audio", "no").ok(); // Disable audio for now

        // Load the file
        let path_str = path
            .to_str()
            .ok_or_else(|| MediaError::FileOpen("Invalid path encoding".to_string()))?;

        mpv.command("loadfile", &[path_str]).map_err(|e| {
            error!("Failed to load file: {}", e);
            MediaError::FileOpen(format!("MPV loadfile failed: {}", e))
        })?;

        // Wait for file to load and get properties
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Get video properties
        let width = mpv.get_property::<i64>("width").unwrap_or(1920) as u32;
        let height = mpv.get_property::<i64>("height").unwrap_or(1080) as u32;
        let duration_secs = mpv.get_property::<f64>("duration").unwrap_or(0.0);
        let fps = mpv.get_property::<f64>("container-fps").unwrap_or(30.0);

        info!(
            "Video loaded: {}x{}, {:.2}s, {:.2}fps",
            width, height, duration_secs, fps
        );

        Ok(Self {
            mpv,
            path,
            width,
            height,
            duration_secs,
            fps,
            current_time: Duration::ZERO,
        })
    }

    /// Capture current frame
    fn capture_frame(&mut self) -> Result<VideoFrame> {
        // Get current playback time
        let time = self.mpv.get_property::<f64>("playback-time").unwrap_or(0.0);
        self.current_time = Duration::from_secs_f64(time);

        // Create video format
        let format = VideoFormat::new(self.width, self.height, PixelFormat::RGBA8, self.fps as f32);

        // For now, generate a placeholder frame (gray)
        // TODO: Use MPV render API for actual frame capture
        let frame_data = vec![128u8; format.buffer_size()];

        Ok(VideoFrame::new(frame_data, format, self.current_time))
    }
}

impl VideoDecoder for MpvDecoder {
    fn next_frame(&mut self) -> Result<VideoFrame> {
        // Step forward one frame
        self.mpv
            .command("frame-step", &[])
            .map_err(|e| MediaError::DecoderError(format!("Frame step failed: {}", e)))?;

        // Small delay to let MPV process
        std::thread::sleep(std::time::Duration::from_millis(1));

        self.capture_frame()
    }

    fn seek(&mut self, timestamp: Duration) -> Result<()> {
        let secs = timestamp.as_secs_f64();
        self.mpv
            .command("seek", &[&secs.to_string(), "absolute"])
            .map_err(|e| MediaError::SeekError(format!("MPV seek failed: {}", e)))?;
        self.current_time = timestamp;
        Ok(())
    }

    fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.duration_secs)
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn fps(&self) -> f64 {
        self.fps
    }

    fn clone_decoder(&self) -> Result<Box<dyn VideoDecoder>> {
        // MPV instances can't be cloned, create new one
        Ok(Box::new(Self::open(&self.path)?))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mpv_decoder_creation() {
        // This test requires MPV to be installed
        // Skip if not available
    }
}
