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
pub mod image_decoder;
pub mod player;
pub mod sequence;
#[cfg(feature = "hap")]
pub mod hap_decoder;
// TODO: Enable pipeline with thread-local scaler approach
// The pipeline module requires VideoDecoder to be Send, but FFmpeg's scaler (SwsContext) is not thread-safe.
// Solution: Use thread-local scaler - create scaler once in decode thread, avoiding Send requirement.
// This provides zero overhead and clean separation. See pipeline.rs for implementation details.
// pub mod pipeline;

pub use decoder::{
    DecodedFrame, FFmpegDecoder, HwAccelType, PixelFormat, TestPatternDecoder, VideoDecoder,
};
pub use image_decoder::{GifDecoder, StillImageDecoder};
pub use player::{
    LoopMode, PlaybackCommand, PlaybackState, PlaybackStatus, PlayerError, VideoPlayer,
};
pub use sequence::ImageSequenceDecoder;
#[cfg(feature = "hap")]
pub use hap_decoder::{HapFrame, HapTextureType, HapError, decode_hap_frame};
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

/// Open a media file or image sequence and create a video player
///
/// This function auto-detects the media type from the path:
/// - If path is a directory, it's treated as an image sequence.
/// - If path has a GIF extension, `GifDecoder` is used.
/// - If path has a still image extension, `StillImageDecoder` is used.
/// - Otherwise, it's assumed to be a video file and opened with `FFmpegDecoder`.
pub fn open_path<P: AsRef<Path>>(path: P) -> Result<VideoPlayer> {
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
        _ => {
            // Default to FFmpeg for video files
            let ffmpeg_decoder = FFmpegDecoder::open(path)?;
            Box::new(ffmpeg_decoder)
        }
    };

    Ok(VideoPlayer::new_with_box(decoder))
}
