# mapmap-media

Media playback and decoding engine for MapFlow.

## Overview
This crate handles the loading, decoding, and playback of video and image files. It abstracts the complexity of codecs and container formats.

## Features

- **Video Decoding**:
  - Uses `ffmpeg-next` (optional feature) for broad format support.
  - Hardware acceleration support (where available).
  - Robust state machine for playback control (Play, Pause, Loop).

- **Image Support**:
  - Decodes standard formats (PNG, JPG, BMP) via the `image` crate.
  - Supports animated GIFs with correct frame timing.
  - Image Sequence playback for high-performance content.

- **HAP Codec**:
  - Specialized support for the HAP GPU-accelerated video codec.
  - Custom Snappy decompression and BC1/BC3 texture handling.
