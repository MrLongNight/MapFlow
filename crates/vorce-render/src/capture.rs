//! Visual capture and readback utilities.

use anyhow::{anyhow, Context, Result};
use image::RgbaImage;
use std::path::Path;

/// Queues a copy of a texture to a readback buffer.
///
/// Returns the buffer and the padded bytes per row.
pub fn queue_readback_copy(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> (wgpu::Buffer, u32) {
    let bytes_per_pixel = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Capture Readback Buffer"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    (buffer, padded_bytes_per_row)
}

/// Saves a readback buffer to a PNG file.
pub fn save_readback_buffer(
    device: &wgpu::Device,
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    format: wgpu::TextureFormat,
    output_path: &Path,
) -> Result<()> {
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});

    // In a real app, we might want to poll outside this function,
    // but for simple capture this is fine.
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    let mapped = slice.get_mapped_range();
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for row in mapped
        .chunks_exact(padded_bytes_per_row as usize)
        .take(height as usize)
    {
        for pixel in row[..(width * 4) as usize].chunks_exact(4) {
            match format {
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
                _ => rgba.extend_from_slice(pixel),
            }
        }
    }

    drop(mapped);
    buffer.unmap();

    save_rgba_png(width, height, &rgba, output_path)
}

/// Saves raw RGBA pixels to a PNG file.
pub fn save_rgba_png(width: u32, height: u32, pixels: &[u8], output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let image = RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| anyhow!("failed to assemble RGBA image buffer"))?;
    image
        .save(output_path)
        .with_context(|| format!("failed to save {}", output_path.display()))?;
    Ok(())
}
