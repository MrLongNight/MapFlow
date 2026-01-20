## 2026-01-02 - Missing CSP on API/WS Server
**Vulnerability:** The web server component in `mapmap-control` (serving API and WebSocket) was missing a `Content-Security-Policy` header. While primarily an API server, the absence of CSP meant that if a browser were to render a response as HTML (e.g., via a misconfigured error page or direct navigation), it could execute arbitrary scripts.
**Learning:** Even API-centric servers should enforce CSP. `default-src 'none'` is a robust default for APIs that prevents any unauthorized resource loading or script execution, providing a strong defense-in-depth layer against potential XSS vectors.
**Prevention:** Always include `Content-Security-Policy: default-src 'none'; frame-ancestors 'none';` for API servers. Use `security_headers` middleware to enforce this globally.

## 2026-01-23 - Insecure Default Bind Address
**Vulnerability:** The web server defaulted to binding to `0.0.0.0` (all interfaces), exposing the unauthenticated control API to the local network (and potentially public internet) without user awareness.
**Learning:** Secure defaults are critical. Development conveniences (like "it just works from my phone") should never compromise security. Users must explicitly opt-in to network exposure.
**Prevention:** Always default server bind addresses to `127.0.0.1`. Update `Default` trait implementations for configuration structs to reflect this.

## 2026-01-26 - Plaintext API Keys in Config
**Vulnerability:** API keys were stored in plain text within the `AuthConfig` struct and serialized to configuration files. This exposed credentials to anyone with read access to the config or memory dumps.
**Learning:** Configuration structs are often serialized directly. Adding security layers (like hashing) requires careful handling of serialization to maintain backward compatibility with legacy plaintext data. A custom deserializer can intelligently migrate legacy data.
**Prevention:** Use SHA-256 hashing for storage of all secrets. Implement `deserialize_with` for `serde` to handle the migration from plaintext to hash transparently on load.

## 2026-02-14 - Implicit Wildcard Logic in Security Config
**Vulnerability:** The web server configuration treated an empty `allowed_origins` list as "Allow All" (`*`), assuming empty meant "unconfigured/default permissive". This violated the principle of least privilege, where empty should imply "Allow None".
**Learning:** Logic that attempts to be "helpful" by falling back to permissive defaults when configuration is missing/empty is a major security trap. Explicit configuration should always be required for permissive security settings.
**Prevention:** Remove fallback logic that maps empty/default values to insecure states. Ensure empty collections in security configs fail closed (deny all) rather than open (allow all).
