//! MPV-based video decoder using libmpv2
//!
//! This module provides video decoding via the MPV library, which supports
//! virtually all video formats with hardware acceleration.

use crate::{MediaError, Result, VideoDecoder};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info, warn};
use vorce_io::format::{PixelFormat, VideoFormat};
use vorce_io::VideoFrame;

use libmpv2::Mpv;

/// MPV-based video decoder
///
/// Uses libmpv2 for video decoding.
/// Uses screenshot-raw command for frame extraction to maintain thread safety and compatibility.
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

        // Configure MPV for offscreen operation
        // `image` allows the screenshot-raw command to capture frames without creating a window.
        // `null` suppresses video entirely and prevents `screenshot-raw` from returning frames.
        mpv.set_property("vo", "image").ok();
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
        std::thread::sleep(std::time::Duration::from_millis(200));

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

        let mut extracted_data = Vec::new();
        let mut actual_width = self.width;
        let mut actual_height = self.height;

        unsafe {
            use libmpv2_sys::*;
            let handle = self.mpv.ctx;

            let cmd_sc = std::ffi::CString::new("screenshot-raw").unwrap();
            let cmd_sc_arg = std::ffi::CString::new("video").unwrap();

            let mut cmd_screenshot = [cmd_sc.as_ptr(), cmd_sc_arg.as_ptr(), std::ptr::null()];

            let mut node = mpv_node {
                format: 0,
                u: mpv_node__bindgen_ty_1 {
                    string: std::ptr::null_mut(),
                },
            };
            let res = mpv_command_ret(
                handle.as_ptr(),
                cmd_screenshot.as_mut_ptr(),
                &mut node as *mut _,
            );

            if res >= 0 && node.format == mpv_format_MPV_FORMAT_NODE_MAP {
                let map = node.u.list;
                let keys = std::slice::from_raw_parts((*map).keys, (*map).num as usize);
                let vals = std::slice::from_raw_parts((*map).values, (*map).num as usize);

                for i in 0..(*map).num as usize {
                    let key = std::ffi::CStr::from_ptr(keys[i]).to_str().unwrap_or("");
                    if key == "data" && vals[i].format == mpv_format_MPV_FORMAT_BYTE_ARRAY {
                        let ba = vals[i].u.ba;
                        let data_slice = std::slice::from_raw_parts(
                            (*ba).data as *const u8,
                            (*ba).size as usize,
                        );
                        extracted_data.extend_from_slice(data_slice);
                    } else if key == "w" && vals[i].format == mpv_format_MPV_FORMAT_INT64 {
                        actual_width = vals[i].u.int64 as u32;
                    } else if key == "h" && vals[i].format == mpv_format_MPV_FORMAT_INT64 {
                        actual_height = vals[i].u.int64 as u32;
                    }
                }
            } else {
                error!(
                    "MPV frame capture failed. Return code: {}, Node format: {}",
                    res, node.format
                );
                return Err(MediaError::DecoderError(format!(
                    "MPV screenshot-raw failed. Error: {}",
                    res
                )));
            }

            mpv_free_node_contents(&mut node);
        }

        if extracted_data.is_empty() {
            return Err(MediaError::DecoderError(
                "No data returned from screenshot-raw".to_string(),
            ));
        }

        // Validate data size (BGRA or RGBA expected: width * height * 4)
        if extracted_data.len() < (actual_width * actual_height * 4) as usize {
            warn!(
                "Captured frame data too small. Expected {} bytes, got {}",
                actual_width * actual_height * 4,
                extracted_data.len()
            );
            return Err(MediaError::DecoderError(
                "Captured frame data too small".to_string(),
            ));
        }

        // screenshot-raw typically returns BGRA or BGR0 format when no format is forced.
        // We will swap R and B channels to output standard RGBA
        // libmpv usually outputs BGRA layout on most platforms for `screenshot-raw`
        let mut final_data = extracted_data;
        for chunk in final_data.chunks_exact_mut(4) {
            let b = chunk[0];
            chunk[0] = chunk[2];
            chunk[2] = b;
            chunk[3] = 255; // Ensure alpha is fully opaque
        }

        // Create video format using the actual dimensions returned by the screenshot
        let format = VideoFormat::new(
            actual_width,
            actual_height,
            PixelFormat::RGBA8,
            self.fps as f32,
        );

        Ok(VideoFrame::new(final_data, format, self.current_time))
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
