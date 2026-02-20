//! Rendering backend abstraction

use crate::{RenderError, Result, ShaderHandle, ShaderSource, TextureDescriptor, TextureHandle};
use std::sync::Arc;
use tracing::{debug, info, warn};
use wgpu::util::StagingBelt;

/// Trait for rendering backends
pub trait RenderBackend: Send {
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<TextureHandle>;
    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()>;
    fn create_shader(&mut self, source: ShaderSource) -> Result<ShaderHandle>;
}

/// wgpu-based rendering backend
pub struct WgpuBackend {
    pub instance: Arc<wgpu::Instance>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub adapter_info: wgpu::AdapterInfo,
    #[allow(dead_code)]
    staging_belt: StagingBelt,
    texture_counter: u64,
    shader_counter: u64,
    start_time: std::time::Instant,
}

impl WgpuBackend {
    /// Create a new wgpu backend
    ///
    /// This implementation is robust against initialization failures on specific backends
    /// (like GL panicking on headless systems). It prioritizes modern backends (Vulkan, Metal, DX12, DX11)
    /// and falls back to GL only if necessary.
    pub async fn new(preferred_gpu: Option<&str>) -> Result<Self> {
        // 1. Try all backends EXCEPT GL first.
        // This includes Vulkan, Metal, DX12, and DX11.
        // We explicitly exclude GL to avoid the "BadDisplay" panic on headless systems
        // where wgpu tries to initialize EGL/GLX eagerly.
        let safe_backends = wgpu::Backends::all() & !wgpu::Backends::GL;
        let primary_result = Self::new_with_options(
            safe_backends,
            wgpu::PowerPreference::HighPerformance,
            preferred_gpu,
        )
        .await;

        if primary_result.is_ok() {
            return primary_result;
        }

        // 2. Fallback to ALL backends including GL if the modern ones failed.
        warn!("Modern graphics backends failed. Falling back to OpenGL/Software...");
        Self::new_with_options(
            wgpu::Backends::all(),
            wgpu::PowerPreference::LowPower,
            preferred_gpu,
        )
        .await
    }

    async fn new_with_options(
        backends: wgpu::Backends,
        power_pref: wgpu::PowerPreference,
        preferred_gpu: Option<&str>,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .map_err(|e| RenderError::DeviceError(e.to_string()))?;

        let adapter_info = adapter.get_info();
        info!(
            "Using GPU: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        // Filter based on preferred GPU name if provided
        if let Some(preferred) = preferred_gpu {
            if !adapter_info
                .name
                .to_lowercase()
                .contains(&preferred.to_lowercase())
            {
                warn!(
                    "Current GPU '{}' does not match preferred '{}'. This might be a secondary adapter.",
                    adapter_info.name, preferred
                );
            }
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("MapFlow Device"),
                required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::DeviceError(e.to_string()))?;

        info!("Device created successfully");

        let staging_belt = StagingBelt::new(1024 * 1024); // 1MB chunks

        Ok(Self {
            instance: Arc::new(instance),
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info,
            staging_belt,
            texture_counter: 0,
            shader_counter: 0,
            start_time: std::time::Instant::now(),
        })
    }

    /// Create a surface using the backend's instance
    ///
    /// # Safety
    /// The window must outlive the surface
    pub fn create_surface(
        &self,
        window: Arc<winit::window::Window>,
    ) -> Result<wgpu::Surface<'static>> {
        self.instance
            .create_surface(window)
            .map_err(move |e| RenderError::DeviceError(format!("Failed to create surface: {}", e)))
    }

    /// Get device limits
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    /// Get adapter info
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// Set surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Bgra8UnormSrgb
    }
}

impl RenderBackend for WgpuBackend {
    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn create_texture(&mut self, desc: TextureDescriptor) -> Result<TextureHandle> {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Backend Texture"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: desc.usage,
            view_formats: &[],
        });

        let handle = TextureHandle {
            id: self.texture_counter,
            texture: Arc::new(texture),
            width: desc.width,
            height: desc.height,
            format: desc.format,
            last_used: Arc::new(std::sync::atomic::AtomicU64::new(
                self.start_time.elapsed().as_secs(),
            )),
        };

        self.texture_counter += 1;
        debug!(
            "Created texture {} ({}x{})",
            handle.id, desc.width, desc.height
        );
        Ok(handle)
    }

    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()> {
        let bytes_per_pixel = 4; // Assume RGBA8
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &handle.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(handle.width * bytes_per_pixel),
                rows_per_image: Some(handle.height),
            },
            wgpu::Extent3d {
                width: handle.width,
                height: handle.height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    fn create_shader(&mut self, source: ShaderSource) -> Result<ShaderHandle> {
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("Shader {}", self.shader_counter)),
                source: match source {
                    ShaderSource::Wgsl(s) => wgpu::ShaderSource::Wgsl(s.into()),
                },
            });

        let handle = ShaderHandle {
            id: self.shader_counter,
            module: Arc::new(module),
        };

        self.shader_counter += 1;
        Ok(handle)
    }
}
