#[cfg(target_os = "windows")]
pub mod spout {
    use std::ptr::NonNull;
    use wgpu::Texture;

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

    pub unsafe fn shared_handle_from_texture(
        _texture: &wgpu::Texture,
    ) -> Result<NonNull<std::ffi::c_void>, &'static str> {
        // TODO: Update Spout wgpu integration for wgpu 0.19
        Err("Spout integration requires update for wgpu 0.19 (DX11/DX12 interop changes)")
    }
}
