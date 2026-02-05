# Project Planning Audit

**Datum:** 2026-01-04

## Roadmap vs. Code Status

### Übereinstimmungen

- **Rebranding (MapFlow):** Roadmap und Changelog zeigen erfolgreichen Abschluss.
- **Layer System:** Implementiert wie geplant.
- **Audio Integration:** AudioAnalyzerV2 und CPAL Backend sind als "Done" markiert und im Code vorhanden.

### Lücken & Diskrepanzen

#### 1. Multi-PC Architektur

- **Roadmap:** Phase 8, diverse Optionen (NDI, Distributed Rendering).
- **Code:** `mapmap-io/src/ndi` existiert, enthält aber "TODO: Implement actual frame sending".
- **Fazit:** Die Implementierung hinkt der Planung hinterher. NDI ist nur ein Skelett.

#### 2. MCP Integration

- **Roadmap:** "MCP-API-Referenz (TODO)".
- **Code:** `mapmap-mcp` Crate existiert und scheint funktional. Die Dokumentation fehlt aber.

#### 3. Assignments System

- **Roadmap:** "Assignment System (PR #140 MERGED)".
- **Code:** `AssignmentManager` existiert. UI ist vorhanden. Scheint konsistent.

#### 4. UI Features

- **Roadmap:** "Icon System (Streamline Ultimate) - Partial".
- **Code:** `window_manager.rs` hatte Probleme mit Icons (jetzt gefixt Agent-seitig).

## Empfehlungen für Roadmap-Updates

1. **NDI Status korrigieren:** Markiere NDI in Roadmap als "In Progress / Experimental" statt implizit fertig.
2. **MCP Docs prio:** Setze "MCP Dokumentation" auf High Priority, da es für Agenten essenziell ist.
3. **Blackscreen Issue aufnehmen:** Das aktuelle Video-Problem sollte als Blocker in der Roadmap auftauchen.

## Offene Klärungspunkte

- Ist `mapmap-control/src/web` (WebSocket) wirklich "NICHT NUTZEN"? Wenn ja, sollte der Code als `deprecated` markiert oder entfernt werden.
- Wie ist der Status von `HAP` Codec? Roadmap sagt "COMPLETED", aber gibt es Tests?
