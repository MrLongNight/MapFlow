//! Image decoder implementations for still images and GIFs
//!
//! Phase 1 feature implementation:
//! - Still images: PNG, JPEG, TIFF via `image` crate
//! - Animated GIF: Frame-by-frame playback with timing

use crate::{MediaError, Result, VideoDecoder};
use image::{AnimationDecoder, DynamicImage};
use mapmap_io::{PixelFormat, VideoFrame};
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

/// Maximum number of frames to load for a GIF
///
/// This limit prevents memory exhaustion when users load large animated GIFs.
#[cfg(not(test))]
pub const MAX_GIF_FRAMES: usize = 500;
#[cfg(test)]
pub const MAX_GIF_FRAMES: usize = 10;

// ============================================================================
// Still Image Decoder
// ============================================================================

/// Decoder for still images (PNG, JPEG, TIFF, etc.)
///
/// Still images are treated as single-frame "videos" with infinite duration.
/// Seeking has no effect, and next_frame() always returns the same image.
#[derive(Clone)]
pub struct StillImageDecoder {
    width: u32,
    height: u32,
    frame_data: Vec<u8>,
    has_been_read: bool,
}

impl StillImageDecoder {
    /// Load a still image from a file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MediaError::FileOpen(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Load image using the `image` crate
        let image = image::open(path)
            .map_err(|e| MediaError::DecoderError(format!("Failed to load image: {}", e)))?;

        let width = image.width();
        let height = image.height();

        // Convert to RGBA8
        let rgba_image = image.to_rgba8();
        let frame_data = rgba_image.into_raw();

        info!(
            "Still image loaded: {}x{} from {}",
            width,
            height,
            path.display()
        );

        Ok(Self {
            width,
            height,
            frame_data,
            has_been_read: false,
        })
    }

    /// Check if the file format is supported
    pub fn supports_format<P: AsRef<Path>>(path: P) -> bool {
        if let Some(ext) = path.as_ref().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp" | "webp"
            )
        } else {
            false
        }
    }
}

impl VideoDecoder for StillImageDecoder {
    fn next_frame(&mut self) -> Result<VideoFrame> {
        // Still images can be read repeatedly
        // For proper "video" behavior, we only return the frame once
        // and then return EndOfStream to match video semantics
        if self.has_been_read {
            return Err(MediaError::EndOfStream);
        }

        self.has_been_read = true;

        Ok(VideoFrame::new(
            self.frame_data.clone(),
            mapmap_io::VideoFormat {
                width: self.width,
                height: self.height,
                pixel_format: PixelFormat::RGBA8,
                frame_rate: 1.0,
            },
            Duration::ZERO,
        ))
    }

    fn seek(&mut self, _timestamp: Duration) -> Result<()> {
        // Seeking in a still image resets to beginning
        self.has_been_read = false;
        Ok(())
    }

    fn duration(&self) -> Duration {
        // Still images have "infinite" duration represented as a very long time
        Duration::from_secs(3600 * 24 * 365) // 1 year
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn fps(&self) -> f64 {
        // Still images don't have FPS, but we return 1 for consistency
        1.0
    }

    fn clone_decoder(&self) -> Result<Box<dyn VideoDecoder>> {
        Ok(Box::new(self.clone()))
    }
}

// ============================================================================
// GIF Decoder
// ============================================================================

/// Decoder for animated GIF files
///
/// Supports frame-by-frame playback with proper timing based on GIF delays.
#[derive(Clone)]
pub struct GifDecoder {
    frames: Vec<(Vec<u8>, Duration)>, // (frame_data, delay)
    width: u32,
    height: u32,
    current_frame: usize,
    current_time: Duration,
    total_duration: Duration,
    fps: f64,
}

impl GifDecoder {
    /// Load an animated GIF from a file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MediaError::FileOpen(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Open the file
        let file = std::fs::File::open(path)
            .map_err(|e| MediaError::FileOpen(format!("Failed to open file: {}", e)))?;

        let decoder = image::codecs::gif::GifDecoder::new(file)
            .map_err(|e| MediaError::DecoderError(format!("Failed to decode GIF: {}", e)))?;

        // Extract frames
        let frames_iter = decoder.into_frames();

        let mut frames = Vec::new();
        let mut total_duration = Duration::ZERO;
        let mut width = 0;
        let mut height = 0;

        for frame_result in frames_iter {
            if frames.len() >= MAX_GIF_FRAMES {
                warn!(
                    "GIF exceeded limit of {} frames. Truncating sequence from {}",
                    MAX_GIF_FRAMES,
                    path.display()
                );
                break;
            }

            let frame = frame_result
                .map_err(|e| MediaError::DecoderError(format!("Failed to decode frame: {}", e)))?;

            let delay = frame.delay();
            let delay_duration = Duration::from_millis(
                (delay.numer_denom_ms().0 as f64 / delay.numer_denom_ms().1 as f64 * 1000.0) as u64,
            );

            let image = DynamicImage::ImageRgba8(frame.into_buffer());
            if width == 0 {
                width = image.width();
                height = image.height();
            }

            frames.push((image.to_rgba8().into_raw(), delay_duration));
            total_duration += delay_duration;
        }

