//! Logging configuration and file rotation management
//!
//! Provides configurable logging with file output, rotation, and console output.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::Level;

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogConfig {
    /// Log level: "trace", "debug", "info", "warn", "error"
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Directory for log files
    #[serde(default = "default_log_path")]
    pub log_path: PathBuf,

    /// Maximum number of log files to keep
    #[serde(default = "default_max_files")]
    pub max_files: usize,

    /// Enable console output
    #[serde(default = "default_console_output")]
    pub console_output: bool,

    /// Enable file output
    #[serde(default = "default_file_output")]
    pub file_output: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_path() -> PathBuf {
    PathBuf::from("logs")
}

fn default_max_files() -> usize {
    10
}

fn default_console_output() -> bool {
    true
}

fn default_file_output() -> bool {
    true
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            log_path: default_log_path(),
            max_files: default_max_files(),
            console_output: default_console_output(),
            file_output: default_file_output(),
        }
    }
}

impl LogConfig {
    /// Parse log level string to tracing Level
    pub fn parse_level(&self) -> Level {
        match self.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }

    /// Generate a new log filename with timestamp
    pub fn generate_log_filename() -> String {
        let now = chrono::Local::now();
        format!("mapflow_{}.log", now.format("%Y-%m-%d_%H-%M-%S"))
    }

    /// Get the full path for the current log file
    pub fn current_log_path(&self) -> PathBuf {
        self.log_path.join(Self::generate_log_filename())
    }

    /// Ensure the log directory exists
    pub fn ensure_log_directory(&self) -> std::io::Result<()> {
        if !self.log_path.exists() {
            fs::create_dir_all(&self.log_path)?;
        }
        Ok(())
    }

    /// Clean up old log files, keeping only the most recent `max_files`
    pub fn cleanup_old_logs(&self) -> std::io::Result<()> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let mut log_files: Vec<_> = fs::read_dir(&self.log_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "log")
                    .unwrap_or(false)
            })
            .collect();

        if log_files.len() <= self.max_files {
            return Ok(());
        }

        // Sort by modification time (oldest first)
        log_files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            a_time.cmp(&b_time)
        });

        // Remove oldest files
        let files_to_remove = log_files.len() - self.max_files;
        for entry in log_files.into_iter().take(files_to_remove) {
            if let Err(e) = fs::remove_file(entry.path()) {
                tracing::warn!("Failed to remove old log file {:?}: {}", entry.path(), e);
            } else {
                tracing::debug!("Removed old log file: {:?}", entry.path());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.max_files, 10);
        assert!(config.console_output);
        assert!(config.file_output);
    }

    #[test]
    fn test_parse_level() {
        let mut config = LogConfig {
            level: "debug".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level(), Level::DEBUG);

        config.level = "WARN".to_string();
        assert_eq!(config.parse_level(), Level::WARN);

        config.level = "invalid".to_string();
        assert_eq!(config.parse_level(), Level::INFO);
    }

    #[test]
    fn test_log_filename_format() {
        let filename = LogConfig::generate_log_filename();
        assert!(filename.starts_with("mapflow_"));
        assert!(filename.ends_with(".log"));
    }

    #[test]
    fn test_current_log_path() {
        let config = LogConfig {
            log_path: PathBuf::from("my_logs"),
            ..Default::default()
        };
        let path = config.current_log_path();
        assert!(path.starts_with("my_logs"));
        assert!(path.to_string_lossy().contains("mapflow_"));
        assert!(path.extension().unwrap() == "log");
    }

    #[test]
    fn test_ensure_log_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().join("test_logs");

        let config = LogConfig {
            log_path: log_dir.clone(),
            ..Default::default()
        };

        config.ensure_log_directory().unwrap();
        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn test_cleanup_old_logs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().join("cleanup_logs");
        fs::create_dir_all(&log_dir).unwrap();

        let file1 = log_dir.join("mapflow_1.log");
        let file2 = log_dir.join("mapflow_2.log");
        let file3 = log_dir.join("mapflow_3.log");

        fs::write(&file1, "log 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&file2, "log 2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&file3, "log 3").unwrap();

        let non_log = log_dir.join("ignore.txt");
        fs::write(&non_log, "ignore me").unwrap();

        let config = LogConfig {
            log_path: log_dir.clone(),
            max_files: 2,
            ..Default::default()
        };

        config.cleanup_old_logs().unwrap();

        assert!(!file1.exists());
        assert!(file2.exists());
        assert!(file3.exists());
        assert!(non_log.exists());
    }
}
