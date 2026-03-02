import os
import glob
import re

render_dir = 'crates/mapmap-render/src'

for filepath in glob.glob(f'{render_dir}/**/*.rs', recursive=True):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Add immediate_size: 0, to PipelineLayoutDescriptor
    content = re.sub(r'bind_group_layouts:\s*&\[(.*?)\]\s*,', r'bind_group_layouts: &[\1],\n            push_constant_ranges: &[],\n            immediate_size: 0,', content, flags=re.DOTALL)
    # Wait, earlier I removed push_constant_ranges. So let's just replace it.
    # A safer way is to find PipelineLayoutDescriptor { and inject inside.
    content = re.sub(r'(&wgpu::PipelineLayoutDescriptor\s*\{)', r'\1\n            push_constant_ranges: &[],\n            immediate_size: 0,', content)

    # Add multiview_mask: 0, to RenderPipelineDescriptor
    content = re.sub(r'(&wgpu::RenderPipelineDescriptor\s*\{)', r'\1\n            multiview_mask: 0,', content)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

# Fix StagingBelt
backend_path = os.path.join(render_dir, 'backend.rs')
with open(backend_path, 'r', encoding='utf-8') as f:
    backend = f.read()
backend = backend.replace('StagingBelt::new(&device, 1024 * 1024)', 'StagingBelt::new(device.clone(), 1024 * 1024)')
with open(backend_path, 'w', encoding='utf-8') as f:
    f.write(backend)

print('WGPU 28 second fixes applied.')
