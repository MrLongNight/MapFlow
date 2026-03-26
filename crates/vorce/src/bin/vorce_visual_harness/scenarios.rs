use clap::ValueEnum;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ScenarioName {
    Checkerboard,
    Gradient,
    #[value(name = "alpha_overlay")]
    AlphaOverlay,
}

pub struct ScenarioSpec {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub source_pixels: Vec<u8>,
    pub expected_pixels: Vec<u8>,
}

pub fn build_scenario(name: ScenarioName) -> ScenarioSpec {
    match name {
        ScenarioName::Checkerboard => checkerboard(),
        ScenarioName::Gradient => gradient(),
        ScenarioName::AlphaOverlay => alpha_overlay(),
    }
}

fn checkerboard() -> ScenarioSpec {
    let mut pixels = blank_rgba(WIDTH, HEIGHT, [0, 0, 0, 255]);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let is_even_cell = ((x / 32) + (y / 32)) % 2 == 0;
            let base = if is_even_cell {
                [28, 179, 124, 255]
            } else {
                [210, 58, 110, 255]
            };
            set_pixel(&mut pixels, WIDTH, x, y, base);
        }
    }

    fill_rect(&mut pixels, WIDTH, 0, 0, 24, 24, [255, 255, 255, 255]);
    fill_rect(
        &mut pixels,
        WIDTH,
        WIDTH - 24,
        HEIGHT - 24,
        24,
        24,
        [0, 0, 0, 255],
    );
    fill_rect(
        &mut pixels,
        WIDTH,
        0,
        HEIGHT - 16,
        WIDTH,
        16,
        [0, 76, 190, 255],
    );
    fill_rect(
        &mut pixels,
        WIDTH,
        WIDTH - 16,
        0,
        16,
        HEIGHT,
        [255, 190, 32, 255],
    );

    ScenarioSpec {
        title: "Vorce Visual Harness - Checkerboard",
        width: WIDTH,
        height: HEIGHT,
        source_pixels: pixels.clone(),
        expected_pixels: pixels,
    }
}

fn gradient() -> ScenarioSpec {
    let mut pixels = blank_rgba(WIDTH, HEIGHT, [0, 0, 0, 255]);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let r = x as u8;
            let g = y as u8;
            let b = (((x + y) / 2) as u8).saturating_add(32);
            set_pixel(&mut pixels, WIDTH, x, y, [r, g, b, 255]);
        }
    }

    fill_rect(&mut pixels, WIDTH, 24, 24, 32, 128, [255, 255, 255, 255]);
    fill_rect(&mut pixels, WIDTH, 96, 176, 128, 24, [255, 64, 0, 255]);

    ScenarioSpec {
        title: "Vorce Visual Harness - Gradient",
        width: WIDTH,
        height: HEIGHT,
        source_pixels: pixels.clone(),
        expected_pixels: pixels,
    }
}

fn alpha_overlay() -> ScenarioSpec {
    let mut source = blank_rgba(WIDTH, HEIGHT, [0, 0, 0, 0]);

    fill_rect(
        &mut source,
        WIDTH,
        20,
        20,
        WIDTH - 40,
        HEIGHT - 40,
        [36, 96, 224, 255],
    );
    fill_rect(&mut source, WIDTH, 56, 56, 144, 144, [255, 64, 64, 128]);
    fill_rect(
        &mut source,
        WIDTH,
        0,
        HEIGHT - 32,
        WIDTH,
        12,
        [255, 255, 0, 192],
    );

    let expected = blend_over_black(&source);

    ScenarioSpec {
        title: "Vorce Visual Harness - Alpha Overlay",
        width: WIDTH,
        height: HEIGHT,
        source_pixels: source,
        expected_pixels: expected,
    }
}

fn blank_rgba(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let mut pixels = vec![0; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            set_pixel(&mut pixels, width, x, y, color);
        }
    }
    pixels
}

fn fill_rect(
    pixels: &mut [u8],
    width: u32,
    start_x: u32,
    start_y: u32,
    rect_width: u32,
    rect_height: u32,
    color: [u8; 4],
) {
    let max_x = (start_x + rect_width).min(width);
    let height = pixels.len() as u32 / (width * 4);
    let max_y = (start_y + rect_height).min(height);

    for y in start_y..max_y {
        for x in start_x..max_x {
            set_pixel(pixels, width, x, y, color);
        }
    }
}

fn set_pixel(pixels: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * width + x) * 4) as usize;
    pixels[idx..idx + 4].copy_from_slice(&color);
}

fn blend_over_black(source: &[u8]) -> Vec<u8> {
    let mut blended = vec![0; source.len()];
    for (src, dst) in source.chunks_exact(4).zip(blended.chunks_exact_mut(4)) {
        let alpha = src[3] as u16;
        dst[0] = ((src[0] as u16 * alpha) / 255) as u8;
        dst[1] = ((src[1] as u16 * alpha) / 255) as u8;
        dst[2] = ((src[2] as u16 * alpha) / 255) as u8;
        dst[3] = 255;
    }
    blended
}
