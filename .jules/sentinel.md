<<<<<<< HEAD
## 2024-05-24 - [DoS] Limit WebSocket Batch Operations
**Vulnerability:** The WebSocket handler allowed clients to send an unlimited number of subscription/unsubscription targets in a single message, potentially causing resource exhaustion (DoS) even if the message size was within limits.
**Learning:** Limiting message size (bytes) is not enough; semantic limits (item count) are also necessary for complex operations.
**Prevention:** Implemented `MAX_BATCH_SIZE` constant and enforced it in `Subscribe` and `Unsubscribe` handlers.
=======
## $(date +%Y-%m-%d) - [Path Traversal bypass using Windows backslashes]
**Vulnerability:** Windows-style path traversal (`..\`) payloads were bypassing validation on non-Windows systems because Rust's `std::path::Path` does not recognize `\` as a directory separator on Unix-like operating systems.
**Learning:** When validating paths for security (like path traversal), always normalize path separators (`\` to `/`) before passing them to OS-dependent path parsing functions to ensure cross-platform payloads are correctly identified.
**Prevention:** Normalize backslashes to forward slashes before any security validation that relies on path component extraction (`Path::components()`).

## 2026-03-08 - Path Traversal (Windows compatibility on Unix)
**Vulnerability:** Windows-style traversal payloads `..\` bypass naive security checks on Unix endpoints because `std::path::Path` does not evaluate `\` as a directory break.
**Learning:** Normalizing user inputs (converting `\` to `/`) before any struct/library-based path evaluation correctly parses `..\` into a unified schema for traversal checks.
**Prevention:** Handlers now normalize `ControlValue::String` through `.replace('\\', "/")` prior to iterating `components()` to flag `Component::ParentDir`.
>>>>>>> origin/jules-mf-048-core-repair-2290194584907283660

## 2024-05-24 - Windows Path Traversal Bypass
**Vulnerability:** Path traversal payload using Windows backslashes (`..\\..\\etc\\passwd`) bypassed `ControlValue::String` validation logic because `std::path::Path::components()` behaves differently depending on host OS. On Linux/macOS, it does not recognize `\` as a directory separator, so `Component::ParentDir` was not matched.
**Learning:** Security validations relying on standard library `Path` parsing are OS-dependent. Validating paths securely requires explicitly normalizing separators or using custom parsing when handling payloads originating from untrusted, potentially cross-platform clients.
**Prevention:** Normalize all incoming string paths (e.g., replacing `\\` with `/`) *before* relying on standard OS-specific path parsing utilities like `Path::new()` for path traversal checks.
