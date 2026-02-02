//! Image sequence decoder (directory of numbered frames)

use crate::{MediaError, Result, VideoDecoder};
use mapmap_io::VideoFrame;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Maximum number of frames to load in a sequence
///
/// This limit prevents memory exhaustion and long loading times when users
/// accidentally select a directory with thousands of files.
#[cfg(not(test))]
pub const MAX_SEQUENCE_FRAMES: usize = 5000;
#[cfg(test)]
pub const MAX_SEQUENCE_FRAMES: usize = 10;

/// Decoder for image sequences (directory of numbered frames)
///
/// Loads a directory of images and plays them back as a video at a specified FPS.
/// Supports common naming patterns: frame_001.png, img001.jpg, etc.
#[derive(Clone)]
pub struct ImageSequenceDecoder {
    frames: Vec<PathBuf>,
    width: u32,
    height: u32,
    current_frame: usize,
    fps: f64,
    duration: Duration,
    current_time: Duration,
    // Cache for the current frame to avoid re-loading
    cached_frame: Option<(usize, Vec<u8>)>,
}

impl ImageSequenceDecoder {
    /// Load an image sequence from a directory
    ///
    /// # Arguments
    /// * `directory` - Path to directory containing numbered images
    /// * `fps` - Frame rate for playback (e.g., 30.0)
    pub fn open<P: AsRef<Path>>(directory: P, fps: f64) -> Result<Self> {
        let directory = directory.as_ref();

        if !directory.exists() {
            return Err(MediaError::FileOpen(format!(
                "Directory not found: {}",
                directory.display()
            )));
        }

        if !directory.is_dir() {
            return Err(MediaError::FileOpen(format!(
                "Path is not a directory: {}",
                directory.display()
            )));
        }

        // Scan directory for image files
        let mut frames = Vec::new();

        for entry in WalkDir::new(directory)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if frames.len() >= MAX_SEQUENCE_FRAMES {
                warn!(
                    "Image sequence exceeded limit of {} frames. Truncating sequence from {}",
                    MAX_SEQUENCE_FRAMES,
                    directory.display()
                );
                break;
            }

            let path = entry.path();
            if path.is_file() && Self::is_supported_image(path) {
                frames.push(path.to_path_buf());
            }
        }

        if frames.is_empty() {
            return Err(MediaError::DecoderError(format!(
                "No image files found in directory: {}",
                directory.display()
            )));
        }

        // Sort frames by filename (natural sorting for numbered sequences)
        frames.sort();

        // Load first frame to get dimensions
        let first_image = image::open(&frames[0])
            .map_err(|e| MediaError::DecoderError(format!("Failed to load first frame: {}", e)))?;

        let width = first_image.width();
        let height = first_image.height();

        let duration = Duration::from_secs_f64(frames.len() as f64 / fps);

        info!(
            "Image sequence loaded: {}x{}, {} frames, {:.2}s duration @ {:.2} fps from {}",
            width,
            height,
            frames.len(),
            duration.as_secs_f64(),
            fps,
            directory.display()
        );

        Ok(Self {
            frames,
            width,
            height,
            current_frame: 0,
            fps,
            duration,
            current_time: Duration::ZERO,
            cached_frame: None,
        })
    }

    /// Check if a file is a supported image format
    pub fn is_supported_image(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp" | "webp"
            )
        } else {
            false
        }
    }

    /// Load and cache a frame
    fn load_frame(&mut self, index: usize) -> Result<Vec<u8>> {
        // Check cache
        if let Some((cached_idx, ref data)) = self.cached_frame {
            if cached_idx == index {
                return Ok(data.clone());
            }
        }

        // Load frame
        let path = &self.frames[index];
        let image = image::open(path)
            .map_err(|e| MediaError::DecoderError(format!("Failed to load frame: {}", e)))?;

        let rgba_image = image.to_rgba8();
        let frame_data = rgba_image.into_raw();

        // Update cache
        self.cached_frame = Some((index, frame_data.clone()));

        Ok(frame_data)
    }
}

impl VideoDecoder for ImageSequenceDecoder {
    fn next_frame(&mut self) -> Result<VideoFrame> {
        if self.current_frame >= self.frames.len() {
            return Err(MediaError::EndOfStream);
        }

        let frame_data = self.load_frame(self.current_frame)?;
        let pts = self.current_time;

        // Advance to next frame
        self.current_frame += 1;
        self.current_time += Duration::from_secs_f64(1.0 / self.fps);

        Ok(VideoFrame::new(
            frame_data,
            mapmap_io::VideoFormat {
                width: self.width,
                height: self.height,
                pixel_format: mapmap_io::PixelFormat::RGBA8,
                frame_rate: self.fps as f32,
            },
            pts,
        ))
    }

    fn seek(&mut self, timestamp: Duration) -> Result<()> {
        if timestamp > self.duration {
            return Err(MediaError::SeekError(
                "Timestamp beyond duration".to_string(),
            ));
        }

        let frame_index = (timestamp.as_secs_f64() * self.fps) as usize;
        self.current_frame = frame_index.min(self.frames.len() - 1);
        self.current_time = Duration::from_secs_f64(self.current_frame as f64 / self.fps);

        Ok(())
    }

    fn duration(&self) -> Duration {
        self.duration
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
    fn test_image_sequence_is_supported_image() {
        assert!(ImageSequenceDecoder::is_supported_image(Path::new(
            "frame001.png"
        )));
        assert!(ImageSequenceDecoder::is_supported_image(Path::new(
            "frame001.jpg"
        )));
        assert!(!ImageSequenceDecoder::is_supported_image(Path::new(
            "frame001.mp4"
        )));
    }

    #[test]
    fn test_sequence_limit() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // 1. Create one valid image (so loading succeeds)
        // We use the 'image' crate which is already a dependency
        let valid_img_path = dir_path.join("frame_0000.png");
        let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(1, 1);
        img.save(&valid_img_path).unwrap();

        // 2. Create excess empty files (enough to trigger limit)
        // Limit is 10 in test mode. We create 15 total (1 valid + 14 empty)
        // We use .png extension so they are picked up
        for i in 1..15 {
            let p = dir_path.join(format!("frame_{:04}.png", i));
            std::fs::write(&p, b"").unwrap();
        }

        // 3. Open decoder
        let decoder = ImageSequenceDecoder::open(dir_path, 30.0).unwrap();

        // 4. Verify limit
        assert_eq!(decoder.frames.len(), MAX_SEQUENCE_FRAMES);
        assert_eq!(decoder.frames.len(), 10);
    }
}
