# Rendering Pipeline & Technik

MapFlow nutzt eine GPU-beschleunigte Pipeline basierend auf **WGPU** (WebGPU native). Dies ermöglicht Cross-Platform-Kompatibilität (Vulkan, DX12, Metal) bei maximaler Performance.

## Pipeline-Stufen

### 1. Compositor (`mapmap-render/src/compositor.rs`)
Der Compositor ist das Herzstück der Bildmischung.
*   **Multi-Layer**: Unterstützt unbegrenzte Layer.
*   **Blend-Modi**: Implementiert über 10 Modi (Normal, Add, Multiply, Screen, Overlay, etc.) via `shaders/blend_modes.wgsl`.
*   **Caching**: Texturen werden intelligent gecacht, um Uploads zu minimieren.

### 2. Mesh & Warping (`mapmap-render/src/mesh_renderer.rs`)
Für Projection Mapping ist geometrische Verzerrung essenziell.
*   **Bezier-Warping**: Meshes können durch Bezier-Kurven verformt werden, um sich gekrümmten Oberflächen anzupassen.
*   **Keystone**: Klassische 4-Punkt-Korrektur.
*   **Shader**: `shaders/mesh_warp.wgsl` berechnet die Texturkoordinaten-Transformation.

### 3. Edge Blending & Kalibrierung
Für Multi-Projektor-Setups.
*   **Edge Blending**: Weiche Überblendung zwischen Projektoren basierend auf Gamma-Kurven (`shaders/edge_blend.wgsl`).
*   **Color Calibration**: RGB-Gain/Offset und Gamma pro Output zur Anpassung an unterschiedliche Projektor-Farbräume.

### 4. Effekte & Shader Graph
*   **Shader Graph**: Benutzer erstellen Effekte durch visuelle Nodes.
*   **Code Generation**: Der Graph wird zur Laufzeit in validen **WGSL**-Code kompiliert.
*   **Hot Reload**: Shader-Änderungen werden live übernommen (Watcher auf `.wgsl` Dateien).

## Performance-Optimierung

### Texture Upload
Video-Decoding ist oft der Flaschenhals.
*   **Staging Buffers**: Wir nutzen einen Pool von Staging-Buffern für asynchrone GPU-Uploads.
*   **Direct Upload**: Wenn möglich (Hardware-Support), werden Frames direkt in GPU-Memory dekodiert (Geplant).

### HAP Codec Support
Für extrem hochauflösende Videos nutzen wir den HAP-Codec.
*   **Funktionsweise**: HAP ist ein Textur-Kompressions-Format (ähnlich DXT/BC1). Die CPU muss nur entpacken (Snappy), die GPU übernimmt das Decoding.
*   **Implementation**: 
    *   Decoder: `mapmap-media/src/hap_decoder.rs`
    *   Shader: `shaders/ycocg_to_rgb.wgsl` (für Farbkonvertierung).
