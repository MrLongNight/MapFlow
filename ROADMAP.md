# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 1.0.0 (Release Candidate)
> **Stand:** 2026-03-05 05:15
> **Zielgruppe:** @Projektleitung und Entwickler-Team
> **Projekt-Version:** 1.0.0-RC1

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
* **Status:** ✅ **COMPLETED (v1.0.0 Audit 05.03.2026)**.
* **Maßnahme:** Alle 'Broken Nodes' im Shader-Graph wurden repariert. Zyklenerkennung implementiert. Bild-Laden stabilisiert.

### B) Timeline Integration (V3)

* **Priorität:** 🚀 **HIGH**
* **Ziel:** Vollständige Integration der Module in die Timeline.
* **Status:** ✅ **COMPLETED**.
* **Funktionalität:**
  * Jeder Parameter eines Nodes via Trigger-Nodes steuerbar.
  * Multi-Track Timeline V3 integriert und getestet.

### C) Stabilität & Performance

* **Priorität:** 🛡️ **HIGH**
* **Ziel:** Fixen von Fehlern und Problemen, Verbesserung der Performance.
* **Status:** ✅ **STABLE FOR RC1**.
* **Cleanup:** Radikaler Cleanup über alle Crates (mapmap-*) durchgeführt. Clippy-Warnings eliminiert.
* ✅ **Refactoring:** Aufteilung übergroßer Dateien abgeschlossen.

### D) Release-Artefakte

* **Priorität:** 📦 **REQUIRED**
* **Status:** 🟡 **IN PROGRESS (Packaging)**.
* **Lieferumfang:**
  * Produktive Version von MapFlow (v1.0.0).
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

* ⚠️ **Performance:** Die App läuft derzeit mit ca. 23 FPS (AMD Radeon R5 430 Sekundär-Adapter).
* ⚠️ **Media-Decoder:** Warnungen `Decoder error: Input changed` bei WebM/VP9. Pipeline-Optimierung für 1.0.1 geplant.
* ⚠️ **Hue-System:** DTLS-Verbindung zur Hue Bridge benötigt OpenSSL im Release-Build (Staging für 1.0.0-RC1).

---

## Task-Gruppen (Adaptiert für Rust)

* **T0:** Architektur & Datenmodell (`structs`, `enums`, `traits`)
* **T1:** Core-Logik & Algorithmen (No-std compatible logic)
* **T2:** Rendering & GPU (`wgpu`, Shader)
* **T3:** UI & Interaktion (`egui`)
* **T4:** IO & Hardware (Disk, Network, USB)

## Implementierungsdetails nach Crate

### `mapmap-core`

* ✅ **READY**: Datenmodell und Business-Logik stabilisiert.

### `mapmap-render`

* ✅ **STABLE**: wgpu-Instanz und Frame-Pipeline stabilisiert.
* 🟡 **TODO**: Spout-Modernisierung für wgpu 0.19+.

### `mapmap-ui`

* ✅ **STABLE**: egui-Interface und Modul-Canvas stabilisiert.

### `mapmap-bevy`

* ✅ **READY**: Bevy-Engine Integration (0.16) validiert.

## Technologie-Stack und Entscheidungen

* **Sprache:** Rust 2021 (wegen Sicherheit und Performance).
* **GUI:** `egui`.
* **Grafik:** `wgpu`.
* **Video:** `ffmpeg-next`.
* **Audio:** `cpal`.
* **Build-System:** Cargo.

## Build- und Test-Strategie

* ✅ **Unit Tests:** In jedem Modul (`#[test]`).
* ✅ **Linter:** `clippy` (Strikt).
* ✅ **Formatter:** `rustfmt`.
