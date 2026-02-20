//! Texture management and pooling

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Handle to a GPU texture
#[derive(Clone)]
pub struct TextureHandle {
    pub id: u64,
    pub texture: Arc<wgpu::Texture>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    /// Timestamp (seconds since pool creation) when last used
    pub last_used: Arc<AtomicU64>,
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

    /// Mark as used now
    pub fn mark_used(&self, start_time: Instant) {
        let now_secs = start_time.elapsed().as_secs();
        self.last_used.store(now_secs, Ordering::Relaxed);
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
    views: RwLock<HashMap<String, Arc<wgpu::TextureView>>>,
    start_time: Instant,
}

impl TexturePool {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            textures: RwLock::new(HashMap::new()),
            views: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
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
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

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

        let last_used = Arc::new(AtomicU64::new(self.start_time.elapsed().as_secs()));

        let handle = TextureHandle {
            id,
            texture: Arc::new(texture),
            width,
            height,
            format,
            last_used,
        };

        let view = handle.create_view();
        let view_arc = Arc::new(view);

        let name_owned = name.to_string();

        self.textures.write().insert(name_owned.clone(), handle);
        self.views.write().insert(name_owned.clone(), view_arc);

        name_owned
    }

    /// Get a texture view by name.
    pub fn get_view(&self, name: &str) -> Arc<wgpu::TextureView> {
        // Refresh usage timestamp
        if let Some(handle) = self.textures.read().get(name) {
            handle.mark_used(self.start_time);
        }

        // Fast path: check views cache
        if let Some(view) = self.views.read().get(name).cloned() {
            return view;
        }

        // Slow path: create from handle
        let view = self
            .textures
            .read()
            .get(name)
            .expect("Texture not found in pool")
            .create_view();

        Arc::new(view)
    }

    /// Alias an existing texture to a new name
    pub fn alias_texture(&self, src_name: &str, dest_name: &str) -> bool {
        let textures = self.textures.read();
        if let Some(handle) = textures.get(src_name) {
            handle.mark_used(self.start_time);
            let handle_clone = handle.clone();
            drop(textures);

            self.textures
                .write()
                .insert(dest_name.to_string(), handle_clone);

            if let Some(view) = self.views.read().get(src_name).cloned() {
                self.views.write().insert(dest_name.to_string(), view);
            }
            true
        } else {
            false
        }
    }

    /// Check if a texture exists in the pool.
    pub fn has_texture(&self, name: &str) -> bool {
        let exists = self.textures.read().contains_key(name);
        if exists {
            if let Some(handle) = self.textures.read().get(name) {
                handle.mark_used(self.start_time);
            }
        }
        exists
    }

    /// Ensure a texture exists with specific properties.
    pub fn ensure_texture(
        &self,
        name: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) {
        {
            let textures = self.textures.read();
            if let Some(handle) = textures.get(name) {
                if handle.width == width
                    && handle.height == height
                    && handle.format == format
                    && handle.texture.usage() == usage
                {
                    handle.mark_used(self.start_time);
                    return;
                }
            }
        }
        self.create(name, width, height, format, usage);
    }

    /// Resize a texture if its dimensions have changed.
    pub fn resize_if_needed(&self, name: &str, new_width: u32, new_height: u32) {
        {
            let textures = self.textures.read();
            if let Some(handle) = textures.get(name) {
                handle.mark_used(self.start_time);
                if handle.width == new_width && handle.height == new_height {
                    return;
                }
            } else {
                return;
            }
        }

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
                handle.mark_used(self.start_time);

                let new_view = handle.create_view();
                self.views
                    .write()
                    .insert(name.to_string(), Arc::new(new_view));
            }
        }
    }

    /// Upload data to a texture.
    pub fn upload_data(
        &self,
        queue: &wgpu::Queue,
        name: &str,
        data: &[u8],
        width: u32,
        height: u32,
    ) {
        self.resize_if_needed(name, width, height);

        let handle = {
            let textures = self.textures.read();
            textures.get(name).cloned()
        };

        let handle = if let Some(h) = handle {
            h
        } else {
            self.create(
                name,
                width,
                height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            return self.upload_data(queue, name, data, width, height);
        };

        handle.mark_used(self.start_time);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &handle.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width: handle.width,
                height: handle.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Release a texture manually.
    pub fn release(&self, name: &str) {
        self.textures.write().remove(name);
        self.views.write().remove(name);
    }

    /// Perform Garbage Collection: remove textures not used for TTL duration.
    pub fn collect_garbage(&self, ttl: Duration) -> usize {
        let now_secs = self.start_time.elapsed().as_secs();
        let ttl_secs = ttl.as_secs();

        let mut to_remove = Vec::new();
        {
            let textures = self.textures.read();
            for (name, handle) in textures.iter() {
                // Persistent textures should never be GC'd
                if name == "composite" || name.starts_with("layer_pong") || name == "bevy_output" {
                    continue;
                }

                let last_used = handle.last_used.load(Ordering::Relaxed);
                if now_secs > last_used + ttl_secs {
                    to_remove.push(name.clone());
                }
            }
        }

        let removed_count = to_remove.len();
        if !to_remove.is_empty() {
            let mut textures = self.textures.write();
            let mut views = self.views.write();
            for name in to_remove {
                textures.remove(&name);
                views.remove(&name);
            }
        }

        removed_count
    }

    /// Get current stats
    pub fn get_stats(&self) -> PoolStats {
        let textures = self.textures.read();
        let total_memory = textures.values().map(|h| h.size_bytes()).sum();

        PoolStats {
            total_textures: textures.len(),
            free_textures: 0,
            total_memory,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_textures: usize,
    pub free_textures: usize,
    pub total_memory: u64,
}
