# Project Planning Audit

**Datum:** 2026-01-04

## GitHub Project Issues vs. Code Status

### Übereinstimmungen
- **Rebranding (MapFlow):** Issues und Changelog zeigen erfolgreichen Abschluss.
- **Layer System:** Implementiert wie geplant.
- **Audio Integration:** AudioAnalyzerV2 und CPAL Backend sind als "Abgeschlossen" markiert und im Code vorhanden.

### Lücken & Diskrepanzen

#### 1. Multi-PC Architektur
- **Planung:** Phase 8, diverse Optionen (NDI, Distributed Rendering).
- **Code:** `mapflow-io/src/ndi` existiert, enthält aber "TODO: Implement actual frame sending".
- **Fazit:** Die Implementierung hinkt der Planung hinterher. NDI ist nur ein Skelett.

#### 2. MCP Integration
- **Planung:** "MCP-API-Referenz (Offen)".
- **Code:** `mapflow-mcp` Crate existiert und scheint funktional. Die Dokumentation fehlt aber.

#### 3. Assignments System
- **Planung:** "Assignment System (Abgeschlossen)".
- **Code:** `AssignmentManager` existiert. UI ist vorhanden. Scheint konsistent.

#### 4. UI Features
- **Planung:** "Icon System (Streamline Ultimate) - In Umsetzung".
- **Code:** `window_manager.rs` hatte Probleme mit Icons (jetzt gefixt Agent-seitig).

## Empfehlungen für GitHub Project Issues
1.  **NDI Status korrigieren:** Markiere NDI in den Issues als "In Umsetzung" statt implizit "Abgeschlossen".
2.  **MCP Docs prio:** Setze "MCP Dokumentation" auf High Priority, da es für Agenten essenziell ist.
3.  **Blackscreen Issue aufnehmen:** Das aktuelle Video-Problem sollte als Blocker in den Issues auftauchen.

## Offene Klärungspunkte
- Ist `mapflow-control/src/web` (WebSocket) wirklich "NICHT NUTZEN"? Wenn ja, sollte der Code als `deprecated` markiert oder entfernt werden.
- Wie ist der Status von `HAP` Codec? Planung sagt "Abgeschlossen", aber gibt es Tests?
