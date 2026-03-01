# 🛰️ MapFlow Projekt-Status & Roadmap

Dieses Dokument ist das zentrale Dashboard für den technischen Zustand von MapFlow.

---

## 🏗️ Architektur & Sicherheit
| Status | Komponente | Problem / Aufgabe | Ort |
| :---: | :--- | :--- | :--- |
| ✅ | **Module Canvas** | Monolith aufgelöst (controller, draw, state). | `ui/module_canvas/` |
| ✅ | **Core Module** | Monolith aufgelöst (manager, config, types). | `core/module/` |
| ✅ | **GPU Uploads** | Async FramePipeline (Micro-Stutter fix). | `orchestration/media.rs` |
| 🟡 | **wgpu Lifetimes** | Unsafe Transmute Hack im Render-Loop. | `app/loops/render.rs` |
| 🟡 | **UI App State** | Raw Pointer (*mut App) für UI Layout. | `app/loops/render.rs` |

---

## 🎨 Feature-Monitor (Backend vs. UI)
| Status | Feature | Beschreibung / Fehlend | Kategorie |
| :---: | :--- | :--- | :--- |
| 🟡 | **NDI Receiver** | Backend als Stub; UI zeigt nur Platzhalter. | Konnektivität |
| 🔴 | **SRT Streaming** | `libsrt` Integration & Logik fehlen komplett. | Konnektivität |
| 🟡 | **OSC Triggers** | UI-Felder für Cue-Mapping fehlen. | Konnektivität |
| 🟡 | **MPV Decoder** | Rendert graue Frames (libmpv2 Sync-Problem). | Medien |
| 🟡 | **HAP Alpha** | YCoCg+A Support instabil bei 4K/Alpha. | Medien |
| 🔵 | **LUT Support** | Core bereit; "LUT Effect" Node fehlt in UI. | Medien |
| 🔴 | **Bevy Nodes** | UI-Steuerung für 3D-Szenen fehlt (Stub). | UI |
| 🟡 | **Shader Graph** | Visuelle Verdrahtung für komplexe Mathe. | UI |
| 🔴 | **Philips Hue** | Pairing-Logik & Area-Selection fehlen. | Hardware |

---

## 🏗️ Technische Schulden & TODOs
| Status | Bereich | Aufgabe / Problem | Datei |
| :---: | :--- | :--- | :--- |
| 🟡 | **Undo/Redo** | Support für Parameter & Verbindungen fehlt. | `core/module.rs` |
| 🔴 | **Smoothing** | TriggerMappingMode::Smoothed (Attack/Rel). | `core/module.rs` |
| 🔴 | **Mesh Import** | Laden von OBJ/SVG Dateien fehlt. | `core/module.rs` |
| 🟡 | **Shader Gen** | Scale/Rot/Trans Parameter-Injektion fehlt. | `core/codegen.rs` |
| 🟡 | **Graph Safety** | Zyklenerkennung & Typ-Validierung fehlen. | `shader_graph.rs` |
| 🔴 | **MCP State** | Zugriff auf geteilten Projektstatus fehlt. | `mcp/server.rs` |
| 🔴 | **Safety Docs** | Fehlende // SAFETY Kommentare bei `unsafe`. | Global |

---

## 🚀 Refactoring Roadmap: "God Files" (März 2026)

### 🟢 Phase 1: UI Inspector Modularisierung
| Status | Datei | Ziel | Umfang |
| :---: | :--- | :--- | :--- |
| 🔵 | `inspector/mod.rs` | Haupteinstiegspunkt & Re-Exports. | ~113 KB |
| 🔵 | `inspector/trigger.rs`| UI für Trigger & Cues. | Neu |
| 🔵 | `inspector/source.rs` | UI für Medien & Generatoren. | Neu |
| 🔵 | `inspector/effect.rs` | UI für Effektketten & Parameter. | Neu |
| 🔵 | `inspector/output.rs` | UI für Fenster, NDI & Spout. | Neu |
| 🔵 | `inspector/layer.rs`  | UI für Blending & Ebenen. | Neu |

### 🔵 Phase 2: Core Evaluator Entkopplung
| Status | Datei | Ziel | Umfang |
| :---: | :--- | :--- | :--- |
| 🔵 | `evaluator/graph.rs` | Caching & Graphentraversierung. | ~72 KB |
| 🔵 | `evaluator/types.rs` | Datenstrukturen & EvaluatorResult. | Neu |
| 🔵 | `evaluator/stages/`  | Logik für Source/Effect/Layer Stufen. | Neu |

### 🔴 Phase 3: MCP Server Refactoring
| Status | Datei | Ziel | Umfang |
| :---: | :--- | :--- | :--- |
| 🔵 | `server/handlers.rs` | Request-Dispatching & Error-Mapping. | ~66 KB |
| 🔵 | `server/tools/`      | Tool-Definitionen (Schemas) & Impl. | Neu |

---

**Legende:** ✅ Erledigt | 🟡 In Arbeit | 🔴 Kritisch/Todo | 🔵 Geplant (Roadmap)

*Zuletzt aktualisiert: 01.03.2026 | Orchestrator: Gemini CLI 🦀*