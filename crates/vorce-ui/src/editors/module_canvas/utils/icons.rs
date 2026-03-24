use egui::TextureHandle;
use vorce_core::runtime_paths;

pub fn ensure_icons_loaded(
    plug_icons: &mut std::collections::HashMap<String, TextureHandle>,
    ctx: &egui::Context,
) {
    if !plug_icons.is_empty() {
        return;
    }

    let paths = [runtime_paths::resource_path("stecker_icons")];

    let files = [
        "audio-jack1.1.svg",
        "audio-jack_1.2.svg",
        "audio-jack_2.svg",
        "plug.svg",
        "power-plug.svg",
        "usb-cable.svg",
    ];

    for path_str in paths {
        let base_path = path_str.as_path();
        if base_path.exists() {
            for file in files {
                let path = base_path.join(file);
                if let Some(texture) = load_svg_icon(&path, ctx) {
                    plug_icons.insert(file.to_string(), texture);
                }
            }
            if !plug_icons.is_empty() {
                break;
            }
        }
    }
}

fn load_svg_icon(path: &std::path::Path, ctx: &egui::Context) -> Option<TextureHandle> {
    let svg_data = std::fs::read(path).ok()?;
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options).ok()?;
    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let mut pixels = Vec::with_capacity((width * height) as usize);
    for pixel in pixmap.pixels() {
        // Preserve original RGBA from SVG
        pixels.push(egui::Color32::from_rgba_premultiplied(
            pixel.red(),
            pixel.green(),
            pixel.blue(),
            pixel.alpha(),
        ));
    }

    let image = egui::ColorImage {
        size: [width as usize, height as usize],
        pixels,
        source_size: egui::Vec2::new(width as f32, height as f32),
    };

    Some(ctx.load_texture(
        path.file_name()?.to_string_lossy(),
        image,
        egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
            mipmap_mode: None,
        },
    ))
}
