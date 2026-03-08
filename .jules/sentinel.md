## $(date +%Y-%m-%d) - [Path Traversal bypass using Windows backslashes]
**Vulnerability:** Windows-style path traversal (`..\`) payloads were bypassing validation on non-Windows systems because Rust's `std::path::Path` does not recognize `\` as a directory separator on Unix-like operating systems.
**Learning:** When validating paths for security (like path traversal), always normalize path separators (`\` to `/`) before passing them to OS-dependent path parsing functions to ensure cross-platform payloads are correctly identified.
**Prevention:** Normalize backslashes to forward slashes before any security validation that relies on path component extraction (`Path::components()`).

## 2026-03-08 - Path Traversal (Windows compatibility on Unix)
**Vulnerability:** Windows-style traversal payloads `..\` bypass naive security checks on Unix endpoints because `std::path::Path` does not evaluate `\` as a directory break.
**Learning:** Normalizing user inputs (converting `\` to `/`) before any struct/library-based path evaluation correctly parses `..\` into a unified schema for traversal checks.
**Prevention:** Handlers now normalize `ControlValue::String` through `.replace('\\', "/")` prior to iterating `components()` to flag `Component::ParentDir`.
