# MapFlow I/O

Professional video input/output subsystem for MapFlow. This crate manages video sources (capture cards, network streams) and sinks (projectors, streams).

## Features

- **NDI**: Network Device Interface support for high-quality, low-latency IP video.
- **DeckLink**: Support for Blackmagic Design capture and playback cards.
- **Spout**: Real-time texture sharing on Windows.
- **Syphon**: Real-time texture sharing on macOS.
- **Streaming**: RTMP/SRT streaming capabilities.
- **Virtual Camera**: Output MapFlow content as a virtual webcam.

## Architecture

The I/O system is built around the `VideoSource` and `VideoSink` traits, allowing for a unified pipeline regardless of the underlying technology.

- `VideoFrame`: A unified container for pixel data, timestamps, and metadata.
- `FormatConverter`: Efficiently converts between different pixel formats (e.g., BGRA to RGBA).

## Platform Support

| Feature        | Windows | macOS | Linux |
|----------------|:-------:|:-----:|:-----:|
| NDI            |    ✓    |   ✓   |   ✓   |
| DeckLink       |    ✓    |   ✓   |   ✓   |
| Spout          |    ✓    |   -   |   -   |
| Syphon         |    -    |   ✓   |   -   |
| Streaming      |    ✓    |   ✓   |   ✓   |
| Virtual Camera |    ✓    |   ✓   |   ✓   |

## Feature Flags

- `ndi`: Enable NDI support (requires NDI SDK).
- `decklink`: Enable DeckLink support.
- `spout`: Enable Spout support (Windows only).
- `syphon`: Enable Syphon support (macOS only).
- `stream`: Enable streaming support.
- `virtual-camera`: Enable virtual camera output.
