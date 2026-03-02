# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 2.1
> **Stand:** 2026-03-02 12:00
> **Zielgruppe:** @Projektleitung und Entwickler-Team
> **Projekt-Version:** 0.2.1

---

## 📋 Inhaltsverzeichnis

1. [Fokus & Ziele für Release 1.0](#fokus--ziele-für-release-10)
2. [Feature-Status-Übersicht](#feature-status-übersicht)
3. [Architektur und Crate-Übersicht](#architektur-und-crate-übersicht)
4. [Multi-PC-Architektur (Phase 8)](#multi-pc-architektur-phase-8--neu)
5. [Arbeitspakete für @jules](#arbeitspakete-für-jules)
6. [Task-Gruppen (Adaptiert für Rust)](#task-gruppen-adaptiert-für-rust)
7. [Implementierungsdetails nach Crate](#implementierungsdetails-nach-crate)
8. [Technologie-Stack und Entscheidungen](#technologie-stack-und-entscheidungen)
9. [Build- und Test-Strategie](#build--und-test-strategie)

---

## Fokus & Ziele für Release 1.0

Basierend auf dem aktuellen Status und den Projektzielen für die erste produktive Version (v1.0):

### A) Render Pipeline & Module Logic

* **Priorität:** 🔥 **CRITICAL**
* **Ziel:** Eine fehlerfreie Render-Pipeline, in der alle Modul-Nodes und die zugehörige Logik stabil funktionieren.
* **Status:** In Stabilisierung. Main Application Entry Point implementiert (2026-01-31). Laufende Bugfixes und Node-Stabilisierung. Module System refactored (2026-02-27).
* **Maßnahme:** "Broken Nodes" reparieren. Experimentelle Features (wie NDI/Multi-PC) ggf. ausklammern oder verstecken, falls sie die Stabilität gefährden.

### B) Timeline Integration (V3)

* **Priorität:** 🚀 **HIGH**
* **Ziel:** Vollständige Integration der Module in die Timeline.
* **Funktionalität:**
  * Jeder Parameter eines Nodes (z.B. "Blur Amount") soll via Trigger-Nodes und Verbindungen definiert werden können.
  * Arrangement der konfigurierten Module in der Timeline.
  * Unterstützung für **Manuelle**, **Hybride** und **Vollautomatische** Steuerung.
  * Möglichkeit, Parameter bei Bedarf manuell zu triggern.

### C) Stabilität & Performance

* **Priorität:** 🛡️ **HIGH**
* **Ziel:** Fixen von Fehlern und Problemen, Verbesserung der Performance.
* **Cleanup:** Entfernen von UI-Elementen, die keine Funktion haben.
* ✅ **Refactoring:** Aufteilung übergroßer Dateien (insb. `module_canvas/mod.rs` und `module.rs` abgeschlossen).

### D) Release-Artefakte

* **Priorität:** 📦 **REQUIRED**
* **Lieferumfang:**
  * Produktive Version von MapFlow (v1.0).
  * Fertiger Installer für **Windows** (.msi/.exe) und **Linux** (.deb/AppImage).
  * Handbuch in Form von **GitHub Wiki**-Beiträgen.

---

## Feature-Status-Übersicht

### General Updates

* ✅ **Professional README Overhaul** (COMPLETED 2026-02-20)
* ✅ **Rebranding: VjMapper -> MapFlow** (COMPLETED 2025-12-22)
  * ✅ Rename Project (2025-12-22)
  * ✅ Update UI Strings & Docs (2025-12-22)
  * ✅ Rename WiX Installer Config (2025-12-22)

### Core / Layer / Mapping System

* ✅ **Module System Refactoring** (`mapmap-core/src/module/`) – **COMPLETED (2026-02-27)**
  * ✅ Split monolithic `module.rs` into submodules (`types.rs`, `config.rs`, `manager.rs`, `mod.rs`)
  * ✅ Improved maintainability and structure
  * ✅ Backward compatibility via re-exports

* ✅ **Layer-System** (`mapmap-core/src/layer.rs`)
  * ✅ Transform-System (Position, Rotation, Scale)
  * ✅ Opacity-Steuerung (0.0-1.0)
  * ✅ Blend-Modi (Normal, Add, Multiply, Screen, Overlay, etc.)
  * ✅ ResizeMode (Fill, Fit, Stretch, Original)
  * ✅ LayerManager für Komposition
  * ✅ Hierarchisches Layer-System

* ✅ **Mapping-System** (`mapmap-core/src/mapping.rs`)
  * ✅ Mapping-Hierarchie (Paint → Mapping → Mesh)
  * ✅ MappingManager für Verwaltung
  * ✅ Mapping-IDs und Referenzen

* ✅ **Mesh-System** (`mapmap-core/src/mesh.rs`)
  * ✅ MeshVertex mit UV-Koordinaten
  * ✅ BezierPatch für Warping
  * ✅ Keystone-Korrektur
  * ✅ MeshType (Quad, Grid, Custom)

* ✅ **Paint-System** (`mapmap-core/src/paint.rs`)
  * ✅ Paint als Basis-Datenstruktur
  * ✅ Media-Source-Integration

---

## Bekannte Probleme & Performance (Aktueller Stand)

* ⚠️ **Performance:** Die App läuft derzeit mit ca. 23 FPS. Dies liegt teilweise an der Auswahl der GPU (AMD Radeon R5 430), die vom System als sekundärer Adapter eingestuft wird.
* ⚠️ **Media-Decoder:** Es treten häufige Warnungen auf: `Decoder error: Input changed`. Dies deutet darauf hin, dass die WebM-Dateien (VP9/VP8) in der aktuellen Pipeline Probleme beim Frame-Decoding verursachen, sobald sich die Eingabeparameter ändern.
* ⚠️ **Hue-System:** Die DTLS-Verbindung zur Hue Bridge schlägt fehl, da OpenSSL im aktuellen Build-Profil deaktiviert ist (um Build-Hänger zu vermeiden).

---

## Aktuelle Jules-Aufträge

_Stand: 2026-02-27 23:59 (Europe/Berlin)_

| Session-ID | Task | Status | Link | Notizen |
|------------|------|--------|------|---------|
| 14374730097834491321 | [ARCH-01] Complete core/module.rs Refactoring | Abgeschlossen | [https://jules.google.com/session/14374730097834491321](https://jules.google.com/session/14374730097834491321) | Monolithic module.rs split into submodules. |
| 15034419910350922962 | [SAFE-01] Eliminate Unsafe Hacks in Render Loop | In Arbeit | [https://jules.google.com/session/15034419910350922962](https://jules.google.com/session/15034419910350922962) | Partially removed; *mut App and transmute still present in render loop. |
| 4749311560780055775 | [IO-01] Functional NDI Sender and MPV Decoder | In Arbeit | [https://jules.google.com/session/4749311560780055775](https://jules.google.com/session/4749311560780055775) | NDI Sender implemented; MPV Decoder integrated but currently renders gray frames. |
| 56d67ed3 | Restore canvas toolbar and diagnostics | Abgeschlossen | - | Modern egui API implementation. |
| 73698441478363935 | link-system-ui | Abgeschlossen | - | Link system implementation. |
| 3125037812423445221 | timeline-v3-integration | Abgeschlossen | - | Multi-track timeline V3 integrated. |

## Abgeschlossene Jules-Aufträge (Archiv)

| Session-ID | Task | Status | Notizen |
|------------|------|--------|---------|
| 14318715518596799691 | [CORE-01] Node Parameter & Trigger Target Expansion | Abgeschlossen | Added OffsetX/Y, FlipH/V and Bevy targets. |
| 12744118335336060991 | Rebuild: FramePipeline threaded uploads | Abgeschlossen | PR #831 merged. |
| 11538622621812368551 | Rebuild: module_canvas in Submodule aufteilen | Abgeschlossen | PR #832 merged. |
| 1499173718553143537 | Fix GPU Upload Thread blocking | Abgeschlossen | PR #826 merged. |
| 9472154532138526611 | Refactor Phase 1 – `module_canvas` God Object aufteilen | Abgeschlossen | PR #822 merged. |

---

## Task-Gruppen (Adaptiert für Rust)

* **T0:** Architektur & Datenmodell (`structs`, `enums`, `traits`)
* **T1:** Core-Logik & Algorithmen (No-std compatible logic)
* **T2:** Rendering & GPU (`wgpu`, Shader)
* **T3:** UI & Interaktion (`egui`)
* **T4:** IO & Hardware (Disk, Network, USB)

## Implementierungsdetails nach Crate

### `mapmap-core`

* Enthält keine Abhängigkeiten zu Rendering oder UI.
* Definiert das Datenmodell (`Layer`, `Mapping`, `Project`).
* Implementiert die Business-Logik (z.B. `overlaps(layer1, layer2)`).

### `mapmap-render`

* Managt die `wgpu` Instanz, Adapter, Device und Queue.
* Implementiert `Renderer` Traits für verschiedene Zeichendienste.
* Hält Shader-Code als Strings oder Dateien.

### `mapmap-ui`

* Implementiert `egui::App`.
* Handhabt Input-Events.
* Visualisiert den State aus `mapmap-core`.

### `mapmap-bevy`

* Integriert die Bevy Engine für 3D-Rendering.
* Bietet Partikelsysteme via Custom Mesh-Based Implementation (Bevy 0.16 compatible).
* Teilt den wgpu-Context mit der Hauptanwendung.

## Technologie-Stack und Entscheidungen

* **Sprache:** Rust 2021 (wegen Sicherheit und Performance).
* **GUI:** `egui` (Immediate Mode, einfach zu integrieren, wgpu-basiert).
* **Grafik:** `wgpu` (WebGPU-Standard, Cross-Platform, Zukunftssicher).
* **Video:** `ffmpeg-next` (Bindings für FFmpeg).
* **Audio:** `cpal` (Low-Level Audio API).
* **Build-System:** Cargo (Standard).

## Build- und Test-Strategie

* **Unit Tests:** In jedem Modul (`#[test]`).
* **Integration Tests:** In `tests/` Ordner.
* **CI:** GitHub Actions (Build, Test, Lint).
* **Linter:** `clippy` (Strikt).
* **Formatter:** `rustfmt`.
