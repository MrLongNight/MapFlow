# Multi-PC Architektur

MapFlow unterstützt skalierbare Setups über mehrere Computer hinweg. Dies ist notwendig für Installationen mit sehr vielen Projektoren oder extrem hohen Auflösungen.

## Optionen

### Option A: NDI Video-Streaming (Empfohlen)
Nutzung von NDI (Network Device Interface) zur Übertragung von Video über IP.
*   **Master**: Rendert das Composing und sendet Slices oder das Gesamtbild per NDI.
*   **Clients**: Empfangen den NDI-Stream und zeigen ihn im Fullscreen an.
*   **Status**: Implementiert (`mapmap-io/src/ndi`).

### Option B: Distributed Rendering (High-End)
Szenen-Synchronisation statt Video-Streaming.
*   **Konzept**: Der Master sendet nur Steuerdaten (OSC, Parameter, Timecode). Jeder Client rendert sein Bild lokal in voller Qualität.
*   **Vorteil**: Geringere Netzwerkbandbreite, höhere Qualität (keine Kompressionsartefakte).
*   **Nachteil**: Alle Clients benötigen starke GPUs.
*   **Status**: Geplant (Phase 8).

### Option C: Legacy Clients & Raspberry Pi
Für Low-Budget oder ältere Hardware.
*   **Raspberry Pi**: Nutzung als kompakter NDI-Player oder RTSP-Client.
*   **Legacy**: Nutzung von H.264 Streams für PCs ohne starke GPU.

## Netzwerk-Anforderungen
*   **LAN**: Gigabit Ethernet ist Minimum. 10GbE empfohlen für 4K NDI Streams.
*   **Latenz**: Optimierung auf <1 Frame (16ms) ist das Ziel.
