# DOC-C5: Output Window Lifecycle Plan

Stand: 2026-03-19

## 1. Zweck

Diese Datei beschreibt den Soll-Zustand fuer den Output-/Window-Lifecycle von MapFlow.

Sie dient als Architekturplan fuer die Konsolidierung zwischen:

- `OutputManager`
- `OutputType::Projector`
- `WindowManager`
- Runtime Render Queue

Wichtig:

- `crates/mapflow/src/window_manager.rs` ist kein normaler Dead-Code-Cleanup-Kandidat.
- Die dort markierten APIs sind teilweise aktive Infrastruktur und teilweise vorbereitete Lifecycle-Bausteine.
- Das Ziel ist Integration und Konsolidierung, nicht blindes Entfernen.

## 2. Aktueller IST-Zustand

### 2.1 Was heute aktiv laeuft

- Das Hauptfenster wird ueber `WindowManager::create_main_window_with_geometry(...)` erzeugt.
- Projector-Fenster werden zur Laufzeit ueber `crates/mapflow/src/orchestration/outputs.rs` erzeugt.
- `sync_output_windows(...)` wird aus `crates/mapflow/src/app/loops/logic.rs` aufgerufen.
- `main.rs` nutzt `WindowManager` bereits fuer Window-zu-Output-Zuordnung, Resize und Redraw.

### 2.2 Wo der Drift liegt

- `OutputManager` verwaltet physische Output-Konfigurationen (`OutputConfig`).
- `OutputType::Projector` traegt zusaetzlich eigene Window-/Output-Felder.
- `WindowManager::sync_windows(...)` bildet den alten `OutputManager`-Pfad ab.
- `sync_output_windows(...)` bildet den neuen `Projector`-Node-Pfad ab.
- Dadurch existieren aktuell zwei teilweise ueberlappende Lifecycle-Konzepte.

### 2.3 Konkrete Probleme

- `sync_output_windows(...)` erzeugt Fenster, entfernt stale Fenster aber nicht.
- `active_window_ids` wird gesammelt, aber nicht fuer Cleanup verwendet.
- `create_output_window(...)` und `sync_windows(...)` sind vorbereitet, aber nicht Teil des aktiven Hauptpfads.
- `WindowContext.output_id` ist derzeit eher Diagnose-/Reserve-Metadatum als aktive Source of Truth.
- UI, Graph und Runtime koennen Output-Aenderungen unterschiedlich interpretieren.

## 3. Architekturentscheidung

### 3.1 Source of Truth

Der Soll-Zustand ist:

- `OutputManager` bleibt die zentrale Registry fuer reale Ausgaenge und deren persistente Konfiguration.
- `OutputType::Projector` beschreibt im Graph nur noch die Bindung auf einen Output und die graphbezogenen Output-Funktionen.
- `WindowManager` besitzt keine fachliche Wahrheit ueber Outputs, sondern nur die Laufzeitverwaltung real geoeffneter Fenster.

Das bedeutet:

- persistente physische Output-Daten gehoeren in `OutputConfig`
- Graph-Routing und graphbezogene Preview-/Output-Flags gehoeren in den Node
- Laufzeitfenster gehoeren in den `WindowManager`

### 3.2 Feldverschiebung

Mittelfristig sollen OS-Window- und physische Display-Parameter nicht doppelt in `OutputConfig` und `OutputType::Projector` leben.

Zielrichtung:

- `OutputConfig` oder ein daran haengender `WindowSpec` traegt physische/lokale Window-Parameter
  - Aufloesung
  - Fullscreen
  - Zielmonitor
  - Cursor-Sichtbarkeit
  - VSync-/Present-Policy
- `OutputType::Projector` traegt graph- und routingbezogene Parameter
  - `id`
  - Preview-Entscheidung
  - zusaetzliche Runtime-/Transport-Flags, falls sie nicht global zum Output gehoeren

## 4. Soll-Lifecycle

Pro Update-Schritt gibt es genau eine Reconciliation-Funktion fuer Output-Fenster.

Diese Funktion muss:

