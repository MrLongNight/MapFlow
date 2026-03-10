# DOC-C11: Video I/O & Interop Strategie (NDI, Spout, WGPU)

## 1. Status Quo & Herausforderungen
MapFlow nutzt `wgpu` für das Rendering. Der Austausch von Videodaten mit anderen Applikationen (NDI, Spout) erfordert effiziente Übergänge zwischen GPU-Kontexten oder schnelles Readback in den RAM.

### Herausforderungen:
*   **MF-019 (Spout):** WGPU abstrahiert native Handles. Spout benötigt unter Windows DX11/DX12 Shared Handles. Ein Update der WGPU-Version ist nötig, um stabilen Zugriff auf `NativeObject` zu erhalten.
*   **MF-021 (NDI):** Erfordert Discovery-Logik im UI (Sidebar) und eine effiziente Pipeline für das Senden/Empfangen von NDI-Frames ohne CPU-Bottlenecks.

## 2. Implementierungsplan

### Phase 1: WGPU & Spout Interop (MF-019)
1.  Upgrade auf aktuelle `wgpu` Version zur Nutzung verbesserter `ExternalImage` APIs.
2.  Implementierung eines `SpoutSender` Moduls in `mapmap-io`.
3.  DirectX 11 Interop-Layer für Spout-Sharing (Zero-Copy Pfad).

### Phase 2: NDI Discovery & UI (MF-021)
1.  Integration von `ndi-rs` Discovery in den `SidebarFlow`.
2.  Automatisches Listen von NDI-Quellen als "Live-Input" Nodes im Shader-Graph (`MF-022`).
3.  Asynchrones Frame-Capturing für den NDI-Sender.

## 3. Roadmap Verlinkung
*   [MF-019-SPOUT-WGPU-UPDATE] -> Dieses Dokument
*   [MF-021-NDI-DISCOVERY-UI] -> Dieses Dokument
*   [MF-003-INSPECTOR-PREVIEW-ALL-NODES] -> Preview-Pfad für NDI-Quellen nutzen.