        if frames.is_empty() {
            return Err(MediaError::DecoderError("GIF has no frames".to_string()));
        }

        let fps = frames.len() as f64 / total_duration.as_secs_f64();

        info!(
            "GIF loaded: {}x{}, {} frames, {:.2}s duration, {:.2} fps from {}",
            width,
            height,
            frames.len(),
            total_duration.as_secs_f64(),
            fps,
            path.display()
        );

        Ok(Self {
            frames,
            width,
            height,
            current_frame: 0,
            current_time: Duration::ZERO,
            total_duration,
            fps,
        })
    }

    /// Check if the file is a GIF
    pub fn supports_format<P: AsRef<Path>>(path: P) -> bool {
        if let Some(ext) = path.as_ref().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "gif"
        } else {
            false
        }
    }
}

impl VideoDecoder for GifDecoder {
    fn next_frame(&mut self) -> Result<VideoFrame> {
        if self.current_time >= self.total_duration {
            return Err(MediaError::EndOfStream);
        }

        let (frame_data, delay) = self.frames[self.current_frame].clone();

        let pts = self.current_time;

        // Advance to next frame
        self.current_time += delay;
        self.current_frame = (self.current_frame + 1) % self.frames.len();

        Ok(VideoFrame::new(
            frame_data,
            mapmap_io::VideoFormat {
                width: self.width,
                height: self.height,
                pixel_format: PixelFormat::RGBA8,
                frame_rate: self.fps as f32,
            },
            pts,
        ))
    }

    fn seek(&mut self, timestamp: Duration) -> Result<()> {
        if timestamp > self.total_duration {
            return Err(MediaError::SeekError(
                "Timestamp beyond duration".to_string(),
            ));
        }

        // Find the frame at the given timestamp
        let mut accumulated_time = Duration::ZERO;
        for (idx, (_image, delay)) in self.frames.iter().enumerate() {
            if accumulated_time + *delay > timestamp {
                self.current_frame = idx;
                self.current_time = accumulated_time;
                return Ok(());
            }
            accumulated_time += *delay;
        }

        // If we get here, seek to last frame
        self.current_frame = self.frames.len() - 1;
        self.current_time = self.total_duration;
        Ok(())
    }

    fn duration(&self) -> Duration {
        self.total_duration
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn fps(&self) -> f64 {
        self.fps
    }

    fn clone_decoder(&self) -> Result<Box<dyn VideoDecoder>> {
        Ok(Box::new(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_still_image_supports_format() {
        assert!(StillImageDecoder::supports_format("test.png"));
        assert!(StillImageDecoder::supports_format("test.jpg"));
        assert!(StillImageDecoder::supports_format("test.jpeg"));
        assert!(StillImageDecoder::supports_format("test.tif"));
        assert!(StillImageDecoder::supports_format("test.tiff"));
        assert!(!StillImageDecoder::supports_format("test.mp4"));
        assert!(!StillImageDecoder::supports_format("test.txt"));
    }

    #[test]
    fn test_gif_supports_format() {
        assert!(GifDecoder::supports_format("test.gif"));
        assert!(!GifDecoder::supports_format("test.png"));
        assert!(!GifDecoder::supports_format("test.jpg"));
    }

    #[test]
    fn test_still_image_decoder_new_not_found() {
        let result = StillImageDecoder::open("a_file_that_does_not_exist.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_gif_decoder_new_not_found() {
        let result = GifDecoder::open("a_file_that_does_not_exist.gif");
        assert!(result.is_err());
    }

    #[test]
    fn test_gif_frame_limit() {
        use image::codecs::gif::{GifEncoder, Repeat};
        use image::{Delay, Frame, RgbaImage};
        use std::fs::File;

        // Create a temp file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_limit.gif");
        let file = File::create(&file_path).unwrap();

        // Create 15 frames
        let mut encoder = GifEncoder::new(file);
        encoder.set_repeat(Repeat::Infinite).unwrap();

        for _ in 0..15 {
            let frame = Frame::from_parts(
                RgbaImage::new(10, 10),
                0,
                0,
                Delay::from_numer_denom_ms(100, 1),
            );
            encoder.encode_frame(frame).unwrap();
        }

        // Drop encoder to flush to file
        drop(encoder);

        // Open decoder
        let decoder = GifDecoder::open(&file_path).unwrap();

        // Expect 10 frames (capped by MAX_GIF_FRAMES)
        assert_eq!(decoder.frames.len(), MAX_GIF_FRAMES);
    }
}
