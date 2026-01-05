# mapmap-media

**mapmap-media** is the media playback engine of MapFlow. It handles video decoding, image loading, and playback state management.

## Key Features

*   **Video Decoding**: FFmpeg integration (via `ffmpeg-next`) for hardware-accelerated video playback.
*   **HAP Codec**: Native support for HAP/HAP-Q codecs using Snappy decompression.
*   **Image Support**: Loading of static images (PNG, JPG, etc.) and GIF animations.
*   **Image Sequences**: Playback of image folders as video sequences.
*   **Playback Control**: Robust state machine (`Idle`, `Playing`, `Paused`, etc.) with looping and seeking support.
*   **Pipeline**: Async pipeline for fetching frames and preparing them for the renderer.

## Usage

The media system runs asynchronously to ensure smooth playback without blocking the main render loop.

```rust
// let player = MediaPlayer::new();
// player.load("video.mp4");
// player.play();
```
