## 2024-05-24 - [DoS] Limit WebSocket Batch Operations
**Vulnerability:** The WebSocket handler allowed clients to send an unlimited number of subscription/unsubscription targets in a single message, potentially causing resource exhaustion (DoS) even if the message size was within limits.
**Learning:** Limiting message size (bytes) is not enough; semantic limits (item count) are also necessary for complex operations.
**Prevention:** Implemented `MAX_BATCH_SIZE` constant and enforced it in `Subscribe` and `Unsubscribe` handlers.

## 2026-03-04 - [Information Disclosure] Hardcoded Absolute Developer Paths
**Vulnerability:** Several fallback paths for loading assets and configurations were using hardcoded absolute developer paths (e.g., `C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\...`).
**Learning:** Hardcoding absolute paths from a developer's machine can leak sensitive information about the developer's environment (username, directory structure) to anyone analyzing the source code or binaries. This is an Information Disclosure vulnerability.
**Prevention:** Always rely strictly on relative paths (e.g., `resources/...` or `../resources/...`) when defining fallback paths for loading resources or configs.
