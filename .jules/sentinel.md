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

## 2026-05-24 - Path Traversal in MCP Server
**Vulnerability:** The Model Context Protocol (MCP) server allowed `project_save` and `project_load` commands to access arbitrary file paths (e.g., `../evil.txt`), potentially allowing an automated agent (or attacker) to overwrite sensitive system files or exfiltrate data.
**Learning:** Agent interfaces that expose file system operations must be sandboxed. Standard `PathBuf` handling does not automatically prevent traversal (`..`).
**Prevention:** Explicitly validate all user-supplied paths in agent tools. Reject paths containing `ParentDir` (`..`) components and enforce relative paths or specific sandboxed roots.

## 2026-10-24 - Insecure Logic for Empty Allowed Origins
**Vulnerability:** The web server treated an empty list of `allowed_origins` as "Allow All" (wildcard), intended as a permissive default. This meant that configurations intending to restrict access (by providing an empty list) inadvertently opened the API to all origins.
**Learning:** "Empty means None" is the standard semantic expectation for security allowlists. "Empty means All" is a dangerous anti-pattern that leads to accidental exposure.
**Prevention:** Treat empty allowlists as "Deny All". Require explicit `*` or `Any` markers for permissive modes. Ensure secure defaults (empty/deny) in configuration structs.

## 2026-10-27 - Unrestricted File Write in MCP Tools
**Vulnerability:** The MCP `project_save` tool validated path traversal but allowed writing to any file extension (e.g., `src/lib.rs`). This allowed an agent to overwrite source code or configuration files within the allowed directory.
**Learning:** Path traversal protection is insufficient for file write operations. Restricting file types by extension is crucial to prevent code injection or configuration corruption.
**Prevention:** Whitelist allowed file extensions (e.g., `.mapmap`, `.json`) for all file write operations exposed to agents.
