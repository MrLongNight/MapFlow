use crate::cli::CliArgs;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct NdiPlayerApp {
    source: String,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for NdiPlayerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!("Initializing NDI Player window...");
            let attr = Window::default_attributes()
                .with_title(format!("MapFlow NDI Player: {}", self.source));
            if let Ok(window) = event_loop.create_window(attr) {
                self.window = Some(Arc::new(window));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        }
    }
}

/// Starts the NDI Player mode.
pub fn run(args: &CliArgs) -> Result<()> {
    info!("Starting NDI Player mode for source: {}", args.source);

    let event_loop = EventLoop::new()?;
    let mut app_handler = NdiPlayerApp {
        source: args.source.clone(),
        window: None,
    };

    event_loop.run_app(&mut app_handler)?;

    Ok(())
}
