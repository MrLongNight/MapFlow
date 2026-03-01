//! Global logging and utility macros
//!
//! Provides macros for rate-limited or one-time logging to prevent spam.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;

/// Global registry for reported log messages to prevent spam
pub static LOG_REGISTRY: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Internal function to check if a message should be logged
#[doc(hidden)]
pub fn should_log(msg: &str) -> bool {
    let mut registry = LOG_REGISTRY.lock();
    registry.insert(msg.to_string())
}

/// Log a warning only once per session for a given message or ID
#[macro_export]
macro_rules! warn_once {
    ($($arg:tt)+) => {
        let msg = format!($($arg)+);
        if $crate::macros::should_log(&msg) {
            tracing::warn!("{}", msg);
        }
    };
}

/// Log an info message only once per session
#[macro_export]
macro_rules! info_once {
    ($($arg:tt)+) => {
        let msg = format!($($arg)+);
        if $crate::macros::should_log(&msg) {
            tracing::info!("{}", msg);
        }
    };
}

/// Log an error message only once per session
#[macro_export]
macro_rules! error_once {
    ($($arg:tt)+) => {
        let msg = format!($($arg)+);
        if $crate::macros::should_log(&msg) {
            tracing::error!("{}", msg);
        }
    };
}
