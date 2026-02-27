## 2024-05-24 - [DoS] Limit WebSocket Batch Operations
**Vulnerability:** The WebSocket handler allowed clients to send an unlimited number of subscription/unsubscription targets in a single message, potentially causing resource exhaustion (DoS) even if the message size was within limits.
**Learning:** Limiting message size (bytes) is not enough; semantic limits (item count) are also necessary for complex operations.
**Prevention:** Implemented `MAX_BATCH_SIZE` constant and enforced it in `Subscribe` and `Unsubscribe` handlers.
