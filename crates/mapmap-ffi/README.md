# mapmap-ffi

**MapFlow C/C++ Foreign Function Interface.**

`mapmap-ffi` provides C-compatible bindings for the MapFlow core, enabling integration with legacy C++ applications or other languages.

## Purpose

Originally designed to bridge the gap between the legacy Qt/C++ application and the new Rust core. As the Rust rewrite (`mapmap`) becomes the primary application, this crate serves as an integration point for external systems.

## Features

- **C Header Generation:** Uses `cbindgen` to generate `mapflow.h`.
- **Stable ABI:** Exposes a C-compatible API for core functionality.
