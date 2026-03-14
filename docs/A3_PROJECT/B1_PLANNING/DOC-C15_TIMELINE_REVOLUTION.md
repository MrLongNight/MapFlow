# DOC-C15_TIMELINE_REVOLUTION: Das neue Mapflow Timeline-Konzept

**Status:** Entwurf / Planung
**Bezug:** Master Issue #1009
**Datum:** 2026-03-13

## 1. Vision
Die Mapflow-Timeline wird von einem einfachen Abspielgerät zu einem hochflexiblen Show-Orchestrator ausgebaut. Sie vereint lineare Zeitabläufe, ereignisbasierte Trigger und komplexe Szenen-Hierarchien in einer intuitiven Multi-Track-Oberfläche.

## 2. Die Hierarchie der Elemente
Um Klarheit im Workflow zu schaffen, definieren wir die Ebenen wie folgt:
1.  **Node:** Die kleinste funktionale Einheit (Effekt, Player, Generator).
2.  **Modul:** Ein verschalteter Graph aus Nodes (eine funktionale Einheit).
3.  **Szene (Scene):** Eine Gruppierung von mehreren Modulen (logische Zusammenfassung für Show-Teile).

## 3. Betriebsmodi (The 4 Modes)
Der Anwender kann je nach Einsatzzweck zwischen verschiedenen Arbeitsweisen wählen:

### A. Timeline Mode (Linear / Automatisiert)
*   **Fokus:** Klassische Show-Abfolge nach Zeitcode.
*   **Verhalten:** Zeitstrahl läuft von links nach rechts. Module und Automationen sind fest auf der Zeitachse verankert.
*   **Anwendung:** Vorproduzierte Musikvideos, Fassadenprojektionen mit festem Soundtrack.

### B. Trackline Mode (Ereignisbasiert / Trigger)
*   **Fokus:** Live-Performance und Theater.
*   **Verhalten:** Die Timeline besteht aus Blöcken ("Points"), zwischen denen manuell oder durch externe Signale gewechselt wird. Der Playhead wartet an definierten Markern auf einen Trigger (MIDI, OSC, Streamdeck, Arduino, App).
*   **Anwendung:** VJing, Theater-Inszenierungen, interaktive Exponate.

### C. Scene Mode (Verschachtelt / Struktur)
*   **Fokus:** Organisation großer Projekte.
*   **Verhalten:** Komplette Szenen (Gruppen von Modulen) werden wie Clips in der Timeline arrangiert. Dies ermöglicht "Sub-Compositions".
*   **Anwendung:** Komplexe Shows mit vielen gleichzeitig aktiven Modulen.

### D. Hybrid Mode (Kombiniert)
*   **Fokus:** Maximale Flexibilität.
*   **Verhalten:** Kombination aus zeitbasierten Sequenzen und manuellen Triggern.
*   **Anwendung:** Live-Konzerte, bei denen Teile fest ablaufen, aber Übergänge manuell gesteuert werden.

## 4. Multi-Track Architektur (Lanes)
Die Timeline wird in spezialisierte Spurentypen unterteilt:

### Modul-Spur (Module Track)
*   Visualisiert die Aktivitätszeiträume von Modul-Blöcken.
*   Ermöglicht Crossfades zwischen aufeinanderfolgenden Modulen.

### Parameter-Spur (Automation Lane)
*   Eine dedizierte Unterspur (aufklappbar) pro Parameter eines Nodes.
*   Hier werden die Keyframe-Kurven (Bezier, Linear, Constant) direkt editiert.
*   Parameter können aus verschiedenen Modulen stammen, um globale Animationen zu ermöglichen.

## 5. Externer Input & Trigger
Ein zentraler "Universal Trigger Router" ermöglicht es, jeden Timeline-Wechsel oder Parameter-Override an externe Quellen zu binden:
*   **MIDI:** Controller-Knöpfe, Fader, Keyboards.
*   **OSC:** Streamdeck, Mobile Apps (TouchOSC), Custom-Software.
*   **GPIO:** Arduino-Sensoren, Schalter, interaktive Hardware.

## 6. Zusammenführung von Cue und Timeline
Cues sind in diesem System keine separate Insel mehr. Ein Cue ist:
*   Entweder ein **Marker** auf der Timeline (Sprungziel).
*   Oder ein **Snapshot** des gesamten Node-Graphen, der als Block auf der Timeline platziert werden kann.
