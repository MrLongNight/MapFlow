//! HAP Video Player
//!
//! Integrates HAP decoding with GPU texture upload for high-performance playback.

use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::hap_decoder::{decode_hap_frame, HapError, HapFrame, HapTextureType};
use crate::player::{LoopMode, PlaybackState, PlayerError, VideoDecoder};
use crate::MediaError;

/// HAP-specific decoder that produces GPU-ready DXT textures
pub struct HapVideoDecoder {
    /// Video width
    width: u32,
    /// Video height
    height: u32,
    /// Frame rate
    fps: f64,
    /// Total frame count
    frame_count: usize,
    /// Current frame index
    current_frame: usize,
    /// Raw HAP frame data (from container)
    frames: Vec<Vec<u8>>,
    /// HAP texture type (determined from first frame)
    texture_type: Option<HapTextureType>,
}

impl HapVideoDecoder {
    /// Open a HAP video file
    ///
    /// Note: This requires FFmpeg to extract frames from the MOV container.
    /// For now, this is a placeholder that needs FFmpeg integration.
    #[cfg(feature = "ffmpeg")]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MediaError> {
        use ffmpeg_next as ffmpeg;

        let path = path.as_ref();
        info!("Opening HAP video: {:?}", path);

        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| MediaError::DecoderError(e.to_string()))?;

        // Open input file
        let ictx = ffmpeg::format::input(&path).map_err(|e| MediaError::FileOpen(e.to_string()))?;

        // Find video stream
        let video_stream = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(MediaError::NoVideoStream)?;

        let video_stream_index = video_stream.index();

        // Get video parameters
        let codec_par = video_stream.parameters();
        let width = unsafe { (*codec_par.as_ptr()).width as u32 };
        let height = unsafe { (*codec_par.as_ptr()).height as u32 };

        // Get frame rate
        let fps = video_stream.avg_frame_rate();
        let fps = if fps.denominator() != 0 {
            fps.numerator() as f64 / fps.denominator() as f64
        } else {
            30.0
        };

        info!("HAP video: {}x{} @ {:.2} fps", width, height, fps);

        // Check codec - HAP has codec_id FOURCC 'Hap1', 'Hap5', 'HapY', etc.
        let codec_name = unsafe {
            let codec_id = (*codec_par.as_ptr()).codec_id;
            std::ffi::CStr::from_ptr(ffmpeg::ffi::avcodec_get_name(codec_id))
                .to_string_lossy()
                .to_string()
        };

        if !codec_name.to_lowercase().contains("hap") {
            warn!("Video may not be HAP encoded: codec = {}", codec_name);
        }

        // Extract raw HAP frames (no decoding - HAP is stored as-is)
        let mut frames = Vec::new();

        // Re-open for packet reading
        let mut ictx =
            ffmpeg::format::input(&path).map_err(|e| MediaError::FileOpen(e.to_string()))?;

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                // HAP frames are stored as raw data in packets
                let data = packet.data().unwrap_or(&[]).to_vec();
                frames.push(data);
            }
        }

        let frame_count = frames.len();
        info!("Loaded {} HAP frames", frame_count);

        // Determine texture type from first frame
        let texture_type = if let Some(first_frame) = frames.first() {
            match decode_hap_frame(first_frame, width, height) {
                Ok(frame) => Some(frame.texture_type),
                Err(e) => {
                    error!("Failed to decode first HAP frame: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            width,
            height,
            fps,
            frame_count,
            current_frame: 0,
            frames,
            texture_type,
        })
    }

    /// Decode current frame to GPU-ready DXT data
    pub fn decode_current_frame(&self) -> Result<HapFrame, HapError> {
        if self.current_frame >= self.frames.len() {
            return Err(HapError::InvalidHeader);
        }

        let raw_data = &self.frames[self.current_frame];
        decode_hap_frame(raw_data, self.width, self.height)
    }

    /// Get texture type (BC1 or BC3)
    pub fn texture_type(&self) -> Option<HapTextureType> {
        self.texture_type
    }

    /// Check if this is a HAP Q video (needs YCoCg conversion)
    pub fn needs_ycocg_conversion(&self) -> bool {
        self.texture_type
            .map(|t| t.needs_ycocg_conversion())
            .unwrap_or(false)
    }
}

/// Placeholder for non-FFmpeg builds
#[cfg(not(feature = "ffmpeg"))]
impl HapVideoDecoder {
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self, MediaError> {
        Err(MediaError::DecoderError(
            "HAP decoding requires FFmpeg feature".to_string(),
        ))
    }
}

impl VideoDecoder for HapVideoDecoder {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn fps(&self) -> f64 {
        self.fps
    }

    fn duration(&self) -> std::time::Duration {
        let seconds = self.frame_count as f64 / self.fps;
        std::time::Duration::from_secs_f64(seconds)
    }

    fn frame_count(&self) -> usize {
        self.frame_count
    }

    fn decode_frame(&mut self) -> Result<crate::DecodedFrame, MediaError> {
        // For HAP, we return the raw DXT data instead of decoded RGBA
        // The GPU will handle the final decompression

        if self.current_frame >= self.frame_count {
            return Err(MediaError::EndOfStream);
        }

        let hap_frame = self
            .decode_current_frame()
            .map_err(|e| MediaError::DecoderError(e.to_string()))?;

        // Note: This returns DXT-compressed data, not RGBA!
        // The caller must use compressed texture upload
        let frame = crate::decoder::DecodedFrame {
            width: self.width,
            height: self.height,
            data: hap_frame.texture_data,
            timestamp: std::time::Duration::from_secs_f64(self.current_frame as f64 / self.fps),
            // Mark as compressed format (caller should check and use BC upload)
            format: crate::decoder::PixelFormat::Unknown,
        };

        self.current_frame += 1;

        Ok(frame)
    }

    fn seek(&mut self, position: std::time::Duration) -> Result<(), MediaError> {
        let target_frame = (position.as_secs_f64() * self.fps) as usize;
        self.current_frame = target_frame.min(self.frame_count.saturating_sub(1));
        debug!("HAP seek to frame {}", self.current_frame);
        Ok(())
    }

    fn seek_frame(&mut self, frame: usize) -> Result<(), MediaError> {
        self.current_frame = frame.min(self.frame_count.saturating_sub(1));
        Ok(())
    }
}

/// Check if a file is likely a HAP video
pub fn is_hap_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // HAP is typically in MOV container
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Could be MOV or AVI
    if ext != "mov" && ext != "avi" {
        return false;
    }

    // TODO: Actually probe the file to check codec
    // For now, we can't determine without opening

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hap_file() {
        assert!(is_hap_file("video.mov"));
        assert!(is_hap_file("VIDEO.MOV"));
        assert!(is_hap_file("test.avi"));
        assert!(!is_hap_file("video.mp4"));
        assert!(!is_hap_file("video.webm"));
    }
}




