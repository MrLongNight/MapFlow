import os
import glob
import re

for f in glob.glob("crates/mapmap-render/src/**/*.rs", recursive=True):
    with open(f, 'r', encoding='utf-8') as file:
        c = file.read()

    # 1. PipelineLayoutDescriptor: replace push_constant_ranges with immediate_size
    c = re.sub(r'push_constant_ranges:\s*&\[\],?', 'immediate_size: 0,', c)

    # 2. RenderPipelineDescriptor: replace multiview with multiview_mask
    c = re.sub(r'multiview:\s*None,?', 'multiview_mask: None,', c)

    # 3. RenderPassDescriptor: add multiview_mask: None after timestamp_writes
    c = re.sub(r'(timestamp_writes:\s*None,)', r'\1\n                multiview_mask: None,', c)

    # 4. Mipmap filters
    c = re.sub(r'mipmap_filter:\s*wgpu::FilterMode::Linear', 'mipmap_filter: wgpu::MipmapFilterMode::Linear', c)
    c = re.sub(r'mipmap_filter:\s*wgpu::FilterMode::Nearest', 'mipmap_filter: wgpu::MipmapFilterMode::Nearest', c)

    with open(f, 'w', encoding='utf-8') as file:
        file.write(c)

# 5. Fix StagingBelt
backend_path = 'crates/mapmap-render/src/backend.rs'
with open(backend_path, 'r', encoding='utf-8') as file:
    c = file.read()
c = c.replace('StagingBelt::new(&device,', 'StagingBelt::new(device.clone(),')
c = c.replace('StagingBelt::new(device,', 'StagingBelt::new(device.clone(),')
with open(backend_path, 'w', encoding='utf-8') as file:
    file.write(c)

print('WGPU 28 exact fixes applied.')