1. den gewuenschten Zustand aus Graph + `OutputManager` ableiten
2. bestehende Fenster mit dem Soll-Zustand vergleichen
3. fehlende Fenster erzeugen
4. geaenderte Fenster rekonfigurieren
5. stale Fenster deterministisch schliessen
6. jeden Repair-/Fallback-Fall sauber loggen

Die Reconciliation arbeitet auf einer expliziten Menge `desired_windows`.

Jedes Element enthaelt mindestens:

- `output_id`
- Window-Typ (`main`, `projector`, ggf. spaeter `preview`)
- physische Window-Konfiguration
- Referenz auf die graphbezogene Output-Nutzung

## 5. Umgang mit bestehender Infrastruktur

### 5.1 `window_manager.rs`

Die Datei bleibt erhalten.

Folgende APIs sind nicht als "weg damit" zu behandeln:

- `create_output_window(...)`
- `create_projector_window(...)`
- `sync_windows(...)`
- `remove_window(...)`
- `main_window_id()`
- `WindowContext.output_id`

Sie muessen im Refactor explizit entweder:

- in den neuen Lifecycle integriert
- vereinheitlicht
- oder nach Abschluss der Konsolidierung gezielt ersetzt werden

### 5.2 `sync_output_windows(...)`

Diese Funktion ist der naheliegende Ort fuer die kuenftige Reconciliation.

Kurzfristig muss sie mindestens:

- stale Fenster anhand von `active_window_ids` entfernen
- fehlende Outputs robust loggen
- nur gueltige und aufloesbare Projector-Outputs verwenden

## 6. Fault Isolation und Logging

Der Output-Lifecycle darf nicht global abstuerzen, wenn:

- ein Output auf eine ungueltige ID zeigt
- ein Zielmonitor nicht mehr verfuegbar ist
- ein Window-Recreate fehlschlaegt
- Window-Konfiguration und Graph nicht zusammenpassen

Pflicht fuer den Soll-Zustand:

- Logging pro `output_id`
- degradierter Weiterlauf statt globalem Abort, wo technisch moeglich
- klare Self-Heal-Logs bei ID-Relinking, Window-Removal oder Fallback auf Primary Monitor

## 7. Umsetzungsreihenfolge

### Phase 1: Modell klarziehen

- Output-Felder zwischen `OutputConfig` und `OutputType::Projector` inventarisieren
- pro Feld entscheiden: Registry, Graph oder Runtime
- Doppelungen markieren und Migrationspfad festlegen

### Phase 2: Lifecycle vereinheitlichen

- `sync_output_windows(...)` zur echten Reconciliation ausbauen
- stale Window-Cleanup vervollstaendigen
- `WindowManager` auf einen klaren Satz aktiver APIs reduzieren

### Phase 3: UI und Graph sauber anbinden

- Output-Panel, Add/Remove-Output und Projector-Node-Erzeugung auf denselben Datenfluss bringen
- Relinking-/Self-Heal-Pfade explizit testen

### Phase 4: Stabilitaet absichern

- Logging schaerfen
- Fehlerszenarien testen
- Automation-/Debug-Smoke-Test weiterhin gruen halten

## 8. Abgrenzung

Nicht Teil dieses Plans:

- neue Output-Backends wie NDI/Spout/Hue end-to-end fertigstellen
- Redesign des Output-Panels
- generelles Multi-Preview-Feature

Diese Themen bauen auf dem vereinheitlichten Lifecycle auf.

## 9. Referenzen

- `crates/mapflow/src/window_manager.rs`
- `crates/mapflow/src/orchestration/outputs.rs`
- `crates/mapflow/src/app/loops/logic.rs`
- `crates/mapflow/src/main.rs`
- `crates/mapflow-core/src/output.rs`
- `crates/mapflow-core/src/module/types/output.rs`
- `docs/A1_SYSTEM/B1_ARCHITECTURE/DOC-C4_RENDER-QUEUE.md`
- `docs/A3_PROJECT/B2_QUALITY/DOC-C10_MODULE_NODE_SYSTEM_AUDIT_2026-03-18.md`
