# Spezifikation: Professional Video I/O (NDI, Spout, Syphon)

## 1. Übersicht
Integration von Industriestandards für den Austausch von Video-Streams zwischen Applikationen in Echtzeit.

*   **Spout (Windows)** / **Syphon (macOS)**: Lokaler Austausch auf demselben Rechner via GPU-Texture-Sharing (Zero-Copy oder Near-Zero-Copy). Extrem schnell, extrem geringe Latenz.
*   **NDI® (Network Device Interface)**: Austausch über LAN (Gigabit Ethernet). Komprimiert (ähnlich MPEG, aber I-Frame only), geringe Latenz (~1 Frame).

## 2. Architektur: `mapmap-io`
Das Crate `mapmap-io` wird zur zentralen Schaltstelle für diese Protokolle.

### 2.1 Spout (Windows)
*   **Library**: Es gibt Rust-Bindings `spout` oder direkte Nutzung der `OpenGL32.dll` / DirectX shared handles.
*   **Wichtig**: Spout basiert traditionell auf DirectX (DX9/DX11) oder OpenGL. Da wir `wgpu` nutzen (meist Vulkan oder DX12), müssen wir auf die Interop-Fähigkeiten achten.
    *   *Herausforderung*: `wgpu` abstrahiert den darunterliegenden API-Handle. Wir müssen an den nativen Handle (z.B. ID3D12Resource) kommen, um ihn mit Spout zu teilen. Das erfordert `unsafe` Code und plattformspezifische Extensions in `wgpu`.
    *   *Workaround*: Falls Direct-Sharing zu schwer ist, müssen wir einen CPU-Roundtrip machen (GPU -> RAM -> Spout), was Performance kostet.
    *   *Lösung 2.0*: Wir schreiben einen nativen DX11-Context nur für Spout und kopieren die Daten via Interop.

### 2.2 NDI (Network)
*   **Library**: `ndi-rs` (Rust Bindings für das offizielle NDI SDK).
*   **Lizenz**: NDI SDK ist proprietär (NewTek/Vizrt). Die DLLs dürfen evtl. nicht direkt in Open-Source-Repos gepackt werden (User muss NDI Runtime installieren).
*   **Workflow**:
    *   **Receive**: Listening thread empfängt Frames -> Dekomprimiert -> Upload in GPU-Textur.
    *   **Send**: GPU-Textur -> Download (Readback) -> Komprimierung -> Senden.

## 3. Implementation Features

### 3.1 NDI Receiver (Input)
*   User wählt "NDI Source" im Media Browser.
*   Dropdown listet alle verfügbaren NDI-Quellen im Netzwerk (Discovery).
*   Stream wird wie ein Live-Video behandelt (Buffer-Management wichtig für Jitter-Ausgleich).

### 3.2 Spout Receiver/Sender
*   **Receiver**: VJMapper empfängt Visuals von Resolume oder TouchDesigner.
*   **Sender**: VJMapper sendet den "Main Mix" oder einzelne Slices an OBS Studio (für Livestreaming).

## 4. UI Integration
Neue Kategorie im "Source"-Panel: **Live Inputs**.
*   Webcams
*   NDI Sources
*   Spout Senders

## 5. Roadmap
1.  **Phase 1: Spout Sender (Windows)**. Das ist der häufigste Use-Case (VJMapper -> OBS / Resolume).
2.  **Phase 2: NDI Receiver**. Damit man Laptops verbinden kann.
3.  **Phase 3: Syphon (macOS)**. Erst wenn Portierung auf Mac stabil ist.
