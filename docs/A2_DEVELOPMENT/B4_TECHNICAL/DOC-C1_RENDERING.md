# Rendering Pipeline & Technik

MapFlow nutzt eine GPU-beschleunigte Pipeline basierend auf **WGPU** (WebGPU native). Dies ermoeglicht Cross-Platform-Kompatibilitaet (Vulkan, DX12, Metal) bei maximaler Performance.

## Pipeline-Stufen

### 1. Compositor (`mapflow-render/src/compositor.rs`)
Der Compositor ist das Herzstueck der Bildmischung.
*   **Multi-Layer**: Unterstuetzt unbegrenzte Layer.
*   **Blend-Modi**: Implementiert ueber 10 Modi (Normal, Add, Multiply, Screen, Overlay, etc.) via `shaders/blend_modes.wgsl`.
*   **Caching**: Texturen werden intelligent gecacht, um Uploads zu minimieren.

### 2. Mesh & Warping (`mapflow-render/src/mesh_renderer.rs`)
Fuer Projection Mapping ist geometrische Verzerrung essenziell.
*   **Bezier-Warping**: Meshes koennen durch Bezier-Kurven verformt werden, um sich gekruemmten Oberflaechen anzupassen.
*   **Keystone**: Klassische 4-Punkt-Korrektur.
*   **Shader**: `shaders/mesh_warp.wgsl` berechnet die Texturkoordinaten-Transformation.

### 3. Edge Blending & Kalibrierung
Fuer Multi-Projektor-Setups.
*   **Edge Blending**: Weiche Ueberblendung zwischen Projektoren basierend auf Gamma-Kurven (`shaders/edge_blend.wgsl`).
*   **Color Calibration**: RGB-Gain/Offset und Gamma pro Output zur Anpassung an unterschiedliche Projektor-Farbraeume.

### 4. Effekte & Shader Graph
*   **Shader Graph**: Benutzer erstellen Effekte durch visuelle Nodes.
*   **Code Generation**: Der Graph wird zur Laufzeit in validen **WGSL**-Code kompiliert.
*   **Hot Reload**: Shader-Aenderungen werden live uebernommen (Watcher auf `.wgsl` Dateien).

## Performance-Optimierung

### Texture Upload
Video-Decoding ist oft der Flaschenhals.
*   **Staging Buffers**: Wir nutzen einen Pool von Staging-Buffern fuer asynchrone GPU-Uploads.
*   **Direct Upload**: Wenn moeglich (Hardware-Support), werden Frames direkt in GPU-Memory dekodiert (geplant).

### HAP Codec Support
Fuer HAP existieren Decoder- und Shader-Bausteine, der Pfad ist aktuell aber nicht der durchgaengig verifizierte Standard-Mediapfad auf allen Setups.
*   **Funktionsweise**: HAP ist ein Textur-Kompressions-Format (aehnlich DXT/BC1). Die CPU muss nur entpacken (Snappy), die GPU uebernimmt das Decoding.
*   **Implementation / aktueller Stand**:
    *   Hinweis: End-to-end-Integration und Produktionsreife muessen separat verifiziert werden.
    *   Decoder: `mapflow-media/src/hap_decoder.rs`
    *   Shader: `shaders/ycocg_to_rgb.wgsl` (fuer Farbkonvertierung).
