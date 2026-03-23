# DOC-C3: WGSL Shader System

Dieses Dokument beschreibt die Implementierung und Nutzung von Shadern in MapFlow.

## 1. Shader-Sprache: WGSL
MapFlow nutzt ausschließlich **WGSL** (WebGPU Shading Language). Dies garantiert native Performance auf allen Plattformen (Vulkan, Metal, DX12).

## 2. Kern-Shader
*   `shaders/mesh_warp.wgsl`: Berechnet die geometrische Verzerrung für das Projection Mapping.
*   `shaders/blend_modes.wgsl`: Implementiert die mathematischen Formeln für Layer-Blending (Add, Multiply, Overlay, etc.).
*   `shaders/edge_blend.wgsl`: Steuert die Helligkeitsverläufe in Überlappungsbereichen.

## 3. Shader Graph
Der Shader-Graph in der UI erlaubt es, visuelle Knoten zu Effekten zu kombinieren.
- **Kompilierung**: Der Graph wird zur Laufzeit in einen monolithischen WGSL-Shader übersetzt.
- **Uniforms**: Parameter wie `Speed`, `Intensity` oder `Color` werden als Uniform-Buffer an den Shader übergeben.

## 4. Hot Reloading
Änderungen an den `.wgsl` Dateien im `shaders/` Verzeichnis werden vom `mapflow-render` automatisch erkannt und zur Laufzeit neu geladen.
