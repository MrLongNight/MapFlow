
## 2024-05-24 - Cross-Platform Path Traversal Bypass in Path::components()
**Vulnerability:** Windows-style path traversal payloads (`..\..\secret`) bypass `std::path::Path::components()` validation when executed on non-Windows systems (like Linux/macOS servers) because backslashes are not treated as path separators.
**Learning:** Rust's `Path` behaves according to the target OS. When validating input that might originate from or target a different OS, you cannot rely solely on native path parsing.
**Prevention:** Always normalize path separators (e.g., `.replace("\\", "/")`) before using `Path::components()` for security validation, or manually scan for both `/../` and `\..\`.
## 2024-05-24 - Missing Target Validation in apply_control
**Vulnerability:** The `apply_control` method validated the `ControlValue` for path traversal, but failed to call `target.validate()` on the `ControlTarget`, allowing potentially malicious target names with path traversals or control characters to bypass validation.
**Learning:** Validation methods on complex types must be explicitly called at the boundary where they are ingested or applied; do not assume they are implicitly validated.
**Prevention:** Always trace the data flow of user-controlled complex types (like enums with string variants) and ensure their inherent `.validate()` methods are executed before the data is used.
