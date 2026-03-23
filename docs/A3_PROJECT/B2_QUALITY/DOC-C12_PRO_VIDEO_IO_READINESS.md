# Professional Video I/O Readiness Baseline

**Status:** Aktuell
**Letztes Update:** 2026-03-21

Dieses Dokument fasst den realen Reifegrad der Professional Video I/O Features (NDI, SRT, HAP) im `main` Branch zusammen, um die Lücke zwischen initialer Planung und tatsächlicher Implementierung transparent zu machen.

## Ziele

1. Klare Abgrenzung der Scopes (Verlinkte Issues: #1091, #1250, #1334).
2. Ehrliche Einstufung der Reife (Disabled, Experimental, Gated, Production-Ready).
3. Dokumentation der Build-/Runtime-/QA-Pfade für QA/Devs.

## Readiness Matrix

| Feature | Typ | Status | Gating/Erreichbarkeit |
|---|---|---|---|
| **NDI Input** | In | **[Experimental]** | Im Code (`mapflow-io`) vorhanden. UI: Gated hinter unsupported warnings. |
| **NDI Output**| Out | **[Experimental]** | Im Code (`mapflow-io`) Platzhalter/Sender. UI: Gated hinter unsupported warnings. |
| **SRT Stream**| Out | **[Experimental]** | Im Code (`mapflow-io`) als reiner Stub markiert. UI: Keine Repräsentation. |
| **HAP Player**| In | **[Experimental]** | Im Code (`mapflow-media`) Dekoder vorhanden, aber Container-Format Placeholder (FFmpeg fehlt). UI: nicht vollständig an Playback-Loop gebunden. |

## Technische Details & Pfade

### NDI (Network Device Interface)
*   **Build-Pfad:** Benötigt das `ndi` Feature in `mapflow-io` und abhängigen Crates. Standardmäßig **nicht** im Default-Build aktiviert.
*   **Runtime-Pfad:**
    *   Input: Nutzt `grafton_ndi`. Discovery funktioniert potenziell, aber Frame-Polling/Upload in die `mapflow` Texture-Pool-Architektur ist unvollständig.
    *   Output: `NdiSender` existiert als Platzhalter.
*   **Issues:** #1091 (NDI MVP), #1250 (External I/O node gating).

### SRT (Secure Reliable Transport)
*   **Build-Pfad:** Benötigt das `stream` Feature in `mapflow-io`.
*   **Runtime-Pfad:** Reines Code-Stub-Skeleton in `crates/mapflow-io/src/stream/srt.rs`. Keine echten Puffer, kein Encoding-Link.
*   **Issues:** Status-Tracking hier verankert (#1334).

### HAP Codec (Hardware Accelerated Video)
*   **Build-Pfad:** Code befindet sich in `mapflow-media` (`hap_decoder.rs`, `hap_player.rs`).
*   **Runtime-Pfad:** Decoder für Snappy und GPU-Upload existieren. Der Container-Parse-Pfad (.mov via FFmpeg) ist ein Placeholder.
*   **Issues:** Status-Tracking hier verankert (#1334). Siehe auch `DOC-C9_HAP_INTEGRATION.md`.

## Abnahme & Freigabe
Keines der genannten Features (NDI, SRT, HAP) ist derzeit als *Production-Ready* abnahmefähig. Sie sind im UI und Code klar als *[Experimental]* oder *[Gated]* markiert, um Produktions-Risiken zu vermeiden.
