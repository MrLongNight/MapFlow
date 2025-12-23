//! Texture management and pooling

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Handle to a GPU texture
#[derive(Clone)]
pub struct TextureHandle {
    pub id: u64,
    pub texture: Arc<wgpu::Texture>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl TextureHandle {
    /// Create a texture view
    pub fn create_view(&self) -> wgpu::TextureView {
        self.texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// Get texture size in bytes
    pub fn size_bytes(&self) -> u64 {
        let bytes_per_pixel = match self.format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => 4, // Default to 4 bytes
        };
        (self.width * self.height * bytes_per_pixel) as u64
    }
}

/// Texture descriptor
#[derive(Debug, Clone, Copy)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub mip_levels: u32,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_levels: 1,
        }
    }
}

/// Texture pool for reusing allocations
pub struct TexturePool {
    device: Arc<wgpu::Device>,
    textures: RwLock<HashMap<String, TextureHandle>>,
    views: RwLock<HashMap<String, wgpu::TextureView>>,
}

impl TexturePool {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            textures: RwLock::new(HashMap::new()),
            views: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new managed texture.
    pub fn create(
        &self,
        name: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> String {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let handle = TextureHandle {
            id,
            texture: Arc::new(texture),
            width,
            height,
            format,
        };

        let view = handle.create_view();
        let name_owned = name.to_string();

        self.textures.write().insert(name_owned.clone(), handle);
        self.views.write().insert(name_owned.clone(), view);

        name_owned
    }

    /// Get a texture view by name.
    pub fn get_view(&self, name: &str) -> wgpu::TextureView {
        self.textures
            .read()
            .get(name)
            .expect("Texture not found in pool")
            .create_view()
    }

    /// Resize a texture if its dimensions have changed.
    pub fn resize_if_needed(&self, name: &str, new_width: u32, new_height: u32) {
        let mut textures = self.textures.write();
        if let Some(handle) = textures.get_mut(name) {
            if handle.width != new_width || handle.height != new_height {
                let new_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(name),
                    size: wgpu::Extent3d {
                        width: new_width,
                        height: new_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: handle.format,
                    usage: handle.texture.usage(),
                    view_formats: &[],
                });

                handle.texture = Arc::new(new_texture);
                handle.width = new_width;
                handle.height = new_height;

                let new_view = handle.create_view();
                self.views.write().insert(name.to_string(), new_view);
            }
        }
    }

    /// Release a texture, making it available for reuse or deallocation.
    pub fn release(&self, name: &str) {
        self.textures.write().remove(name);
        self.views.write().remove(name);
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_textures: usize,
    pub free_textures: usize,
    pub total_memory: u64,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_texture_descriptor_default() {
//         let desc = TextureDescriptor::default();
//         assert_eq!(desc.width, 1);
//         assert_eq!(desc.height, 1);
//         assert_eq!(desc.mip_levels, 1);
//     }

//     #[test]
//     fn test_texture_pool() {
//         let pool = TexturePool::new(10);
//         let stats = pool.stats();
//         assert_eq!(stats.total_textures, 0);
//         assert_eq!(stats.free_textures, 0);
//     }
// }
