import os
import glob
import re

render_dir = 'crates/mapmap-render/src'

for filepath in glob.glob(f'{render_dir}/**/*.rs', recursive=True):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # 1. Remove push_constant_ranges: &[],
    content = re.sub(r'\s*push_constant_ranges:\s*&\[\],?', '', content)

    # 2. Remove multiview: None,
    content = re.sub(r'\s*multiview:\s*None,?', '', content)

    # 3. Change mipmap_filter: wgpu::FilterMode::Linear to MipmapFilterMode::Linear
    content = re.sub(r'mipmap_filter:\s*wgpu::FilterMode::Linear', 'mipmap_filter: wgpu::MipmapFilterMode::Linear', content)
    content = re.sub(r'mipmap_filter:\s*wgpu::FilterMode::Nearest', 'mipmap_filter: wgpu::MipmapFilterMode::Nearest', content)

    # 4. Add multiview_mask: None to RenderPassDescriptor
    # Find &wgpu::RenderPassDescriptor { and inject multiview_mask: None, inside it.
    # To be safe, we can look for occlusion_query_set: None, or 	imestamp_writes: None, and append it there.
    content = re.sub(r'(timestamp_writes:\s*None,)', r'\1\n                multiview_mask: None,', content)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

# Fix StagingBelt
backend_path = os.path.join(render_dir, 'backend.rs')
with open(backend_path, 'r', encoding='utf-8') as f:
    backend = f.read()
backend = backend.replace('StagingBelt::new(1024 * 1024)', 'StagingBelt::new(&device, 1024 * 1024)')
with open(backend_path, 'w', encoding='utf-8') as f:
    f.write(backend)

print('WGPU 28 fixes applied.')
