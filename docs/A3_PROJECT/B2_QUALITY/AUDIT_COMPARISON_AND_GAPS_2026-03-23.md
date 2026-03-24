# Vergleichsanalyse & Abweichungen: Aktuelles Audit vs. Archivierte Berichte (März 2026)

Dieses Dokument fasst die Abweichungen zwischen dem aktuellen, durch Subagents erstellten Code-Audit (23.03.2026) und den archivierten Berichten (Stand 18.03.2026 - 21.03.2026) zusammen. Es dient als Entscheidungsgrundlage für die Anpassung der Projektplanung.

---

## 1. Neue Erkenntnisse (Nur im aktuellen Audit identifiziert)

Diese Punkte wurden durch die aktuelle Deep-Scan-Analyse der Subagents neu aufgedeckt und fehlen in den archivierten Berichten oder wurden dort nicht in dieser Schärfe benannt:

### 1.1 Kritischer WGPU-Versionskonflikt (Blocker-Risiko)
*   **Aktueller Befund**: `mapmap-bevy` nutzt **WGPU 24.0**, während der Rest des Workspaces auf **WGPU 27.0** migriert.
*   **Abweichung**: Die Archiv-Berichte (`DOC-C10`, `DOC-C11`) erwähnen zwar allgemeine Render-Probleme und Bevy-Integrationsthemen, identifizieren aber nicht diesen harten Versions-Clash als Kernrisiko für Linker-Fehler und inkompatible Grafik-Pipelines.
*   **Relevanz**: **Hoch**. Dies ist ein technischer Blocker, der sofortige Harmonisierung erfordert.

### 1.2 Unsafe Code & Sicherheits-Defizite
*   **Aktueller Befund**: In `hap_player.rs` und `decoder.rs` fehlen konsequent `// SAFETY:` Dokumentationen. Es wurden riskante Pointer-Dereferenzierungen ohne Null-Prüfung gefunden.
*   **Abweichung**: Die Archiv-Berichte (z. B. `DOC-C12`) bewerten HAP lediglich auf Feature-Ebene ("nicht produktionsreif"). Die mangelhafte Code-Sicherheit und das Risiko von Memory-Corruptions an den FFI-Grenzen wurden dort nicht thematisiert.
*   **Relevanz**: **Hoch**. Sicherheitskritische Mängel im Video-Dekoder können zu Abstürzen bei der Verarbeitung manipulierter Medien führen.

### 1.3 Engine-Isolation & Performance-Flaschenhals
*   **Aktueller Befund**: Die Bevy-Integration erzwingt teure **GPU-CPU-GPU Transfers**, da die Engines isoliert voneinander arbeiten.
*   **Abweichung**: Archiv-Berichte (`DOC-C11`) sprechen zwar von "Render-Hot-Path-Optimierung" (BindGroups etc.), aber die strukturelle Ineffizienz der Frame-Kopien zwischen Bevy und dem Haupt-Renderer wurde nicht als primärer Performance-Flaschenhals isoliert.
*   **Relevanz**: **Hoch** für 4K- und Multi-Output-Szenarien.

---

## 2. Veraltete oder Erledigte Punkte (Im Archiv vorhanden, aktuell nicht mehr kritisch)

Diese Punkte aus den Berichten vom 18.03.2026 wurden in der aktuellen Analyse als bereits adressiert oder verbessert eingestuft:

*   **Node-Graph Dirty-Marking**: `DOC-C10` bemängelte, dass `get_module_mut()` den Graphen sofort dirty markiert. Die aktuelle Code-Analyse zeigt, dass dies (laut Implementierung in `mapflow-core`) entkoppelt wurde.
*   **Doku-Migration**: Der "Loose Files"-Drift aus `DOC-C11` ist weitgehend bereinigt; die Struktur in `docs/` ist nun konsistenter (auch wenn inhaltlicher Drift bei Features bleibt).
*   **Basis-Node-Architektur**: Die in `DOC-C10` geforderten stabilen Schema-Metadaten für Sockets scheinen in den letzten Tagen teilweise implementiert worden zu sein (identifiziert durch den Subagent-Investigator).

---

## 3. Bestätigte & Vertiefte Punkte (Konsens zwischen Audit & Archiv)

Hier decken sich die Analysen, wobei die Archiv-Berichte oft eine höhere Detailtiefe bei der Feature-Matrix bieten:

*   **Feature-Lücken in der Runtime**: Sowohl mein Audit als auch `DOC-C10/C11` bestätigen, dass **Masken, Blend-Modes und Source-Transforms** im Core deklariert, aber in der Runtime weitgehend ignoriert oder nur als Stubs vorhanden sind.
*   **NDI/SRT/HAP Reifegrad**: Die Einstufung als "Experimental/Stub" wird durch alle Berichte gestützt. Die aktuelle Analyse bestätigt, dass `crates/mapflow-io/src/stream/srt.rs` weiterhin ein reiner Code-Stub ist.
*   **Legacy-imgui**: Mein Audit bestätigt die Archiv-Warnung, dass `imgui`-Reste (trotz egui-Migration) den Workspace unnötig belasten.

---

## 4. Relevanz-Einschätzung & Fazit

Die archivierten Berichte (insbesondere `DOC-C10`) sind **extrem relevant**, um die semantische Tiefe des Node-Systems zu verstehen (Socket-IDs vs. Indizes). Mein aktuelles Audit liefert jedoch die **notwendige technologische Korrektur** bezüglich der WGPU-Versionen und der Code-Sicherheit (Unsafe), die in den alten Berichten übersehen wurde.

### Empfohlene Priorisierung für die Planung:
1.  **Technischer Blocker**: Harmonisierung der WGPU-Versionen (Crate `mapmap-bevy`).
2.  **Sicherheit**: Audit der `unsafe`-Blöcke in `mapmap-media` (HAP/FFmpeg) und Ergänzung der Safety-Docs.
3.  **Architektur**: Umstellung von index-basierten auf ID-basierte Sockets (wie in `DOC-C10` vorgeschlagen), um das Node-System robust zu machen.
4.  **Bereinigung**: Entfernung der `imgui`-Legacy-Crates und des Vendor-Codes.

---
*Erstellt durch Gemini CLI Agent (Maestro/Code-Auditor) am 23.03.2026*
