//! Global logging and utility macros
//! 
//! Provides macros for rate-limited or one-time logging to prevent spam.

/// Log a warning only once per session for a given message or ID
#[macro_export]
macro_rules! warn_once {
    ($($arg:tt)+) => {
        use once_cell::sync::Lazy;
        use parking_lot::Mutex;
        use std::collections::HashSet;
        
        static REPORTED: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
        let msg = format!($($arg)+);
        let mut reported = REPORTED.lock();
        if reported.insert(msg.clone()) {
            tracing::warn!("{}", msg);
        }
    };
}

/// Log an info message only once per session
#[macro_export]
macro_rules! info_once {
    ($($arg:tt)+) => {
        use once_cell::sync::Lazy;
        use parking_lot::Mutex;
        use std::collections::HashSet;
        
        static REPORTED: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
        let msg = format!($($arg)+);
        let mut reported = REPORTED.lock();
        if reported.insert(msg.clone()) {
            tracing::info!("{}", msg);
        }
    };
}

/// Log an error message only once per session
#[macro_export]
macro_rules! error_once {
    ($($arg:tt)+) => {
        use once_cell::sync::Lazy;
        use parking_lot::Mutex;
        use std::collections::HashSet;
        
        static REPORTED: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
        let msg = format!($($arg)+);
        let mut reported = REPORTED.lock();
        if reported.insert(msg.clone()) {
            tracing::error!("{}", msg);
        }
    };
}
