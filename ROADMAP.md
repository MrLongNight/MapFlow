# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 1.0.0 (Rescue & Reconstruction Edition)
> **Stand:** 2026-03-05 21:00
> **Status:** Stabilisierungs-Phase abgeschlossen. Alle Kern-Regressionen behoben.

---

## 📋 Inhaltsverzeichnis
1. [Status-Quo nach Rescue-Session](#status-quo)
2. [Kritische Fehler (Blocker für 1.0.0)](#kritische-fehler)
3. [Geplante Features für Release Candidate 1](#geplante-features)
4. [Langfristige Ziele (V2.0)](#langfristige-ziele)

---

## Status-Quo (Report 05.03.2026) {#status-quo}

Das System wurde nach massiven Regressionen erfolgreich rekonstruiert und auf eine modulare Architektur umgestellt.

### ✅ Erledigte Reparaturen & Features:
- **UI-Architektur:** Komplette Modularisierung von `MenuBar` und `Inspector`. 
- **UI-Stabilität:** Fix von Layout-Deadlocks durch separate Toolbar-Orchestrierung in `ui_layout.rs`.
- **Canvas Node Graph:** Verbindungen funktionieren wieder einwandfrei (Radius: 30px).
- **Audio-Analyse:** Echtzeit-Sync zwischen Engine und UI repariert; Peak-Decay für Level-Meter implementiert.
- **Show Automation:** Timeline um Modi **Fully Auto**, **Semi Auto** und **Manual** erweitert.
- **HAP Video Engine:** HAP Q Alpha Support implementiert und Syntaxfehler behoben.
- **Settings & About:** Dialoge vollständig rekonstruiert und integriert.
- **CI/CD:** Job01 Validation Fehler (toolchain input & hap_decoder syntax) behoben.

---

## 🔴 Verbleibende Blocker (Prio 1) {#kritische-fehler}

| Task | Bereich | Status | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Hue-Stabilität** | Control | 🟠 In Arbeit | Integration der `HueFlow` Logik für stabilere DTLS-Verbindungen. |
| **Spout Support Update** | Engine | 🔴 Hoch | Anpassung des Spout-Moduls an die aktuelle wgpu-Version (0.19+). |
| **Timeline Interaktion** | UI/Core | 🔴 Hoch | Keyframes können im UI noch nicht verschoben oder gelöscht werden. |

---

## 🚀 Geplante Features für RC1 {#geplante-features}

- [x] **Vollständiges Splitting der God-Files:** `menu_bar.rs` und `inspector/mod.rs` erfolgreich zerlegt.
- [ ] **NDI-Discovery UI:** Integration der Quellensuche direkt im Sidebar-Tab.
- [ ] **Shader-Graph Expansion:** Hinzufügen weiterer Node-Typen (Math, Noise, Filter).

---

## 🛠 Technische Schulden {#langfristige-ziele}

*   **Testing:** 100% Passrate erreicht (400+ Tests). Neue Tests für Timeline-Automation hinzugefügt.
*   **Fehler-Handling:** Implementierung von Toast-Notifications für Engine-Fehler steht noch aus.

---

*Zuletzt aktualisiert: 05.03.2026 | Orchestrator: Gemini CLI (Stabilization Mode) 🦀*
