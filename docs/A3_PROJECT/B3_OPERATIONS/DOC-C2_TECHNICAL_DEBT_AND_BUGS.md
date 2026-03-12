# 🛠️ MapFlow: Technische Schulden, Bugs & Roadmap

Dieses Dokument dient der zentralen Erfassung von Architektur-Problemen, monolithischen "God Files" und der Strategie zur Modularisierung des MapFlow (VJMapper) Projekts.

---

## 🛑 Kritische Architektur-Probleme
*Kernkomponenten, die eine strukturelle Überarbeitung oder Sicherheits-Audits benötigen.*

| Komponente | Problem | Status | Auswirkung | Ort |
| :--- | :--- | :---: | :--- | :--- |
| **Module Canvas** | "God Object" Monolith | ✅ | Aufgeteilt in `controller`, `draw`, `state`, `types`. | `ui/module_canvas/` |
| **Core Module** | Monolithische Logik | ✅ | Refaktorierung abgeschlossen (2026-02-27). | `core/module/` |
| **GPU Uploads** | Thread-Blockierung | ✅ | Async `FramePipeline` implementiert. | `orchestration/media.rs` |
| **wgpu Lifetimes** | Unsafe Transmute | ✅ | Sicherheitsrisiko im Render-Loop behoben (PR #831). | `app/loops/render.rs` |
| **UI App State** | Raw Pointer Hack | ✅ | `*mut App` Pointer entfernt; Refaktoriert auf sichere Referenzen. | `app/loops/render.rs` |

**Status-Legende:** ✅ Erledigt | 🟡 In Arbeit | 🔴 Kritisch/Todo | 🔵 Geplant

---

## 🎨 Feature-Lücken (Code vs. UI)
*Diskrepanzen, bei denen das Backend existiert, aber die UI unvollständig ist.*

- **OSC Triggers**: ✅ Integriert (PR #905). UI-Felder für Cue-Trigger Mapping hinzugefügt.
- **Philips Hue**: Pairing-Logik und Area-Selection sind Stubs. (Status: 🔴)

---

## 📑 Signifikante technische Schulden (Audit 1.0.0-RC1)
*Interne Logik-Probleme und fehlende Validierung.*

### 🚀 Release 1.0.0 Blockers (Priority)
- **Spout Support**: Muss für wgpu 0.19+ (modernisiert auf DX11/DX12 interop) aktualisiert werden. (`crates/mapmap-render/src/spout.rs`) (Status: 🔴)
- **HAP Q Alpha**: Handling für sekundäre Texturen in `hap_decoder.rs` implementieren. (Status: 🔴)
- **About Dialog & Export**: Grundlegende Implementierung für Release-Build erforderlich. (`crates/mapmap/src/app/actions.rs`) (Status: 🔴)

### 📈 Performance & Media
- **Media Thumbnails**: Hintergrund-Generierung für `media_browser.rs` implementieren. (Status: 🔵)
- **Media-Decoder**: VP9/VP8 `Decoder error: Input changed` bei Parameter-Wechsel beheben. (Status: 🟡)

### 🔌 Integration & Control
- **Philips Hue**: DTLS-Verbindung und Pairing-UI finalisieren. (Status: 🔴)
- **Undo/Redo System**: Aktuell nur für Positionen; braucht Parameter-Unterstützung. (Status: 🔵)
- **MCP Server**: Shared State Zugriff für AI-Tools implementieren. (Status: 🟡)

---

## ✅ 1.0.0 Cleanup Audit (05.03.2026)
*Erfolgreich abgeschlossener radikaler Cleanup für Release 1.0.0.*

| Crate | Maßnahme | Status | Ergebnis |
| :--- | :--- | :---: | :--- |
| **mapmap-core** | TODO Resolution & Graph Validation | ✅ | Zyklus-Check & Auto-Disconnect implementiert. Clippy sauber. |
| **mapmap-io** | NDI Activation & Test Stability | ✅ | NDI-Warnungen entfernt, Projekt-Roundtrip verifiziert. |
| **mapmap-render** | Paint Image Loading | ✅ | Synchrones Laden von Bildern in GPU-Cache implementiert. |
| **mapmap-ui** | ID Sync & Clippy Scrub | ✅ | Node-Editor ID-Synchronisation gefixt. Redundante Bindings entfernt. |
| **mapmap-bevy** | Integration Audit | ✅ | Main-Loop und UI-Brücke validiert. |
| **mapmap-mcp** | Protocol Audit | ✅ | JSON-Schnittstellen für AI-Tools stabilisiert. |

---

## 🚀 Refactoring Roadmap: Die "God Files" (März 2026)
*Strategie zur Aufteilung der drei größten monolithischen Dateien.*

### 🟢 Phase 1: Modularisierung des UI-Inspectors
**Fokus:** `crates/mapmap-ui/src/editors/module_canvas/inspector/mod.rs`
- **Status:** ✅ Abgeschlossen. Logik in Submodule aufgeteilt.

### 🔵 Phase 2: Entkopplung des Core Evaluators
**Fokus:** `crates/mapmap-core/src/module_eval.rs`
- **Ziel:** Trennung von Graphentraversierung, Evaluierungs-Zustand und Tests.

### 🔴 Phase 3: Refactoring des MCP-Servers
**Fokus:** `crates/mapmap-mcp/src/server.rs`
- **Ziel:** Trennung der Tool-Definitionen vom Server-Protokoll.

---

### 🛡️ Engineering Standards für Refactoring
- **Sichtbarkeit:** `pub(crate)` für interne Modul-Kommunikation nutzen.
- **Sicherheit:** Jeden `unsafe` Block mit `// SAFETY:` dokumentieren.
- **Validierung:** Jede Session muss `cargo check` und `cargo test` bestehen.

*Zuletzt aktualisiert: 05.03.2026 | Orchestrator: Gemini CLI 🦀*
