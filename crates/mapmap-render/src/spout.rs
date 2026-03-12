#[cfg(target_os = "windows")]
use std::ptr::NonNull;
#[cfg(target_os = "windows")]
use wgpu::Texture;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HANDLE;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D12::{ID3D12Device, ID3D12Resource};

#[cfg(target_os = "windows")]
/// Create a WGPU texture from a shared handle.
///
/// # Safety
///
/// This function is unsafe because it takes a raw pointer (`NonNull<std::ffi::c_void>`)
/// representing a shared resource handle. The caller must ensure that the handle is valid,
/// refers to a compatible D3D11/D3D12 resource, and that the resource is kept alive
/// for the duration of the texture's usage.
pub unsafe fn texture_from_shared_handle(
    device: &wgpu::Device,
    handle: NonNull<std::ffi::c_void>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> Result<Texture, &'static str> {
    if let Some(dx12_device) = device.as_hal::<wgpu_hal::dx12::Api>() {
        let id3d12_device: &ID3D12Device = dx12_device.raw_device();
        let mut resource: Option<ID3D12Resource> = None;
        let nt_handle = HANDLE(handle.as_ptr() as *mut _);

        // Open the shared handle
        if id3d12_device
            .OpenSharedHandle(nt_handle, &mut resource)
            .is_ok()
        {
            if let Some(res) = resource {
                let size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };

                let hal_texture = wgpu_hal::dx12::Device::texture_from_raw(
                    res,
                    format,
                    wgpu::TextureDimension::D2,
                    size,
                    1,
                    1,
                );

                let desc = wgpu::TextureDescriptor {
                    label: Some("Spout Shared Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                };

                return Ok(
                    device.create_texture_from_hal::<wgpu_hal::dx12::Api>(hal_texture, &desc)
                );
            }
        }
        return Err("Failed to bridge handle to DX12 resource");
    }

    Err("Spout integration is only supported on the DX12 backend")
}

#[cfg(target_os = "windows")]
/// Create a shared handle from a WGPU texture.
///
/// # Safety
///
/// This function is unsafe because obtaining a shared handle involves platform-specific
/// interop calls which may result in undefined behavior if the texture is not compatible
/// with sharing (e.g., incorrect usage flags or format) or if the device context is lost.
pub unsafe fn shared_handle_from_texture(
    device: &wgpu::Device,
    texture: &wgpu::Texture,
) -> Result<NonNull<std::ffi::c_void>, &'static str> {
    if let (Some(dx12_texture), Some(dx12_device)) = (
        texture.as_hal::<wgpu_hal::dx12::Api>(),
        device.as_hal::<wgpu_hal::dx12::Api>(),
    ) {
        let resource: &ID3D12Resource = dx12_texture.raw_resource();
        let id3d12_device: &ID3D12Device = dx12_device.raw_device();

        // Pass a null string explicitly for NT handle name.
        let name = windows::core::PCWSTR::null();

        // 0x10000000 = GENERIC_ALL
        let handle = id3d12_device
            .CreateSharedHandle(resource, None, 0x10000000, name)
            .map_err(|_| "Failed to create shared handle from resource")?;

        if let Some(ptr) = NonNull::new(handle.0) {
            return Ok(ptr);
        }
        return Err("Created handle was null");
    }

    Err("Spout integration is only supported on the DX12 backend")
}
