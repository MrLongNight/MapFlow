# MapFlow Media

Video decoding and playback engine for MapFlow.

## Features

- **Video Decoding**: Hardware-accelerated decoding via FFmpeg.
- **HAP Codec**: Native support for the HAP codec family (HAP, HAP Alpha, HAP Q) for high-performance playback.
- **Image Sequences**: Playback of image sequences (folders of PNG/JPG/etc.) with high performance.
- **Animation Support**: Full GIF animation support including variable frame delays.
- **Playback Control**: Robust state machine for Play, Pause, Stop, Seek, Loop, and Speed control.

## Usage

```rust,no_run
use mapmap_media::{open_path, LoopMode};

// Open a video file (auto-detects format)
let mut player = open_path("content/video.mp4").unwrap();

// Start playback
player.play();
player.set_loop_mode(LoopMode::Loop);

// In the render loop:
if let Some(frame) = player.get_current_frame() {
    // Upload frame to GPU...
}
```

## Supported Formats

- **Video**: H.264, H.265 (HEVC), VP8, VP9, ProRes, HAP.
- **Images**: PNG, JPEG, BMP, TIFF, WebP, GIF.
