# MapFlow Media (`mapmap-media`)

Media playback pipeline for video, images, and image sequences.

## Features

- **Video**: Hardware-accelerated video decoding via `ffmpeg-next`.
- **Images**: Support for PNG, JPG, BMP, etc.
- **Image Sequences**: Efficient playback of folder-based image sequences.
- **HAP Codec**: Native support for HAP playback for high performance.
- **Player**: State machine for robust playback control (Play, Pause, Loop, Seek).

## Usage

```rust
use mapmap_media::player::MediaPlayer;

let mut player = MediaPlayer::new();
player.load("video.mp4")?;
player.play();
```
