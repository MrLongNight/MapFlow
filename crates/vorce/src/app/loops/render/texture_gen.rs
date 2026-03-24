pub(crate) fn generate_grid_texture(width: u32, height: u32, layer_id: u64) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    let _bg_color = [0, 0, 0, 255];
    let _grid_color = [255, 255, 255, 255];
    let _text_color = [0, 255, 255, 255];

    for i in 0..(width * height) {
        let idx = (i * 4) as usize;
        data[idx] = 0;
        data[idx + 1] = 0;
        data[idx + 2] = 0;
        data[idx + 3] = 255;
    }
    let grid_step = 64;
    for y in 0..height {
        for x in 0..width {
            if x % grid_step == 0 || y % grid_step == 0 || x == width - 1 || y == height - 1 {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    let id_str = format!("{}", layer_id);
    let digit_scale = 8;
    let digit_w = 3 * digit_scale;
    let total_w = id_str.len() as u32 * (digit_w + 2 * digit_scale);
    let start_x = (width.saturating_sub(total_w)) / 2;
    let start_y = (height.saturating_sub(5 * digit_scale)) / 2;
    for (i, char) in id_str.chars().enumerate() {
        if let Some(digit) = char.to_digit(10) {
            draw_digit(
                &mut data,
                width,
                digit as usize,
                start_x + i as u32 * (digit_w + 2 * digit_scale),
                start_y,
                digit_scale,
                [0, 255, 255, 255],
            );
        }
    }
    data
}

const BITMAPS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

pub(crate) fn draw_digit(
    data: &mut [u8],
    width: u32,
    digit: usize,
    offset_x: u32,
    offset_y: u32,
    scale: u32,
    color: [u8; 4],
) {
    if digit > 9 {
        return;
    }
    let bitmap = BITMAPS[digit];
    for (row, row_bits) in bitmap.iter().enumerate() {
        for col in 0..3 {
            if (row_bits >> (2 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = offset_x + col as u32 * scale + dx;
                        let y = offset_y + row as u32 * scale + dy;
                        if x < width && y < (data.len() as u32 / width / 4) {
                            let idx = ((y * width + x) * 4) as usize;
                            data[idx] = color[0];
                            data[idx + 1] = color[1];
                            data[idx + 2] = color[2];
                            data[idx + 3] = color[3];
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn ensure_missing_texture_fallback(
    pool: &vorce_render::TexturePool,
    queue: &wgpu::Queue,
) {
    let fallback_name = "missing_texture_fallback";
    if !pool.has_texture(fallback_name) {
        let width = 64;
        let height = 64;
        let data = [0, 0, 0, 255].repeat((width * height) as usize);
        pool.ensure_texture(
            fallback_name,
            width,
            height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        pool.upload_data(queue, fallback_name, &data, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_grid_texture() {
        let tex = generate_grid_texture(100, 100, 1);
        assert_eq!(tex.len(), 40000);
    }

    #[test]
    fn test_draw_digit_bounds() {
        let mut data = vec![0; 400]; // 10x10
        draw_digit(&mut data, 10, 1, 0, 0, 1, [255, 255, 255, 255]);
        // Should not panic
    }
}
