#[cfg(target_os = "windows")]
use std::ptr::NonNull;
#[cfg(target_os = "windows")]
use wgpu::Texture;

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
    _device: &wgpu::Device,
    _handle: NonNull<std::ffi::c_void>,
    _width: u32,
    _height: u32,
    _format: wgpu::TextureFormat,
) -> Result<Texture, &'static str> {
    // TODO: Update Spout wgpu integration for wgpu 0.19
    Err("Spout integration requires update for wgpu 0.19 (DX11/DX12 interop changes)")
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
    _texture: &wgpu::Texture,
) -> Result<NonNull<std::ffi::c_void>, &'static str> {
    // TODO: Update Spout wgpu integration for wgpu 0.19
    Err("Spout integration requires update for wgpu 0.19 (DX11/DX12 interop changes)")
}




