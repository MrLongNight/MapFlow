//! MapFlow Media - Video Decoding and Playback
//!
//! This crate provides video decoding capabilities via FFmpeg, including:
//! - Video decoder abstraction
//! - Playback control (seek, speed, loop)
//!
//! Multi-threaded decoding pipeline is planned for a future phase.

use std::path::Path;
use thiserror::Error;

pub mod decoder;
#[cfg(feature = "hap")]
pub mod hap_decoder;
#[cfg(feature = "hap")]
pub mod hap_player;
pub mod image_decoder;
pub mod player;
pub mod sequence;
// TODO: Enable pipeline with thread-local scaler approach
// The pipeline module requires VideoDecoder to be Send, but FFmpeg's scaler (SwsContext) is not thread-safe.
// Solution: Use thread-local scaler - create scaler once in decode thread, avoiding Send requirement.
// This provides zero overhead and clean separation. See pipeline.rs for implementation details.
// pub mod pipeline;

pub use decoder::{FFmpegDecoder, HwAccelType, PixelFormat, TestPatternDecoder, VideoDecoder};
#[cfg(feature = "hap")]
pub use hap_decoder::{decode_hap_frame, HapError, HapFrame, HapTextureType};
#[cfg(feature = "hap")]
pub use hap_player::{is_hap_file, HapVideoDecoder};
pub use image_decoder::{GifDecoder, StillImageDecoder};
pub use player::{
    LoopMode, PlaybackCommand, PlaybackState, PlaybackStatus, PlayerError, VideoPlayer,
};
pub use sequence::ImageSequenceDecoder;
// pub use pipeline::{FramePipeline, PipelineConfig, PipelineStats, Priority, FrameScheduler};

/// Media errors
#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Failed to open file: {0}")]
    FileOpen(String),

    #[error("No video stream found")]
    NoVideoStream,

    #[error("Decoder error: {0}")]
    DecoderError(String),

    #[error("End of stream")]
    EndOfStream,

    #[error("Seek error: {0}")]
    SeekError(String),
}

/// Result type for media operations
pub type Result<T> = std::result::Result<T, MediaError>;

/// Options for opening media
#[derive(Debug, Clone, Default)]
pub struct MediaOpenOptions {
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
    pub target_fps: Option<f32>,
}

/// Open a media file or image sequence and create a video player
///
/// This function auto-detects the media type from the path:
/// - If path is a directory, it's treated as an image sequence.
/// - If path has a GIF extension, `GifDecoder` is used.
/// - If path has a still image extension, `StillImageDecoder` is used.
/// - If HAP feature is enabled and file might be HAP, try HAP decoder first.
/// - Otherwise, it's assumed to be a video file and opened with `FFmpegDecoder`.
pub fn open_path<P: AsRef<Path>>(path: P, options: MediaOpenOptions) -> Result<VideoPlayer> {
    let path = path.as_ref();

    // Check if it's an image sequence (directory)
    if path.is_dir() {
        let decoder = ImageSequenceDecoder::open(path, 30.0)?; // Default to 30 fps
        return Ok(VideoPlayer::new(decoder));
    }

    // Check file extension for still images and GIFs
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let decoder: Box<dyn VideoDecoder> = match ext.as_str() {
        "gif" => Box::new(GifDecoder::open(path)?),
        "png" | "jpg" | "jpeg" | "tif" | "tiff" | "bmp" | "webp" => {
            Box::new(StillImageDecoder::open(path)?)
        }
        // HAP videos are typically in MOV containers
        #[cfg(feature = "hap")]
        "mov" => {
            // Try HAP first, fall back to FFmpeg if it fails
            match HapVideoDecoder::open(path) {
                Ok(hap_decoder) => {
                    tracing::info!("Opened as HAP video: {:?}", path);
                    Box::new(hap_decoder)
                }
                Err(_) => {
                    tracing::debug!("Not a HAP file, falling back to FFmpeg: {:?}", path);
                    Box::new(FFmpegDecoder::open(path, options)?)
                }
            }
        }
        _ => {
            // Default to FFmpeg for video files
            let ffmpeg_decoder = FFmpegDecoder::open(path, options)?;
            Box::new(ffmpeg_decoder)
        }
    };

    Ok(VideoPlayer::new_with_box(decoder))
}
