import re

with open("crates/mapmap-render/src/spout.rs", "r") as f:
    data = f.read()

replacement = """pub unsafe fn texture_from_shared_handle(
    device: &wgpu::Device,
    handle: NonNull<std::ffi::c_void>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> Result<Texture, &'static str> {
    Err("Spout integration is currently disabled due to wgpu backend changes")
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
    Err("Spout integration is currently disabled due to wgpu backend changes")
}
"""

data = re.sub(r"""pub unsafe fn texture_from_shared_handle\(
    device: &wgpu::Device,
    handle: NonNull<std::ffi::c_void>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
\) -> Result<Texture, &'static str> \{
    if let Some\(dx12_device\) = device\.as_hal::<wgpu_hal::dx12::Api>\(\) \{
        let id3d12_device: &ID3D12Device = dx12_device\.raw_device\(\);
        let mut resource: Option<ID3D12Resource> = None;
        let nt_handle = HANDLE\(handle\.as_ptr\(\) as \*mut _\);

        // Open the shared handle
        if id3d12_device
            \.OpenSharedHandle\(nt_handle, &mut resource\)
            \.is_ok\(\)
        \{
            if let Some\(res\) = resource \{
                let size = wgpu::Extent3d \{
                    width,
                    height,
                    depth_or_array_layers: 1,
                \};

                let hal_texture = wgpu_hal::dx12::Device::texture_from_raw\(
                    res,
                    format,
                    wgpu::TextureDimension::D2,
                    size,
                    1,
                    1,
                \);

                let desc = wgpu::TextureDescriptor \{
                    label: Some\("Spout Shared Texture"\),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING \| wgpu::TextureUsages::COPY_SRC,
                    view_formats: &\[\],
                \};

                return Ok\(
                    device\.create_texture_from_hal::<wgpu_hal::dx12::Api>\(hal_texture, &desc\)
                \);
            \}
        \}
        return Err\("Failed to bridge handle to DX12 resource"\);
    \}

    Err\("Spout integration is only supported on the DX12 backend"\)
\}

#\[cfg\(target_os = "windows"\)\]
/// Create a shared handle from a WGPU texture\.
///
/// # Safety
///
/// This function is unsafe because obtaining a shared handle involves platform-specific
/// interop calls which may result in undefined behavior if the texture is not compatible
/// with sharing \(e\.g\., incorrect usage flags or format\) or if the device context is lost\.
pub unsafe fn shared_handle_from_texture\(
    device: &wgpu::Device,
    texture: &wgpu::Texture,
\) -> Result<NonNull<std::ffi::c_void>, &'static str> \{
    if let \(Some\(dx12_texture\), Some\(dx12_device\)\) = \(
        texture\.as_hal::<wgpu_hal::dx12::Api>\(\),
        device\.as_hal::<wgpu_hal::dx12::Api>\(\),
    \) \{
        let resource: &ID3D12Resource = dx12_texture\.raw_resource\(\);
        let id3d12_device: &ID3D12Device = dx12_device\.raw_device\(\);

        // Pass a null string explicitly for NT handle name\.
        let name = windows::core::PCWSTR::null\(\);

        // 0x10000000 = GENERIC_ALL
        let handle = id3d12_device
            \.CreateSharedHandle\(resource, None, 0x10000000, name\)
            \.map_err\(\|\_\| "Failed to create shared handle from resource"\)\?;

        if let Some\(ptr\) = NonNull::new\(handle\.0\) \{
            return Ok\(ptr\);
        \}
        return Err\("Created handle was null"\);
    \}

    Err\("Spout integration is only supported on the DX12 backend"\)
\}
""", replacement, data)

with open("crates/mapmap-render/src/spout.rs", "w") as f:
    f.write(data)
