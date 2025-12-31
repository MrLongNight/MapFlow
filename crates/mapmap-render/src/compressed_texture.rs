//! DXT/BCn Compressed Texture Upload
//!
//! This module provides GPU upload functionality for DXT-compressed textures,
//! primarily used for HAP video codec support.
//!
//! Supported formats:
//! - BC1 (DXT1) - HAP RGB
//! - BC3 (DXT5) - HAP Alpha, HAP Q

use std::sync::Arc;
use tracing::debug;

/// DXT block size constants
pub const BC1_BLOCK_SIZE: u32 = 8; // 8 bytes per 4x4 block
pub const BC3_BLOCK_SIZE: u32 = 16; // 16 bytes per 4x4 block
pub const BLOCK_DIMENSION: u32 = 4; // 4x4 pixel blocks

/// DXT texture format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DxtFormat {
    /// BC1/DXT1 - RGB only, 4 bits per pixel
    Bc1,
    /// BC3/DXT5 - RGBA, 8 bits per pixel
    Bc3,
}

impl DxtFormat {
    /// Get the wgpu texture format
    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Bc1 => wgpu::TextureFormat::Bc1RgbaUnorm,
            Self::Bc3 => wgpu::TextureFormat::Bc3RgbaUnorm,
        }
    }

    /// Get bytes per 4x4 block
    pub fn block_size(&self) -> u32 {
        match self {
            Self::Bc1 => BC1_BLOCK_SIZE,
            Self::Bc3 => BC3_BLOCK_SIZE,
        }
    }

    /// Calculate expected data size for given dimensions
    pub fn calculate_size(&self, width: u32, height: u32) -> usize {
        let blocks_x = (width + 3) / 4;
        let blocks_y = (height + 3) / 4;
        (blocks_x * blocks_y * self.block_size()) as usize
    }
}

/// Create a compressed texture with BC1/BC3 format
pub fn create_compressed_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: DxtFormat,
    label: Option<&str>,
) -> wgpu::Texture {
    // Ensure dimensions are multiples of 4 (required for BCn)
    let aligned_width = (width + 3) & !3;
    let aligned_height = (height + 3) & !3;

    debug!(
        "Creating compressed texture: {}x{} (aligned: {}x{}), format: {:?}",
        width, height, aligned_width, aligned_height, format
    );

    device.create_texture(&wgpu::TextureDescriptor {
        label,
        size: wgpu::Extent3d {
            width: aligned_width,
            height: aligned_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: format.wgpu_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// Upload DXT-compressed data to a texture
pub fn upload_compressed_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    data: &[u8],
    width: u32,
    height: u32,
    format: DxtFormat,
) {
    // Calculate row pitch (bytes per row of blocks)
    let blocks_per_row = (width + 3) / 4;
    let bytes_per_row = blocks_per_row * format.block_size();

    // Calculate aligned dimensions
    let aligned_width = (width + 3) & !3;
    let aligned_height = (height + 3) & !3;

    debug!(
        "Uploading compressed texture: {}x{}, {} bytes, bytes_per_row: {}",
        aligned_width,
        aligned_height,
        data.len(),
        bytes_per_row
    );

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some((aligned_height + 3) / 4), // Number of block rows
        },
        wgpu::Extent3d {
            width: aligned_width,
            height: aligned_height,
            depth_or_array_layers: 1,
        },
    );
}

/// Handle for a compressed texture
#[derive(Clone)]
pub struct CompressedTextureHandle {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
    pub format: DxtFormat,
}

impl CompressedTextureHandle {
    /// Create a new compressed texture handle
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: DxtFormat,
        label: Option<&str>,
    ) -> Self {
        let texture = create_compressed_texture(device, width, height, format, label);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture: Arc::new(texture),
            view: Arc::new(view),
            width,
            height,
            format,
        }
    }

    /// Upload compressed data
    pub fn upload(&self, queue: &wgpu::Queue, data: &[u8]) {
        upload_compressed_texture(
            queue,
            &self.texture,
            data,
            self.width,
            self.height,
            self.format,
        );
    }

    /// Get expected data size
    pub fn expected_size(&self) -> usize {
        self.format.calculate_size(self.width, self.height)
    }
}

/// Check if the GPU adapter supports BC (DXT) compressed textures
pub fn check_bc_support(adapter: &wgpu::Adapter) -> bool {
    let features = adapter.features();
    features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxt_format_sizes() {
        // 1920x1080 BC1
        let size = DxtFormat::Bc1.calculate_size(1920, 1080);
        assert_eq!(size, 480 * 270 * 8); // 1036800 bytes

        // 1920x1080 BC3
        let size = DxtFormat::Bc3.calculate_size(1920, 1080);
        assert_eq!(size, 480 * 270 * 16); // 2073600 bytes
    }

    #[test]
    fn test_wgpu_formats() {
        assert_eq!(
            DxtFormat::Bc1.wgpu_format(),
            wgpu::TextureFormat::Bc1RgbaUnorm
        );
        assert_eq!(
            DxtFormat::Bc3.wgpu_format(),
            wgpu::TextureFormat::Bc3RgbaUnorm
        );
    }
}
