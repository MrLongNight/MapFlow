# Vorce - GitHub Copilot Review Instructions

## 🎯 Project Overview

**Vorce** ist ein professionelles Open-Source Projection-Mapping-Tool in Rust.

**Tech Stack:**

- **Sprache:** Rust 2021 (MSRV 1.94+)
- **Graphics:** wgpu (Vulkan/Metal/DX12)
- **UI:** egui (Immediate Mode)
- **Media:** FFmpeg, NDI SDK
- **Build:** Cargo Workspace

**Crates:**

```
crates/
├── vorce-core       # Kernlogik, Projektmanagement
├── vorce-ui         # egui UI-Komponenten
├── vorce-io         # FFmpeg, NDI, Spout/Syphon
├── vorce-bevy       # 3D Engine (Bevy)
└── vorce            # Haupt-Binary
```

---

## 🔍 Review Priorities

### 🔴 HOCH (Immer kommentieren)

**1. Memory Safety & `unsafe`**

```rust
// ❌ BAD
unsafe {
    // No SAFETY comment
    *ptr = value;
}

// ✅ GOOD
// SAFETY: ptr is valid for writes because it was just allocated
// and is within bounds of the allocation.
unsafe {
    *ptr = value;
}
```

**2. Error Handling**

```rust
// ❌ BAD
let data = file.read().unwrap();

// ✅ GOOD
let data = file.read()
    .map_err(|e| Error::FileRead { source: e })?;
```

**3. Security Issues**

- Unvalidated user input
- Path traversal vulnerabilities
- Command injection risks
- Credential exposure

---

### 🟡 MITTEL (Bei Signifikanz)

**4. Performance**

```rust
// ⚠️ WARNUNG
for item in large_vec.iter() {
    result.push(item.clone()); // Unnecessary clone
}

// ✅ BESSER
for item in large_vec.iter() {
    result.push(item); // Borrow statt Clone
}
```

**5. Cross-Platform Issues**

```rust
// ❌ BAD
use std::os::windows::*; // Nur Windows

// ✅ GOOD
#[cfg(target_os = "windows")]
use std::os::windows::*;
```

**6. GPU Resource Management**

```rust
// ✅ GOOD - Implementiere Drop für Cleanup
impl Drop for GpuTexture {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}
```

---

### 🟢 NIEDRIG (Optional)

**7. Code Style (nur bei klaren Verbesserungen)**

```rust
// Akzeptabel (wird von rustfmt gehandhabt)
fn foo(  ) {  }

// Bevorzugt, aber nicht kritisch
fn foo() {}
```

**8. Micro-Optimierungen**

- Nur bei Hot Paths kommentieren
- Mit Benchmarks belegen

---

## 🚫 NICHT Kommentieren

- ✅ Formatierung (handled by `cargo fmt`)
- ✅ Clippy Warnings (handled by CI)
- ✅ Typos in Comments/Docs (nicht kritisch)
- ✅ Import-Reihenfolge
- ✅ Variable-Namen (außer sehr verwirrend)

---

## 💬 Tone Guidelines

**DO:**

- ✅ Konstruktiv: "Erwäge stattdessen..."
- ✅ Erklärend: "Dies könnte problematisch sein, weil..."
- ✅ Kurz: Max 2-3 Sätze pro Kommentar

**DON'T:**

- ❌ "Das ist falsch"
- ❌ "Du musst..."
- ❌ Nitpicking ohne Begründung

---

## 🎯 Spezifische Checks

### egui UI Code

```rust
// ✅ Accessibility
if ui.button("Delete").clicked() ||
   ui.input(|i| i.key_pressed(Key::Delete)) {
    // Keyboard + Mouse support
}
```

### FFmpeg/Media Handling

```rust
// ✅ Resource Cleanup
let mut decoder = ffmpeg::decoder::new(stream)?;
// ... verwenden ...
drop(decoder); // Explizit cleanup bei C-Bindings
```

### Shader Code (WGSL)

```wgsl
// Prüfe auf:
// - Korrekte Binding-Indices
// - Valid Types (f32, vec4, etc.)
// - Vertex/Fragment Shader Compatibility
```

---

## 📋 Review Template

```markdown
## 🤖 Copilot Review

### ✅ Positives
- [Was gut gemacht ist]

### ⚠️ Zu Beachten
**[Datei:Zeile]** - [Problem]
- **Warum:** [Begründung]
- **Lösung:**
  ```rust
  // Vorschlag
  ```

### 💡 Optional

- [Nice-to-have Verbesserungen]

```

---

## 🔗 Ressourcen

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [wgpu Best Practices](https://wgpu.rs/)
- [egui Docs](https://docs.rs/egui/)

---

**Version:** 1.0  
**Last Updated:** 2026-02-05
