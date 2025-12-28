use anyhow::{Context, Result};
use mapmap_core::logging::LogConfig;
use std::fs::File;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

/// Handle to keep the logging worker thread alive
pub struct LogGuard {
    // Kept alive until dropped
    _guard: WorkerGuard,
}

/// Initialize the logging system
pub fn init(config: &LogConfig) -> Result<Option<LogGuard>> {
    // 1. Ensure log directory exists
    config
        .ensure_log_directory()
        .context("Failed to create log directory")?;

    // 2. Clean up old logs
    if let Err(e) = config.cleanup_old_logs() {
        eprintln!("Warning: Failed to cleanup old log files: {}", e);
    }

    // 3. Setup Filters
    // Parse level from config (defaulting to INFO if invalid)
    let config_filter = EnvFilter::builder()
        .with_default_directive(config.parse_level().into())
        .from_env_lossy(); // RUST_LOG env var takes precedence

    // 4. Console Layer
    let console_layer = if config.console_output {
        Some(
            fmt::layer()
                .with_writer(std::io::stderr) // Use stderr for logs, stdout for CLI output
                .with_ansi(true)
                .with_target(false) // Simpler output for console
                .with_filter(config_filter.clone()), // Clone filter or use same logic
        )
    } else {
        None
    };

    // 5. File Layer
    let (file_layer, guard) = if config.file_output {
        let log_path = config.current_log_path();

        let file = File::create(&log_path)
            .with_context(|| format!("Failed to create log file: {:?}", log_path))?;

        let (non_blocking, worker_guard) = tracing_appender::non_blocking(file);

        // Log setup success to stderr (before logging starts redirecting)
        eprintln!("Logging to file: {:?}", log_path);

        let layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false) // No colors in file
            .with_filter(config_filter);

        (
            Some(layer),
            Some(LogGuard {
                _guard: worker_guard,
            }),
        )
    } else {
        (None, None)
    };

    // 6. Initialize Registry
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    // Log startup info
    tracing::info!("Logging initialized at level: {}", config.level);
    tracing::info!("Log file path: {:?}", config.current_log_path());

    Ok(guard)
}
