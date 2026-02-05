# Hello World Projection Mapping Tutorial

Welcome to your first projection mapping example with MapFlow! This tutorial will guide you through creating a simple "Hello World" projection mapping application that demonstrates the core concepts of the MapFlow projection mapping system.

## Table of Contents

1. [What is Projection Mapping?](#what-is-projection-mapping)
2. [Prerequisites](#prerequisites)
3. [Understanding MapFlow Architecture](#understanding-mapflow-architecture)
4. [Building Your First Projection](#building-your-first-projection)
5. [Running the Example](#running-the-example)
6. [Understanding the Code](#understanding-the-code)
7. [Next Steps](#next-steps)

---

## What is Projection Mapping?

**Projection mapping** (also known as video mapping or spatial augmented reality) is a projection technology used to turn objects—often irregularly shaped—into display surfaces for video projection. By mapping digital content onto physical surfaces with precise geometric alignment, you can create stunning visual effects that transform ordinary objects into dynamic, interactive displays.

### Key Concepts

MapFlow uses a **Paint → Mapping → Mesh → Output** pipeline:

- **Paint**: A media source (video, image, test pattern, or solid color)
- **Mapping**: Connects a Paint to a Mesh with opacity, depth, and transforms
- **Mesh**: The warping geometry (quad, triangle, ellipse, or custom shape) with perspective correction
- **Output**: The display window(s) where the final composition is rendered

---

## Prerequisites

### System Requirements

- **Rust 1.75+** (install from [rustup.rs](https://rustup.rs))
- **Operating System**: Linux, macOS, or Windows
- **GPU**: Any modern GPU with Vulkan, Metal, or DirectX 12 support

### System Dependencies

Please refer to the main [BUILD.md](../01-GETTING-STARTED/BUILD.md) for detailed, up-to-date dependency lists for your platform.

### Clone the Repository

```bash
git clone https://github.com/johnjanik/mapmap.git
cd mapmap
```

---

## Understanding MapFlow Architecture

Before we dive into code, let's understand MapFlow's modular architecture:

### Crate Structure

MapFlow is organized as a Cargo workspace with specialized crates:

```
mapmap/
├── mapmap-core/      # Domain model (Paint/Mapping/Mesh)
├── mapmap-render/    # GPU rendering (wgpu backend)
├── mapmap-media/     # Video/image decoding (FFmpeg)
├── mapmap-ui/        # User interface (ImGui/egui)
├── mapmap-control/   # Control systems (MIDI/OSC/DMX)
├── mapmap-io/        # Professional I/O (NDI/DeckLink)
├── mapmap-ffi/       # Plugin API
└── mapmap/           # Main application binary
```

### Core Components

1. **WgpuBackend** (`mapmap-render`): GPU abstraction layer using wgpu
2. **Paint** (`mapmap-core`): Media source definition
3. **Mesh** (`mapmap-core`): Warping geometry
4. **Mapping** (`mapmap-core`): Connection between Paint and Mesh
5. **QuadRenderer** (`mapmap-render`): GPU renderer for textured quads

---

## Building Your First Projection

Let's create a simple "Hello World" projection mapping example that:

1. Creates a window
2. Loads a "Hello World" image or creates a colored texture
3. Projects it onto a quad mesh with basic warping

### Step 1: Create the Example File

Create a new file at `examples/hello_world_projection.rs`:

```rust
//! Hello World Projection Mapping Example
//!
//! This example demonstrates the basics of projection mapping:
//! 1. Creating a Paint (media source)
//! 2. Creating a Mesh (warping geometry)
//! 3. Creating a Mapping (connecting Paint to Mesh)
//! 4. Rendering the result

use mapmap_core::{Paint, PaintType, Mesh, MeshType, Mapping};
use mapmap_render::{QuadRenderer, TextureDescriptor, WgpuBackend};
use glam::Vec2;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    println!("MapMap - Hello World Projection Mapping Example");
    println!("===============================================\n");

    // Step 1: Create the window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("MapFlow - Hello World Projection")
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    println!("✓ Window created (1280x720)");

    // Step 2: Initialize GPU backend
    let mut backend = pollster::block_on(WgpuBackend::new()).unwrap();
    println!("✓ GPU Backend initialized");
    println!("  Adapter: {:?}", backend.adapter_info());

    // Step 3: Create surface for rendering
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: 1280,
        height: 720,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };

    surface.configure(backend.device(), &surface_config);
    println!("✓ Surface configured");

    // Step 4: Create quad renderer
    let quad_renderer = QuadRenderer::new(backend.device(), surface_config.format).unwrap();
    println!("✓ Quad renderer created");

    // Step 5: Create a Paint (media source)
    // For this example, we'll create a simple colored texture
    let paint = Paint::color(1, "Hello World Paint", [0.2, 0.6, 1.0, 1.0]); // Blue color
    println!("✓ Paint created: '{}'", paint.name);

    // Step 6: Create a Mesh (warping geometry)
    // We'll use a simple quad mesh
    let mesh = Mesh::new_quad(
        1, // mesh_id
        "Hello World Mesh",
        Vec2::new(0.0, 0.0),    // top-left
        Vec2::new(800.0, 0.0),  // top-right
        Vec2::new(800.0, 600.0), // bottom-right
        Vec2::new(0.0, 600.0),  // bottom-left
    );
    println!("✓ Mesh created: '{}'", mesh.name);

    // Step 7: Create a Mapping (connects Paint to Mesh)
    let mapping = Mapping::new(
        1,                // mapping_id
        "Hello World Mapping",
        paint.id,         // paint_id
        mesh.id,          // mesh_id
    );
    println!("✓ Mapping created: '{}'", mapping.name);
    println!("  Paint ID: {} → Mesh ID: {}", mapping.paint_id, mapping.mesh_id);

    // Step 8: Create GPU texture for the Paint
    let tex_desc = TextureDescriptor {
        width: 512,
        height: 512,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        mip_levels: 1,
    };

    let texture = backend.create_texture(tex_desc).unwrap();

    // Create a "Hello World" pattern
    // We'll create a simple gradient with the Paint's color
    let texture_data = create_hello_world_texture(512, 512, paint.color);
    backend.upload_texture(texture.clone(), &texture_data).unwrap();
    println!("✓ Texture uploaded (512x512)");

    println!("\n🎉 Setup complete! Rendering...\n");
    println!("Controls:");
    println!("  ESC - Exit");
    println!("  Any key - See the magic!\n");

    // Step 9: Render loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Goodbye! 👋");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                },
                ..
            } => {
                println!("Goodbye! 👋");
                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawRequested(_) => {
                // Get the current frame
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => return,
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Create command encoder
                let mut encoder = backend.device().create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    }
                );

                {
                    // Begin render pass
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Main Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

                    // Render the textured quad (our projection mapping!)
                    let texture_view = texture.create_view();
                    let bind_group = quad_renderer.create_bind_group(
                        backend.device(),
                        &texture_view
                    );
                    quad_renderer.draw(&mut render_pass, &bind_group);
                }

                // Submit commands and present
                backend.queue().submit(Some(encoder.finish()));
                frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

/// Creates a "Hello World" texture with a gradient pattern
fn create_hello_world_texture(width: u32, height: u32, base_color: [f32; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a radial gradient effect
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let max_distance = (center_x * center_x + center_y * center_y).sqrt();
            let gradient = 1.0 - (distance / max_distance).min(1.0);

            // Apply gradient to base color
            let r = (base_color[0] * gradient * 255.0) as u8;
            let g = (base_color[1] * gradient * 255.0) as u8;
            let b = (base_color[2] * gradient * 255.0) as u8;
            let a = (base_color[3] * 255.0) as u8;

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}
```

### Step 2: Add Dependencies

The example requires some additional dependencies. Add these to your `Cargo.toml` in the `[dev-dependencies]` section:

```toml
[dev-dependencies]
pollster = "0.3"
```

---

## Running the Example

### Build and Run

Then build and run the example from the mapmap crate directory:

```bash
# Navigate to the mapmap crate
cd crates/mapmap

# Build the example
cargo build --example hello_world_projection

# Run the example
cargo run --example hello_world_projection --release
```

### What You Should See

1. A window opens (1280x720)
2. A blue radial gradient is displayed in the center
3. Console output shows each initialization step

The gradient represents your "Hello World" projection - the Paint (media source) mapped onto a Mesh (geometry) and rendered to an Output (window).

### Controls

- **ESC** - Exit the application
- The window will continuously render the projection

---

## Understanding the Code

Let's break down what's happening in our Hello World example:

### 1. Window Creation

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new()
    /* ... */
    .build(&event_loop)
    .unwrap();
```

**Purpose**: Creates the display window using `winit`, a cross-platform windowing library.

### 2. GPU Backend Initialization

```rust
let mut backend = pollster::block_on(WgpuBackend::new()).unwrap();
```

**Purpose**: Initializes the GPU backend using `wgpu`, which provides a modern graphics API abstraction over Vulkan/Metal/DirectX 12.

### 3. Surface Configuration

```rust
let surface = unsafe { instance.create_surface(&window) }.unwrap();
let surface_config = wgpu::SurfaceConfiguration { /* ... */ };
```

**Purpose**: Creates a GPU surface attached to the window and configures its properties (size, format, present mode).

### 4. Create Renderer

```rust
let quad_renderer = QuadRenderer::new(backend.device(), surface_config.format).unwrap();
```

**Purpose**: Creates a specialized renderer for drawing textured quads with GPU acceleration.

### 5. Create Paint (Media Source)

```rust
let paint = Paint::color(1, "Hello World Paint", [0.2, 0.6, 1.0, 1.0]);
```

**Purpose**: Defines a media source - in this case, a solid blue color.

### 6. Create Mesh (Warping Geometry)

```rust
let mesh = Mesh::new_quad(1, "Hello World Mesh", /* ... */);
```

**Purpose**: Defines the geometry where the Paint will be projected. The quad's corner positions define how the texture is warped.

### 7. Create Mapping (Connect Paint to Mesh)

```rust
let mapping = Mapping::new(1, "Hello World Mapping", paint.id, mesh.id);
```

**Purpose**: Connects a Paint (media source) to a Mesh (geometry). This is the core of projection mapping.

### 8. Create Texture

```rust
let texture_data = create_hello_world_texture(512, 512, paint.color);
backend.upload_texture(texture.clone(), &texture_data).unwrap();
```

**Purpose**: Creates a GPU texture and uploads the pixel data.

### 9. Render Loop

```rust
event_loop.run(move |event, _, control_flow| {
    match event {
        Event::RedrawRequested(_) => {
            // 1. Get frame
            // 2. Create render pass
            // 3. Draw textured quad
            // 4. Submit and present
        }
        // ...
    }
});
```

**Purpose**: The main event loop that handles user input and renders frames continuously.

---

## Next Steps

Congratulations! You've created your first projection mapping with MapFlow. Here's what to explore next:

### 1. Load a Real Image or Video

Replace the colored Paint with an actual image or video (requires integrating `mapmap-media`).

### 2. Warp the Mesh

Try different mesh coordinates to create perspective distortion. This is the essence of projection mapping.

### 3. Explore the Full Application

Run the full MapFlow application to see all features: `cargo run --release`

### 4. Read the Documentation

Explore the comprehensive documentation in this `docs` folder.

---

**Happy Projection Mapping! 🎨🔦**
