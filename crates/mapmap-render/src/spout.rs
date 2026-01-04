#[cfg(target_os = "windows")]
#[allow(clippy::module_inception)]
pub mod spout {
    use std::ptr::NonNull;
    use wgpu::Texture;

    /// Creates a wgpu texture from a shared Windows handle.
    ///
    /// # Safety
    /// - `handle` must be a valid shared texture handle obtained from Spout
    /// - The handle must remain valid for the lifetime of the returned texture
    /// - The device must support DX11/DX12 interop
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

    /// Extracts a shared Windows handle from a wgpu texture.
    ///
    /// # Safety
    /// - `texture` must be a valid texture created with shared handle support
    /// - The returned handle must not outlive the texture
    /// - Caller is responsible for proper handle cleanup
    pub unsafe fn shared_handle_from_texture(
        _texture: &wgpu::Texture,
    ) -> Result<NonNull<std::ffi::c_void>, &'static str> {
        // TODO: Update Spout wgpu integration for wgpu 0.19
        Err("Spout integration requires update for wgpu 0.19 (DX11/DX12 interop changes)")
    }
}
