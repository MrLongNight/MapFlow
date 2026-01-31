# Task: Implement Hardware Acceleration for Video Playback

## Objective
Enable hardware-accelerated video decoding (NVDEC, DXVA2, QSV) to reduce CPU usage and improve playback performance for high-resolution files.

## Context
Currenly `mapmap_media` uses software decoding via `ffmpeg-next`. This causes performance bottlenecks with 4K content or multiple HD streams.

## Implementation Steps

### 1. Dependency Configuration
- Verify `ffmpeg-next` features in `crates/mapmap-media/Cargo.toml`.
- Ensure `vcpkg` or system FFmpeg installation supports hardware codecs (`h264_nvenc`, `hevc_nvenc`, `dxva2`, etc.).

### 2. Decoder Initialization (`crates/mapmap-media/src/decoder/ffmpeg_impl.rs`)
- Modify `VideoDecoder::new` or similar initialization logic.
- Use `ffmpeg::codec::context::Context::set_hw_device_ctx`.
- Iterate available HW configs (`ffmpeg::util::media::Type::Video`) and select best available (DXVA2/D3D11VA on Windows).

### 3. Frame Handling
- Hardware frames (`AV_PIX_FMT_D3D11`, `NV12`, etc.) function differently than software `RGBA`.
- Implement a fallback transfer step (`av_hwframe_transfer_data`) to download frames to CPU memory if direct texture sharing is not yet implemented (Phase 1 of HW accel).
- **Advanced (Phase 2):** Zero-copy logic. Pass D3D11 texture pointer directly to `wgpu` via specific platform extensions (requires unsafe `wgpu` interop). *Start with copy-back first for stability.*

### 4. Verification
- Add log output verifying `hw_accel` is active (e.g., "Initialized decoder with dxva2").
- Compare CPU usage before/after with 4K video.

## Constraints
- Focus on Windows (`dxva2`, `d3d11va`) first.
- Ensure fallback to software decoding if HW init fails.
