# HAP Codec Integration Plan für MapFlow

## Übersicht

HAP ist ein Video-Codec für VJ-Software, der GPU-Texturkompression (S3TC/DXT) nutzt.
Die GPU kann die Frames direkt dekomprimieren → minimale CPU-Last.

## HAP Varianten

| Variante | Textur-Format | Beschreibung |
|----------|--------------|--------------|
| **HAP** | DXT1 (BC1) | Beste Kompression, kein Alpha |
| **HAP Alpha** | DXT5 (BC3) | Mit Alpha-Kanal |
| **HAP Q** | 2x DXT5 | Höchste Qualität |
| **HAP Q Alpha** | 2x DXT5 | Höchste Qualität + Alpha |

## Technische Architektur

```
HAP-Datei (.mov container)
    ↓
libavformat (Demuxer) → Extrahiert HAP-Frames
    ↓
Snappy-Dekompression (CPU, schnell)
    ↓
DXT-komprimierte Texturdaten
    ↓
wgpu::TextureFormat::Bc1/Bc3 (GPU, direkt!)
```

## Implementierungsschritte

### Phase 1: Container-Handling (FFmpeg)

- [ ] HAP-Frames via `ffmpeg-next` extrahieren
- [ ] MOV/AVI Container-Support verifizieren
- [ ] Frame-Metadaten (Typ, Größe, Flags) parsen

### Phase 2: HAP-Decoder

- [ ] Snappy-Dekompression (`snap` crate)
- [ ] HAP Frame Header parsen
- [ ] Section-basiertes Decoding (multi-section für große Frames)

### Phase 3: GPU-Upload

- [ ] wgpu BC1/BC3 Textur-Format nutzen
- [ ] Direkt-Upload ohne CPU-Dekompression
- [ ] Textur-Caching für HAP-Frames

### Phase 4: Integration

- [ ] `MediaType::Hap` zu Decoder hinzufügen
- [ ] UI: HAP-Dateien im Media Browser erkennen
- [ ] Performance-Tests vs. H.264

## Rust Crates benötigt

```toml
# Cargo.toml
snap = "1.1"      # Snappy compression
bcndecode = "0.2" # DXT/BCn decode (nur für Software-Fallback)
```

## wgpu Textur-Formate

```rust
// HAP → DXT1 (keine Alpha)
wgpu::TextureFormat::Bc1RgbaUnorm

// HAP Alpha → DXT5
wgpu::TextureFormat::Bc3RgbaUnorm

// HAP Q → Dual DXT5 (Luma + Chroma)
wgpu::TextureFormat::Bc3RgbaUnorm // × 2 Texturen
```

## HAP Frame Header Format

```
Offset  Size  Description
0       4     Section Header Magic (0x484150 = "HAP")
4       4     Section Size (little-endian)
8       1     Section Type:
              - 0x0B = HAP (DXT1)
              - 0x0E = HAP Alpha (DXT5)
              - 0x0C = HAP Q (HapY + CoCg)
9       1     Compressor Type:
              - 0xA0 = None
              - 0xB0 = Snappy
              - 0xC0 = Complex (multi-section)
10+     n     Compressed/Raw texture data
```

## Performance-Ziele

| Metrik | Ziel |
|--------|------|
| 4K HAP Decode | < 1ms/Frame |
| GPU Upload | < 0.5ms/Frame |
| CPU Usage | < 5% |

## Referenzen

- [hap.video](https://hap.video) - Offizielle HAP Spezifikation
- [GitHub Vidvox/hap](https://github.com/Vidvox/hap) - Reference Implementation
- [snap crate](https://docs.rs/snap) - Snappy für Rust
