pub const VIDEO_LOG_THROTTLE: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) fn should_log_video_issue(
    log_times: &mut std::collections::HashMap<String, std::time::Instant>,
    key: impl Into<String>,
) -> bool {
    let key = key.into();
    let now = std::time::Instant::now();
    match log_times.get(&key) {
        Some(last_logged) if now.duration_since(*last_logged) < VIDEO_LOG_THROTTLE => false,
        _ => {
            log_times.insert(key, now);
            true
        }
    }
}

pub(crate) fn clear_video_issue(
    log_times: &mut std::collections::HashMap<String, std::time::Instant>,
    key: impl AsRef<str>,
) {
    log_times.remove(key.as_ref());
}
