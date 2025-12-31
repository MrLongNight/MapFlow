## 2024-05-23 - Missing Authentication Middleware in Axum
**Vulnerability:** The `mapmap-control` crate implemented API key validation logic (`extract_api_key`, `AuthConfig::validate`) but never actually used it to protect the API routes. The `extract_api_key` function was unused, meaning all API endpoints were publicly accessible without authentication, regardless of configuration.
**Learning:** Having authentication helpers and configuration structs is not enough; one must explicitly verify that the protection mechanism (middleware) is applied to the router. The disconnect happened because `auth.rs` was isolated from `server.rs`/`routes.rs` and no integration test verified access control.
**Prevention:**
1. Always write integration tests that attempt to access protected endpoints *without* credentials and assert that they fail (401 Unauthorized).
2. When using frameworks like Axum, ensure middleware is applied globally or per-route using `.layer()` or `.route_layer()`.
3. Use compiler warnings (dead code) to catch unused security functions - if `extract_api_key` had triggered a dead code warning (it might have been suppressed or public), this would have been caught earlier.
