## $(date +%Y-%m-%d) - [Path Traversal bypass using Windows backslashes]
**Vulnerability:** Windows-style path traversal (`..\`) payloads were bypassing validation on non-Windows systems because Rust's `std::path::Path` does not recognize `\` as a directory separator on Unix-like operating systems.
**Learning:** When validating paths for security (like path traversal), always normalize path separators (`\` to `/`) before passing them to OS-dependent path parsing functions to ensure cross-platform payloads are correctly identified.
**Prevention:** Normalize backslashes to forward slashes before any security validation that relies on path component extraction (`Path::components()`).
