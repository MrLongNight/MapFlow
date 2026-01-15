//! Window Manager
//!
//! This module handles the creation, tracking, and destruction of all application windows,
//! including the main control window and all output windows. It abstracts away the
//! complexities of managing winit windows and wgpu surfaces.

use anyhow::Result;
use mapmap_core::{OutputId, OutputManager};
use mapmap_render::WgpuBackend;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Fullscreen, Window, WindowBuilder, WindowId},
};

/// Context for a single window, containing the `winit` window, `wgpu` surface,
/// and other related configuration.
pub struct WindowContext {
    /// The `winit` window.
    pub window: Arc<Window>,
    /// The `wgpu` surface associated with the window.
    pub surface: wgpu::Surface<'static>,
    /// The configuration for the `wgpu` surface.
    pub surface_config: wgpu::SurfaceConfiguration,
    /// The `OutputId` associated with this window. For the main window, this is `0`.
    #[allow(dead_code)] // TODO: Prüfen, ob dieses Feld dauerhaft benötigt wird!
    pub output_id: OutputId,
}

/// Manages all application windows, including the main control window and all output windows.
pub struct WindowManager {
    windows: HashMap<OutputId, WindowContext>,
    window_id_map: HashMap<WindowId, OutputId>,
    main_window_id: Option<OutputId>,
}

