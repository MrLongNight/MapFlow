# MapFlow – Vollständige Roadmap und Feature-Status

> **Version:** 1.0.0 (Rescue & Recovery Edition)
> **Stand:** 2026-03-05 19:30
> **Status:** Stabilisierungs-Phase nach kritischen Regressionen abgeschlossen.

---

## 📋 Inhaltsverzeichnis
1. [Status-Quo nach Rescue-Session](#status-quo)
2. [Kritische Fehler (Blocker für 1.0.0)](#kritische-fehler)
3. [Geplante Features für Release Candidate 1](#geplante-features)
4. [Langfristige Ziele (V2.0)](#langfristige-ziele)

---

## Status-Quo (Rescue-Report 05.03.2026) {#status-quo}

Nach einer massiven Fehlfunktion in der ersten Maestro-Session wurde das System am 05.03.2026 erfolgreich stabilisiert. 

### ✅ Wiederhergestellte Funktionen:
- **Canvas Node Graph:** Verbindungen zwischen Nodes funktionieren wieder (Radius: 30px).
- **Audio-Analyse:** Echtzeit-Sync zwischen Engine und UI wiederhergestellt.
- **UI-Orchestrierung:** Hauptmenü, Inspector, Sidebar und Timeline sind wieder an ihren Plätzen.
- **Inspector:** Video-Vorschau und grafischer Mesh-Editor sind wieder verfügbar; Breite auf 400px erhöht.
- **Video-Engine:** FFmpeg-Support im Start-Skript aktiviert und DLL-Sync automatisiert.
- **Module Presets:** Neue Funktion zum Speichern von Canvas-Presets hinzugefügt.
- **Level Meter:** Analoge Skala wird nicht mehr abgeschnitten (Min-Höhe 60px).

---

## 🔴 Kritische Fehler & Fehlende Kern-Funktionen (Blocker) {#kritische-fehler}

Die folgenden Punkte müssen zwingend vor einem Release behoben werden:

| Task | Bereich | Status | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Settings-Dialog Rekonstruktion** | UI | 🔴 Kritisch | Fast alle ursprünglichen Einstellungen (OSC, NDI, Audio-Config, I18n) fehlen derzeit im Dialog. |
| **Spout Support Update** | Engine | 🔴 Hoch | Anpassung des Spout-Moduls an die aktuelle wgpu-Version (0.19+). |
| **HAP Q Alpha Support** | Engine | 🟠 Mittel | Korrekte Dekodierung von Alpha-Kanälen für HAP-Videos. |
| **Timeline Interaktion** | UI/Core | 🔴 Hoch | Play/Seek funktionieren, aber Keyframes können im UI noch nicht verschoben oder gelöscht werden. |
| **Export Funktion** | Actions | 🟠 Mittel | Das "Export"-Menü ist derzeit ein Platzhalter ohne Logik. |
| **Digital Meter Polish** | UI | 🟡 Niedrig | Peak-Decay (weiches Abfallen der Spitzen) für das digitale Level-Meter fehlt. |

---

## 🚀 Geplante Features für RC1 {#geplante-features}

- [ ] **Vollständiges Splitting der God-Files:** `menu_bar.rs` und `inspector/mod.rs` müssen chirurgisch in kleinere Module zerlegt werden (AI-Sicherheit).
- [ ] **About-Dialog:** Implementierung des Info-Fensters mit Versionsnummern.
- [ ] **NDI-Discovery UI:** Integration der Quellensuche direkt im Sidebar-Tab.
- [ ] **Shader-Graph Expansion:** Hinzufügen weiterer Node-Typen (Math, Noise, Filter).

---

## 🛠 Technische Schulden (Zusammenfassung) {#langfristige-ziele}

*   **Monolithen:** `mapmap-ui/src/lib.rs` (über 5000 Zeilen) muss modularisiert werden.
*   **Testing:** Es fehlen Integrationstests für das Zusammenspiel von Shader-Graph und Rendering-Pipeline.
*   **Fehler-Handling:** Silent Fails bei Shader-Kompilierung müssen in das UI (Toast-Notifications) geleitet werden.

---

*Zuletzt aktualisiert: 05.03.2026 | Orchestrator: Gemini CLI (Rescue-Mode) 🦀*
