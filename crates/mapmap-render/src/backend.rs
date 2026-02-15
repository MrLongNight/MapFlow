//! Rendering backend abstraction.

use crate::{RenderError, Result, ShaderHandle, ShaderSource, TextureDescriptor, TextureHandle};
use std::sync::Arc;
use tracing::{debug, info};
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
    staging_belt: StagingBelt,
    texture_counter: u64,
    shader_counter: u64,
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

        info!("Primary backend initialization failed, attempting GL fallback...");

        // 2. Fallback to GL if PRIMARY failed
        // Note: This step might still panic on headless systems if GL is selected but unavailable,
        // but it's a necessary fallback for older hardware.
        Self::new_with_options(
            wgpu::Backends::GL,
            wgpu::PowerPreference::HighPerformance,
            preferred_gpu,
        )
        .await
    }

    /// Create a new wgpu backend with specific options
    pub async fn new_with_options(
        backends: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        preferred_gpu: Option<&str>,
    ) -> Result<Self> {
        info!("Initializing wgpu backend");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let mut adapter = None;

        if let Some(gpu_name) = preferred_gpu {
            if !gpu_name.is_empty() {
                if gpu_name.contains("Microsoft Basic Render Driver") {
                    tracing::warn!("Ignoring preferred GPU '{}' as it is a software renderer. Falling back to auto-selection.", gpu_name);
                } else {
                    let adapters = instance.enumerate_adapters(backends);
                    for a in adapters {
                        let info = a.get_info();
                        if info.name == gpu_name {
                            info!("Found preferred adapter: {}", info.name);
                            adapter = Some(a);
                            break;
                        }
                    }
                    if adapter.is_none() {
                        tracing::warn!(
                            "Preferred GPU '{}' not found, falling back to auto-selection.",
                            gpu_name
                        );
                    }
                }
            }
        }

        if adapter.is_none() {
            // Manual selection to prioritize Discrete > Integrated > CPU
            let adapters = instance.enumerate_adapters(backends);
            let mut best_adapter = None;
            let mut best_score = -1;

            for a in adapters {
                let info = a.get_info();
                let score = match info.device_type {
                    wgpu::DeviceType::DiscreteGpu => 3,
                    wgpu::DeviceType::IntegratedGpu => 2,
                    wgpu::DeviceType::VirtualGpu => 1,
                    wgpu::DeviceType::Cpu => 0,
                    wgpu::DeviceType::Other => 0,
                };

                if score > best_score {
                    best_score = score;
                    best_adapter = Some(a);
                }
            }

            if let Some(a) = best_adapter {
                let info = a.get_info();
                info!(
                    "Auto-selected best adapter: {} ({:?})",
                    info.name, info.device_type
                );
                adapter = Some(a);
            }
        }

        if adapter.is_none() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        let adapter =
            adapter.ok_or_else(|| RenderError::DeviceError("No adapter found".to_string()))?;

        let adapter_info = adapter.get_info();
        info!(
            "Selected adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("MapFlow Device"),
                required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                ..Default::default()
            })
            .await
            .map_err(|e: wgpu::RequestDeviceError| RenderError::DeviceError(e.to_string()))?;

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

    /// Recall staging belt buffers
    pub fn recall_staging_belt(&mut self) {
        self.staging_belt.recall();
    }

    /// Finish staging belt
    pub fn finish_staging_belt(&mut self) {
        self.staging_belt.finish();
    }

    /// Get the preferred surface format.
    /// Note: This is hardcoded for now as the backend is surface-agnostic.
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
            label: Some(&format!("Texture {}", self.texture_counter)),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: desc.mip_levels,
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
        };

        self.texture_counter += 1;
        debug!(
            "Created texture {} ({}x{})",
            handle.id, desc.width, desc.height
        );

        Ok(handle)
    }

    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()> {
        let bytes_per_pixel = match handle.format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            _ => {
                return Err(RenderError::TextureCreation(
                    "Unsupported texture format for upload".to_string(),
                ))
            }
        };

        let expected_size = (handle.width * handle.height * bytes_per_pixel) as usize;
        if data.len() != expected_size {
            return Err(RenderError::TextureCreation(format!(
                "Data size mismatch: expected {}, got {}",
                expected_size,
                data.len()
            )));
        }

        // Calculate row stride
        let bytes_per_row = handle.width * bytes_per_pixel;

        // Use direct write for all textures (queue.write_texture is efficient)
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
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(handle.height),
            },
            wgpu::Extent3d {
                width: handle.width,
                height: handle.height,
                depth_or_array_layers: 1,
            },
        );

        debug!(
            "Uploaded texture {} ({}x{}, {} bytes)",
            handle.id,
            handle.width,
            handle.height,
            data.len()
        );
        Ok(())
    }

    fn create_shader(&mut self, source: ShaderSource) -> Result<ShaderHandle> {
        let module = match source {
            ShaderSource::Wgsl(ref code) => {
                self.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(&format!("Shader {}", self.shader_counter)),
                        source: wgpu::ShaderSource::Wgsl(code.clone().into()),
                    })
            }
        };

        let handle = ShaderHandle {
            id: self.shader_counter,
            module: Arc::new(module),
        };

        self.shader_counter += 1;
        debug!("Created shader {}", handle.id);

        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        pollster::block_on(async {
            let backend = WgpuBackend::new(None).await;
            if backend.is_err() {
                // Skipping test on CI/Headless systems without GPU support.
                eprintln!("SKIP: Backend konnte nicht initialisiert werden (möglicherweise kein GPU-Backend/HW im CI).");
                return;
            }
            assert!(backend.is_ok());

            if let Ok(backend) = backend {
                println!("Backend: {:?}", backend.adapter_info);
            }
        });
    }

    #[test]
    fn test_initialization_robustness() {
        pollster::block_on(async {
            // This test ensures that trying to create a backend doesn't panic,
            // even if it fails.
            let result = WgpuBackend::new(None).await;
            match result {
                Ok(b) => println!("Backend init success: {:?}", b.adapter_info),
                Err(e) => println!("Backend init failed gracefully: {}", e),
            }
        });
    }
}
