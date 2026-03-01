# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 2.2
> **Stand:** 2026-02-28 12:00
> **Zielgruppe:** @Projektleitung und Entwickler-Team
> **Projekt-Version:** 0.2.2

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
* **Status:** In Stabilisierung. Module System refactored (2026-02-27).
* **Maßnahme:**
    * **Trigger Smoothing** (Attack/Release) implementieren (Mapping existiert, Logik fehlt).
    * **Shader Codegen** (Scale/Rot/Trans Injection) vervollständigen.
    * **Graph Validation** (Cycle Detection) hinzufügen.

### B) Timeline Integration (V3)

* **Priorität:** 🚀 **HIGH**
* **Ziel:** Vollständige Integration der Module in die Timeline.
* **Funktionalität:**
  * Jeder Parameter eines Nodes via Trigger-Nodes steuerbar.
  * **Status:** V3 integriert, aber Undo/Redo Deckung für Layer-Mutationen fehlt (derzeit nur Snapshots).

### C) Stabilität & Performance

* **Priorität:** 🛡️ **HIGH**
* **Ziel:** Fixen von Fehlern und Problemen.
* **Maßnahme:**
    * **wgpu Lifetime Hack** in `render.rs` final eliminieren (Unsafe Transmute L501).
    * **MPV Decoder** Performance optimieren (Umstieg von `screenshot-raw` auf native GPU sharing).
    * **Panic-Free Policy** (MCP Server Cleanup, .unwrap() Ersetzung).

---

## Feature-Status-Übersicht

### Core / Layer / Mapping System

* ✅ **Module System Refactoring** (`mapmap-core/src/module/`) – **COMPLETED (2026-02-27)**
* ✅ **Layer-System** (`mapmap-core/src/layer.rs`) – **COMPLETED**
* 🟡 **Trigger System** – **PARTIAL** (Smoothing/Attack/Release Logik fehlt)
* 🟡 **Mesh System** – **PARTIAL** (OBJ/SVG Import fehlt)
* 🟡 **Shader Graph** – **PARTIAL** (Validation & Parameter Injection fehlt)

### IO & Media

* 🟡 **MPV Decoder** – **PARTIAL** (Screenshot-Raw Fallback aktiv)
* 🟡 **NDI Support** – **PARTIAL** (Sender OK, Receiver fehlt)
* 🔴 **Spout Support** – **MISSING** (In Sync-Loop deaktiviert)
* 🟡 **Philips Hue** – **PARTIAL** (Core Params OK, UI/Pairing fehlt)

---

## Bekannte Probleme & Performance (Aktueller Stand)

* ⚠️ **Performance:** Die App läuft derzeit mit ca. 23 FPS (AMD Radeon R5 430). Optimierung der Video-Upload-Pipeline (MPV) notwendig.
* ⚠️ **Media-Decoder:** `Decoder error: Input changed` Warnungen bei WebM.
* ⚠️ **Hue-System:** DTLS-Verbindungsprobleme (OpenSSL).

---

## Aktuelle Jules-Aufträge

| Session-ID | Task | Status | Link | Notizen |
|------------|------|--------|------|---------|
| 15034419910350922962 | [SAFE-01] Eliminate Unsafe Hacks in Render Loop | In Arbeit | [Link](https://jules.google.com/session/15034419910350922962) | Transmute in render.rs noch vorhanden. |
| 4749311560780055775 | [IO-01] Functional NDI Sender and MPV Decoder | In Arbeit | [Link](https://jules.google.com/session/4749311560780055775) | MPV Decoder integriert; Performance-Optimierung offen. |
| - | [CORE-02] Trigger Smoothing & Attack/Release | **NEU** | - | Implementierung der Mapping-Logik in trigger.rs. |
| - | [CORE-03] Mesh Import & Shader Codegen | **NEU** | - | OBJ/SVG Loader & Parameter Injection. |
| - | [UI-01] Bevy Node Controls | **NEU** | - | UI Felder für Partikel, Atmosphere und Modelle. |

---

## Task-Gruppen (Adaptiert für Rust)

* **T0:** Architektur & Datenmodell (`structs`, `enums`, `traits`)
* **T1:** Core-Logik & Algorithmen (No-std compatible logic)
* **T2:** Rendering & GPU (`wgpu`, Shader)
* **T3:** UI & Interaktion (`egui`)
* **T4:** IO & Hardware (Disk, Network, USB)

## Implementierungsdetails nach Crate

### `mapmap-core`
* Datenmodell (`Layer`, `Mapping`, `Project`).
* Business-Logik.

### `mapmap-render`
* `wgpu` Management, Shader, Render-Pipelines.

### `mapmap-ui`
* `egui` Implementation, Inspector, Canvas.

### `mapmap-bevy`
* 3D-Rendering Integration.

## Technologie-Stack
* **Sprache:** Rust 2021.
* **GUI:** `egui`.
* **Grafik:** `wgpu`.
* **Build-System:** Cargo.
