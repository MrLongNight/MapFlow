//! Paint texture cache - manages GPU textures for paints

use mapmap_core::paint::{Paint, PaintId, PaintType};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu;

/// Caches GPU textures for paints to avoid recreating them every frame
pub struct PaintTextureCache {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    /// Map of PaintId -> (TextureView, last_updated_version)
    cache: RwLock<HashMap<PaintId, CachedTexture>>,
}

#[allow(dead_code)]
struct CachedTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl PaintTextureCache {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create texture view for a paint
    pub fn get_texture_view(&self, paint: &Paint) -> wgpu::TextureView {
        let mut cache = self.cache.write();

        // Check if we already have this paint cached
        if let Some(cached) = cache.get(&paint.id) {
            // Return a new view of the cached texture
            return cached
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }

        // Create new texture for this paint
        let (width, height) = (paint.dimensions.x as u32, paint.dimensions.y as u32);
        let texture = self.create_texture_for_paint(paint, width, height);
        let _view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cached = CachedTexture {
            texture,
            view: self
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("dummy"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default()), // Placeholder, not used
            width,
            height,
        };

        // Store the actual texture, get a view from it
        let result_view = cached
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        cache.insert(paint.id, cached);

        result_view
    }

    /// Create a GPU texture for a paint based on its type
    fn create_texture_for_paint(&self, paint: &Paint, width: u32, height: u32) -> wgpu::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("paint_{}", paint.id)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Generate texture data based on paint type
        let data = match paint.paint_type {
            PaintType::Color => self.generate_solid_color(width, height, paint.color),
            PaintType::TestPattern => self.generate_test_pattern(width, height),
            PaintType::Image => {
                // TODO: Load from source_path
                self.generate_test_pattern(width, height)
            }
            PaintType::Video => {
                // TODO: Load from video decoder
                self.generate_test_pattern(width, height)
            }
            PaintType::Camera => {
                // TODO: Load from camera capture
                self.generate_solid_color(width, height, [0.2, 0.2, 0.2, 1.0])
            }
        };

        // Upload to GPU
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    /// Generate solid color texture data
    fn generate_solid_color(&self, width: u32, height: u32, color: [f32; 4]) -> Vec<u8> {
        let r = (color[0] * 255.0) as u8;
        let g = (color[1] * 255.0) as u8;
        let b = (color[2] * 255.0) as u8;
        let a = (color[3] * 255.0) as u8;

        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
        data
    }

    /// Generate classic test pattern (color bars)
    fn generate_test_pattern(&self, width: u32, height: u32) -> Vec<u8> {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);

        // Color bars: White, Yellow, Cyan, Green, Magenta, Red, Blue, Black
        let colors: [[u8; 4]; 8] = [
            [255, 255, 255, 255], // White
            [255, 255, 0, 255],   // Yellow
            [0, 255, 255, 255],   // Cyan
            [0, 255, 0, 255],     // Green
            [255, 0, 255, 255],   // Magenta
            [255, 0, 0, 255],     // Red
            [0, 0, 255, 255],     // Blue
            [0, 0, 0, 255],       // Black
        ];

        let bar_width = width / 8;

        for _y in 0..height {
            for x in 0..width {
                let bar_index = ((x / bar_width) as usize).min(7);
                let color = colors[bar_index];
                data.push(color[0]);
                data.push(color[1]);
                data.push(color[2]);
                data.push(color[3]);
            }
        }

        data
    }

    /// Invalidate cache for a specific paint (call when paint changes)
    pub fn invalidate(&self, paint_id: PaintId) {
        self.cache.write().remove(&paint_id);
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}
