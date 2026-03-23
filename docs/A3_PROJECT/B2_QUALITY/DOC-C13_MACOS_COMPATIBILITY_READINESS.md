# DOC-C13: macOS Compatibility Readiness Baseline

**Status:** Aktuell
**Letztes Update:** 2026-03-23

Dieses Dokument fasst den realen Reifegrad der macOS Compatibility Features im `main` Branch zusammen, um die Lücke zwischen initialer Planung und tatsächlicher Implementierung transparent zu machen.

## Ziele

1. Klare Abgrenzung der Scopes (Verlinkte Issues: #1080, #1081, #1082, #1083, #1084, #1027, #1028).
2. Ehrliche Einstufung der Reife (Offen, In Analyse, In Umsetzung, Abgeschlossen).
3. Dokumentation der Build-/Runtime-/QA-Pfade für QA/Devs.

## Readiness Matrix

| Feature / Sub-Issue | Status | Beschreibung |
|---|---|---|
| **MFsub_#1080-macOS-Build-Bootstrap** | **[Abgeschlossen]** | Build-System ist auf macOS vorbereitet (`macos-beta` Feature vorhanden). |
| **MFsub_#1081-macOS-Runtime-Stabilization** | **[Abgeschlossen]** | Runtime-Stabilisierung auf macOS Metal. |
| **MFsub_#1082-macOS-Media-FFmpeg-Path** | **[Abgeschlossen]** | FFmpeg Media-Pfad auf macOS validiert. |
| **MFsub_#1083-macOS-Audio-Validation** | **[Abgeschlossen]** | Audio-Validierung durchgeführt (ggf. Feature-gated). |
| **MFsub_#1084-macOS-CI-Validation** | **[Abgeschlossen]** | CI Build für macOS eingerichtet (`build-macos` Job). |
| **MFsub_#1027-macOS-Packaging-Notarization**| **[Abgeschlossen]** | Packaging und Notarization (DMG) Strategie validiert. |
| **MFsub_#1028-macOS-Native-Interop** | **[Abgeschlossen]** | Native Interop (Syphon, VirtualCamera) vorhanden/gated. |

## Abnahme & Freigabe
Alle gelisteten Sub-Issues wurden als **Abgeschlossen** markiert. Die Umsetzung ist im Code, in der CI und in den Build-Flags (`macos-beta`) sichtbar.
