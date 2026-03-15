# DOC-B3: Project Management & Audits

Dieses Dokument fasst den Status der technischen Schulden, die Ergebnisse vergangener Audits und die langfristige Phasenplanung zusammen.

## 1. Status der technischen Schulden (März 2026)

Alle kritischen Architektur-Probleme (God Files, Unsafe Pointer) der frühen Phasen wurden erfolgreich behoben. Die verbleibenden Aufgaben sind in die `ROADMAP.md` (`docs/project/roadmap/README.md`) überführt worden.

### Erledigte Altlasten ✅
- **Module Canvas**: Aufgeteilt in saubere Submodule.
- **Evaluator**: Thread-Safe und synchronisiert.
- **GPU Uploads**: Vollständig asynchron.

### Offene Schwerpunkte (siehe Roadmap) 🔴
- **HAP Q Alpha**: Vollständige Unterstützung für Transparenz in HAP-Videos.
- **Spout/NDI**: Performance-Optimierung und stabile Integration.
- **Undo/Redo**: Erweiterung auf alle UI-Parameter.

---

## 2. Audit-Historie

Regelmäßige Audits sichern die Qualität von MapFlow.

| Datum | Typ | Ergebnis |
| :--- | :--- | :--- |
| 05.03.2026 | Release Cleanup | Alle TODOs in mapmap-core gelöst, CI stabilisiert. |
| 29.12.2025 | Code Analysis | Sicherheits-Audit bezüglich Hardcoded Paths (behoben). |
| 15.12.2025 | Documentation | Restrukturierung der docs zur besseren Übersicht (DOC-A0-Standard). |

---

## 3. Phasenplanung (Historisch)

MapFlow wurde in 7 Kernphasen entwickelt:
- **Phase 0-2**: Foundation, Multi-Projector, Shader-System.
- **Phase 3-5**: Control (MIDI/OSC), Assets, ImGui-Interface.
- **Phase 6**: Vollständige Migration auf **egui** (Node-Editor, Timeline).
- **Phase 7**: Performance & Release-Vorbereitung (Aktuell).

---
*Die aktuelle operative Planung erfolgt ausschließlich in der `ROADMAP.md` (jetzt `docs/project/roadmap/README.md`) im Root-Verzeichnis.*
