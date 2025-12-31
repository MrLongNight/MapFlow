
#[cfg(target_os = "windows")]
pub mod spout {
    use std::ptr::NonNull;
    use wgpu::Texture;

    pub unsafe fn texture_from_shared_handle(
        device: &wgpu::Device,
        handle: NonNull<std::ffi::c_void>,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Texture, &'static str> {
        let dx11_texture =
            wgpu_hal::dx12::Dx11Texture::new(handle.as_ptr() as *mut _, None, None);

        let hal_texture = wgpu_hal::dx12::Texture {
            inner: wgpu_hal::dx12::TextureInner::Dx11(dx11_texture),
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("spout_shared_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture_from_hal::<wgpu_hal::api::Dx12>(
            hal_texture,
            &texture_descriptor,
        );
        Ok(texture)
    }

    pub unsafe fn shared_handle_from_texture(
        texture: &wgpu::Texture,
    ) -> Result<NonNull<std::ffi::c_void>, &'static str> {
        let handle = match texture.as_hal::<wgpu_hal::api::Dx12, _>(|texture| {
            if let Some(wgpu_hal::dx12::TextureInner::Dx11(dx11_texture)) = texture.map(|t| &t.inner) {
                Some(dx11_texture.handle() as *mut std::ffi::c_void)
            } else {
                None
            }
        }) {
            Some(handle) => handle,
            None => return Err("Texture is not a DirectX 11 texture"),
        };

        NonNull::new(handle).ok_or("Received null handle")
    }
}