impl WindowManager {
    /// Creates a new, empty `WindowManager`.
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            window_id_map: HashMap::new(),
            main_window_id: None,
        }
    }

    /// Creates the main control window.
    ///
    /// This is the primary window for the application, where the UI is displayed.
    /// It is assigned a reserved `OutputId` of `0`.
    #[allow(dead_code)] // Used for tests and as simple API wrapper
    pub fn create_main_window<T>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        backend: &WgpuBackend,
    ) -> Result<OutputId> {
        // Use default size
        self.create_main_window_with_geometry(event_loop, backend, None, None, None, None, false)
    }

    /// Creates the main control window with optional saved geometry.
    #[allow(clippy::too_many_arguments)]
    pub fn create_main_window_with_geometry<T>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        backend: &WgpuBackend,
        width: Option<u32>,
        height: Option<u32>,
        x: Option<i32>,
        y: Option<i32>,
        maximized: bool,
    ) -> Result<OutputId> {
        let default_width = width.unwrap_or(1920);
        let default_height = height.unwrap_or(1080);

        let mut window_builder = WindowBuilder::new()
            .with_title("MapFlow - Main Control")
            .with_window_icon(load_app_icon())
            .with_inner_size(winit::dpi::PhysicalSize::new(default_width, default_height))
            .with_maximized(maximized);

        // Set position if provided
        if let (Some(pos_x), Some(pos_y)) = (x, y) {
            window_builder =
                window_builder.with_position(winit::dpi::PhysicalPosition::new(pos_x, pos_y));
        }

        let window = Arc::new(window_builder.build(event_loop)?);

        // Re-apply icon explicitly to be sure
        if let Some(icon) = load_app_icon() {
            window.set_window_icon(Some(icon));
        }

        let window_id = window.id();
        let output_id: OutputId = 0; // Reserved ID for the main window

        let surface = backend.create_surface(window.clone())?;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: default_width,
            height: default_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&backend.device, &surface_config);

        let context = WindowContext {
            window,
            surface,
            surface_config,
            output_id,
        };

        self.windows.insert(output_id, context);
        self.window_id_map.insert(window_id, output_id);
        self.main_window_id = Some(output_id);

        Ok(output_id)
    }

    /// Creates a new output window based on an `OutputConfig`.
    ///
    /// If a window for the given `OutputId` already exists, this function does nothing.
    #[allow(dead_code)] // TODO: Prüfen, ob diese Funktion dauerhaft benötigt wird!
    pub fn create_output_window<T>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        backend: &WgpuBackend,
        output_config: &mapmap_core::OutputConfig,
    ) -> Result<()> {
        let output_id = output_config.id;

        // Skip if window already exists
        if self.windows.contains_key(&output_id) {
            return Ok(());
        }

        info!(
            "Creating window for output '{}' (ID: {})",
            output_config.name, output_id
        );

        let window = Arc::new(
            WindowBuilder::new()
                .with_title(format!("MapFlow Output - {}", output_config.name))
                .with_window_icon(load_app_icon())
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    output_config.resolution.0,
                    output_config.resolution.1,
                ))
                .with_fullscreen(if output_config.fullscreen {
                    Some(Fullscreen::Borderless(None))
                } else {
                    None
                })
                .build(event_loop)?,
        );

        // Re-apply icon explicitly to be sure
        if let Some(icon) = load_app_icon() {
            window.set_window_icon(Some(icon));
        }

        let window_id_winit = window.id();

        // Create surface for this output window
        let surface = backend.create_surface(window.clone())?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: backend.surface_format(),
            width: output_config.resolution.0,
            height: output_config.resolution.1,
            present_mode: wgpu::PresentMode::Fifo, // VSync for synchronized output
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&backend.device, &surface_config);

        let window_context = WindowContext {
            window,
            surface,
            surface_config,
            output_id,
        };

        self.windows.insert(output_id, window_context);
        self.window_id_map.insert(window_id_winit, output_id);

        info!(
            "Created output window for '{}' at {}x{}",
            output_config.name, output_config.resolution.0, output_config.resolution.1
        );

        Ok(())
    }

    /// Creates a new projector window from a Module OutputType::Projector.
    ///
    /// If a window for the given `output_id` already exists, this function does nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn create_projector_window<T>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        backend: &WgpuBackend,
        output_id: OutputId,
        name: &str,
        fullscreen: bool,
        hide_cursor: bool,
        target_screen: u8,
    ) -> Result<()> {
        // Skip if window already exists
        if self.windows.contains_key(&output_id) {
            return Ok(());
        }

        info!(
            "Creating projector window '{}' (ID: {}, Screen: {})",
            name, output_id, target_screen
        );

        // Get all available monitors
        let monitors: Vec<_> = event_loop.available_monitors().collect();

        // Select target monitor (default to primary if target_screen is out of range)
        let target_monitor = if (target_screen as usize) < monitors.len() {
            Some(monitors[target_screen as usize].clone())
        } else if let Some(primary) = event_loop.primary_monitor() {
            info!(
                "Target screen {} not found, using primary monitor",
                target_screen
            );
            Some(primary)
        } else {
            None
        };

        // Default resolution: 1920x1080 or monitor native resolution
        let (default_width, default_height) = if let Some(ref monitor) = target_monitor {
            if let Some(mode) = monitor.video_modes().next() {
                let size = mode.size();
                (size.width, size.height)
            } else {
                (1920, 1080)
            }
        } else {
            (1920, 1080)
        };

        let mut window_builder = WindowBuilder::new()
            .with_title(format!("MapFlow - {}", name))
            .with_window_icon(load_app_icon())
            .with_inner_size(winit::dpi::PhysicalSize::new(default_width, default_height));

        // Set fullscreen if requested
        if fullscreen {
            if let Some(monitor) = target_monitor.clone() {
                window_builder =
                    window_builder.with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            } else {
                window_builder = window_builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }
        }

        // If not fullscreen, set position on the target monitor
        if !fullscreen {
            if let Some(ref monitor) = target_monitor {
                window_builder = window_builder.with_position(monitor.position());
            }
        }

        // Build the window
        let window = Arc::new(window_builder.build(event_loop)?);

        // Re-apply icon explicitly to be sure
        if let Some(icon) = load_app_icon() {
            window.set_window_icon(Some(icon));
        }

        // Hide cursor if requested
        window.set_cursor_visible(!hide_cursor);

        let window_id_winit = window.id();

        // Create surface for this output window
        let surface = backend.create_surface(window.clone())?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: backend.surface_format(),
            width: default_width,
            height: default_height,
            present_mode: wgpu::PresentMode::Fifo, // VSync for synchronized output
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&backend.device, &surface_config);

        let window_context = WindowContext {
            window,
            surface,
            surface_config,
            output_id,
        };

        self.windows.insert(output_id, window_context);
        self.window_id_map.insert(window_id_winit, output_id);

        info!(
            "Created projector window '{}' at {}x{}",
            name, default_width, default_height
        );

        Ok(())
    }

    /// Synchronizes the active windows with the `OutputManager`'s configuration.
    ///
    /// This function will create windows for new outputs and remove windows for outputs
    /// that no longer exist.
    #[allow(dead_code)] // TODO: Prüfen, ob diese Funktion dauerhaft benötigt wird!
    pub fn sync_windows<T>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        backend: &WgpuBackend,
        output_manager: &OutputManager,
    ) -> Result<()> {
        // Create windows for new outputs
        for output_config in output_manager.outputs() {
            if !self.windows.contains_key(&output_config.id) {
                self.create_output_window(event_loop, backend, output_config)?;
            }
        }

        // Remove windows for outputs that no longer exist
        let output_ids: Vec<OutputId> = output_manager.outputs().iter().map(|o| o.id).collect();

        let mut windows_to_remove = Vec::new();
        for &window_output_id in self.windows.keys() {
            if window_output_id != 0 && !output_ids.contains(&window_output_id) {
                windows_to_remove.push(window_output_id);
            }
        }

        for output_id in windows_to_remove {
            self.remove_window(output_id);
            info!("Removed output window for output ID {}", output_id);
        }

        Ok(())
    }

    /// Removes a window by its `OutputId`.
    #[allow(dead_code)] // TODO: Prüfen, ob diese Funktion dauerhaft benötigt wird!
    pub fn remove_window(&mut self, output_id: OutputId) -> Option<WindowContext> {
        if let Some(context) = self.windows.remove(&output_id) {
            self.window_id_map.remove(&context.window.id());
            Some(context)
        } else {
            None
        }
    }

    /// Returns a reference to a `WindowContext` by its `OutputId`.
    pub fn get(&self, output_id: OutputId) -> Option<&WindowContext> {
        self.windows.get(&output_id)
    }

    /// Returns a mutable reference to a `WindowContext` by its `OutputId`.
    pub fn get_mut(&mut self, output_id: OutputId) -> Option<&mut WindowContext> {
        self.windows.get_mut(&output_id)
    }

    /// Returns the main window's `OutputId`.
    #[allow(dead_code)] // TODO: Prüfen, ob diese Funktion dauerhaft benötigt wird!
    pub fn main_window_id(&self) -> Option<OutputId> {
        self.main_window_id
    }

    /// Returns an iterator over all `OutputId`s.
    pub fn window_ids(&self) -> impl Iterator<Item = &OutputId> {
        self.windows.keys()
    }

    /// Returns an iterator over all `WindowContext`s.
    #[allow(dead_code)] // TODO: Prüfen, ob diese Funktion dauerhaft benötigt wird!
    pub fn iter(&self) -> impl Iterator<Item = &WindowContext> {
        self.windows.values()
    }

    /// Returns the `OutputId` for a given `winit` `WindowId`.
    pub fn get_output_id_from_window_id(&self, window_id: WindowId) -> Option<OutputId> {
        self.window_id_map.get(&window_id).copied()
    }

    /// Requests a redraw for all managed windows.
    ///
    /// This avoids the need for the caller to collect window IDs and iterate manually,
    /// preventing unnecessary allocations in the hot loop.
    pub fn request_redraw_all(&self) {
        for context in self.windows.values() {
            context.window.request_redraw();
        }
    }
}

/// Helper function to load the application icon.
fn load_app_icon() -> Option<winit::window::Icon> {
    let search_paths = [
        "resources/app_icons/MapFlow_Logo_HQ-Full-M.png",
        "../resources/app_icons/MapFlow_Logo_HQ-Full-M.png",
        "../../resources/app_icons/MapFlow_Logo_HQ-Full-M.png",
        "resources/app_icons/mapflow.png",
        "../resources/app_icons/mapflow.png",
    ];

    for path_str in search_paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            match image::open(path) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    tracing::info!("Found icon at {:?} ({}x{})", path, width, height);
                    match winit::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                        Ok(icon) => {
                            tracing::info!("Successfully created winit icon from {:?}", path);
                            return Some(icon);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to create winit icon from {:?}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to open icon {:?}: {}", path, e);
                }
            }
        }
    }
    None
}
