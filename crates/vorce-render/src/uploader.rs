use std::sync::Arc;
use wgpu::util::DeviceExt;

/// Frame uploader for efficient threaded texture uploads
pub struct WgpuFrameUploader {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl WgpuFrameUploader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self { device, queue }
    }

    /// Upload frame data to a texture.
    ///
    /// This method uses a staging buffer (created with `create_buffer_init`) and
    /// `copy_buffer_to_texture` to perform the upload. This is generally non-blocking
    /// for the CPU (except for allocation/mapping overhead) and allows the driver
    /// to schedule the transfer asynchronously.
    ///
    /// It automatically handles padding if the row stride is not 256-byte aligned.
    pub fn upload(&self, texture: &wgpu::Texture, data: &[u8], width: u32, height: u32) {
        let bytes_per_pixel = 4;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = 256;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        let buffer = if padding == 0 {
            // Data is already aligned, direct upload
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Frame Upload Staging Buffer"),
                    contents: data,
                    usage: wgpu::BufferUsages::COPY_SRC,
                })
        } else {
            // Data is not aligned, need to repack
            let mut padded_data = Vec::with_capacity((padded_bytes_per_row * height) as usize);
            for i in 0..height {
                let start = (i * unpadded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                if end <= data.len() {
                    padded_data.extend_from_slice(&data[start..end]);
                    #[allow(clippy::manual_repeat_n)]
                    padded_data.extend(std::iter::repeat(0u8).take(padding as usize));
                }
            }

            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Frame Upload Staging Buffer (Padded)"),
                    contents: &padded_data,
                    usage: wgpu::BufferUsages::COPY_SRC,
                })
        };

        // Create command encoder for the copy
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Upload Encoder"),
            });

        // Copy from staging buffer to texture
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Submit the command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
