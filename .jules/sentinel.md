## 2024-05-24 - [DoS] Limit WebSocket Batch Operations
**Vulnerability:** The WebSocket handler allowed clients to send an unlimited number of subscription/unsubscription targets in a single message, potentially causing resource exhaustion (DoS) even if the message size was within limits.
**Learning:** Limiting message size (bytes) is not enough; semantic limits (item count) are also necessary for complex operations.
**Prevention:** Implemented `MAX_BATCH_SIZE` constant and enforced it in `Subscribe` and `Unsubscribe` handlers.
## 2024-05-24 - [Path Traversal Validation]
**Vulnerability:** Path traversal detection logic was manually implemented using multiple string matching conditions (`..`, `../`, `..\\`, etc.), which is prone to bypasses.
**Learning:** `std::path::Path` with `components().any(|c| matches!(c, std::path::Component::ParentDir))` correctly handles directory traversal sequences including mixed separators on Windows vs Unix.
**Prevention:** In Rust applications, use `std::path::Path` and `std::path::Component` logic over manual string manipulation for robust path traversal detection to cover all edge cases across operating systems.
