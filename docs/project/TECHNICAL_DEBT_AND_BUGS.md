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
| **wgpu Lifetimes** | Unsafe Transmute | 🟡 | Sicherheitsrisiko im Render-Loop. | `app/loops/render.rs` |
| **UI App State** | Raw Pointer Hack | 🟡 | `*mut App` Pointer für UI-Layout-Rendering. | `app/loops/render.rs` |

**Status-Legende:** ✅ Erledigt | 🟡 In Arbeit | 🔴 Kritisch/Todo | 🔵 Geplant

---

## 🎨 Feature-Lücken (Code vs. UI)
*Diskrepanzen, bei denen das Backend existiert, aber die UI unvollständig ist.*

### 📡 Konnektivität & Streaming
- **NDI Receiver**: Backend als Stub vorhanden; UI zeigt Platzhalter. (Status: 🟡)
- **SRT Streaming**: `libsrt` Integration fehlt; Logik-Stubs vorhanden. (Status: 🔴)
- **OSC Triggers**: UI-Felder für Cue-Trigger Mapping fehlen. (Status: 🟡)

### 🎥 Medien & Rendering
- **MPV Decoder**: Rendert graue Platzhalter (libmpv2 Sync-Problem). (Status: 🟡)
- **HAP Alpha**: YCoCg+A Unterstützung ist bei komplexen Dateien instabil. (Status: 🟡)
- **LUT Support**: Core-Logik bereit; "LUT Effect" Node fehlt in der UI. (Status: 🔵)

### 🧩 Node-Spezifische UI
- **Bevy Controls**: UI-Labels für 3D/Particles sind "Not yet implemented". (Status: 🔴)
- **Shader Graph**: Visuelle Verdrahtung für komplexe Mathe fehlt. (Status: 🟡)
- **Philips Hue**: Pairing-Logik und Area-Selection sind Stubs. (Status: 🔴)

---

## 🏗️ Signifikante technische Schulden (TODOs)
*Interne Logik-Probleme und fehlende Validierung.*

- **Undo/Redo System**: Aktuell nur für Positionen; braucht Parameter und Verbindungen.
- **Trigger Smoothing**: `Smoothed` Modus (Attack/Release) ist ein TODO in `module.rs`.
- **Mesh-Import**: Logik zum Laden von OBJ/SVG Dateien fehlt in `module.rs`.
- **Shader Codegen**: Fehlende Parameter-Injektion für Scale, Rotation und Translation.
- **Graphen-Sicherheit**: Zyklenerkennung und Typ-Validierung fehlen in `shader_graph.rs`.
- **MCP Server**: Server kann noch nicht auf den geteilten Projektstatus zugreifen.

---

## 🚀 Refactoring Roadmap: Die "God Files" (März 2026)
*Strategie zur Aufteilung der drei größten monolithischen Dateien.*

### 🟢 Phase 1: Modularisierung des UI-Inspectors
**Fokus:** `crates/mapmap-ui/src/editors/module_canvas/inspector/mod.rs` (~113 KB)
- **Ziel:** Aufteilung der massiven UI-Rendering-Logik in spezialisierte Dateien.
- **Struktur:**
  - `inspector/common.rs`: UI-Hilfsfunktionen & Standardparameter.
  - `inspector/trigger.rs`: UI für `TriggerType` & Cue-Konfiguration.
  - `inspector/source.rs`: UI für Medien, Kamera und Generatoren.
  - `inspector/effect.rs`: UI für Effektketten und Parameter.
  - `inspector/output.rs`: UI für Fenster, NDI und Spout Ziele.
  - `inspector/layer.rs`: UI für Blending-Modi und Ebenen.

### 🔵 Phase 2: Entkopplung des Core Evaluators
**Fokus:** `crates/mapmap-core/src/module_eval.rs` (~72 KB)
- **Ziel:** Trennung von Graphentraversierung, Evaluierungs-Zustand und Tests.
- **Struktur:** `evaluator/types.rs`, `evaluator/graph.rs`, `evaluator/stages/`.

### 🔴 Phase 3: Refactoring des MCP-Servers
**Fokus:** `crates/mapmap-mcp/src/server.rs` (~66 KB)
- **Ziel:** Trennung der Tool-Definitionen vom Server-Protokoll.
- **Struktur:** `server/handlers.rs`, `server/tools/definitions.rs`, `server/tools/impl/`.

---

### 🛡️ Engineering Standards für Refactoring
- **Sichtbarkeit:** `pub(crate)` für interne Modul-Kommunikation nutzen.
- **Sicherheit:** Jeden `unsafe` Block mit `// SAFETY:` dokumentieren.
- **Validierung:** Jede Session muss `cargo check` und `cargo test` bestehen.

*Zuletzt aktualisiert: 01.03.2026 | Orchestrator: Gemini CLI 🦀*