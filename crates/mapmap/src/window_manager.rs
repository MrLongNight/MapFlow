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
            .with_inner_size(winit::dpi::PhysicalSize::new(default_width, default_height))
            .with_maximized(maximized);

        // Set position if provided
        if let (Some(pos_x), Some(pos_y)) = (x, y) {
            window_builder =
                window_builder.with_position(winit::dpi::PhysicalPosition::new(pos_x, pos_y));
        }

        let window = Arc::new(window_builder.build(event_loop)?);

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
        _monitor_topology: &mapmap_core::monitor::MonitorTopology,
    ) -> Result<()> {
        let output_id = output_config.id;

        // Skip if window already exists
        if self.windows.contains_key(&output_id) {
            return Ok(());
        }

        // Only create if enabled
        if !output_config.enabled {
            return Ok(());
        }

        info!(
            "Creating window for output '{}' (ID: {})",
            output_config.name, output_id
        );

        // Find target monitor
        let target_monitor = if let Some(monitor_name) = &output_config.monitor_name {
            event_loop.available_monitors().find(|m| {
                m.name().map_or(false, |n| &n == monitor_name)
            })
        } else {
            None
        };

        let mut builder = WindowBuilder::new()
            .with_title(format!("MapFlow Output - {}", output_config.name))
            .with_inner_size(winit::dpi::PhysicalSize::new(
                output_config.resolution.0,
                output_config.resolution.1,
            ));

        if let Some(monitor) = target_monitor {
            builder = builder.with_fullscreen(if output_config.fullscreen {
                 Some(Fullscreen::Borderless(Some(monitor)))
            } else {
                // If not fullscreen, move to monitor position (requires position info which winit handles via build usually for fullscreen, but for windowed...)
                // Actually winit doesn't easily let us position windowed window on specific monitor at creation cross-platform without position
                // But we can just use Borderless(None) for current or position if known.
                // For now, if monitor is specified, we try to use it for fullscreen.
                None
            });
            // If not fullscreen, we ideally want to position it on that monitor.
            // But we don't have stored positions in config yet.
            // We can infer from topology if we had it mapped to winit handles.
            // For now, just fullscreen uses the monitor handle.
        } else {
             builder = builder.with_fullscreen(if output_config.fullscreen {
                 Some(Fullscreen::Borderless(None))
            } else {
                None
            });
        }

        // Refined fullscreen logic:
        // If monitor is specified:
        //   - Fullscreen: Use that monitor
        //   - Windowed: Try to position it there (hard without mapping monitor info to coordinates perfectly matching winit, but we can try)
        // For now, minimal implementation:

        let window = Arc::new(builder.build(event_loop)?);

        let window_id_winit = window.id();

        // Create surface for this output window
        let surface = backend.create_surface(window.clone())?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
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
        monitor_topology: &mapmap_core::monitor::MonitorTopology,
    ) -> Result<()> {
        // Create windows for new/modified outputs
        for output_config in output_manager.outputs() {
            if output_config.enabled {
                if !self.windows.contains_key(&output_config.id) {
                    self.create_output_window(event_loop, backend, output_config, monitor_topology)?;
                } else {
                    // Check if properties changed and recreate if needed
                    if let Some(window_context) = self.windows.get(&output_config.id) {
                        let current_fullscreen = window_context.window.fullscreen().is_some();
                        let target_fullscreen = output_config.fullscreen;

                        // Check if monitor changed
                        // Since we don't store current monitor name in WindowContext, we assume if config changed
                        // and we are in fullscreen or moving to fullscreen, we might need update.
                        // For simplicity/robustness: if monitor name in config implies a change (e.g. was None, now Some),
                        // and we are fullscreen, we should update.
                        // But set_fullscreen handles monitor change if we pass the new monitor handle!

                        if current_fullscreen != target_fullscreen {
                            let new_fullscreen = if target_fullscreen {
                                // Find target monitor if specified
                                let target_monitor = if let Some(monitor_name) = &output_config.monitor_name {
                                    event_loop.available_monitors().find(|m| {
                                        m.name().map_or(false, |n| &n == monitor_name)
                                    })
                                } else {
                                    None
                                };
                                Some(Fullscreen::Borderless(target_monitor))
                            } else {
                                None
                            };

                            info!(
                                "Updating fullscreen state for output {} to {}",
                                output_config.id, target_fullscreen
                            );
                            window_context.window.set_fullscreen(new_fullscreen);
                        } else if target_fullscreen {
                            // If already fullscreen, but maybe monitor changed?
                            // We should re-apply fullscreen with potentially new monitor.
                            // This is cheap if nothing changed.
                            let target_monitor = if let Some(monitor_name) = &output_config.monitor_name {
                                event_loop.available_monitors().find(|m| {
                                    m.name().map_or(false, |n| &n == monitor_name)
                                })
                            } else {
                                None
                            };
                            window_context.window.set_fullscreen(Some(Fullscreen::Borderless(target_monitor)));
                        }
                    }
                }
            } else {
                // If disabled but exists, remove it
                if self.windows.contains_key(&output_config.id) {
                    self.remove_window(output_config.id);
                }
            }
        }

        // Remove windows for outputs that no longer exist in config
        let active_output_ids: Vec<OutputId> = output_manager.outputs().iter().map(|o| o.id).collect();

        let mut windows_to_remove = Vec::new();
        for &window_output_id in self.windows.keys() {
            if window_output_id != 0 && !active_output_ids.contains(&window_output_id) {
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
}
