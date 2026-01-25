# 🌍 Internationalisierung (i18n) - Implementierungsplan

> **Status:** Geplant  
> **Priorität:** Niedrig (nach UI-Stabilisierung)  
> **Geschätzter Aufwand:** 3-5 Entwicklertage  
> **Ziel:** MapFlow soll mehrere Sprachen in der Benutzeroberfläche unterstützen

---

## 📋 Zusammenfassung

### Aktueller Stand
- ❌ **Keine i18n-Unterstützung** im Rust-Rewrite
- ❌ Alle UI-Texte sind als englische Strings hardcoded
- ⚠️ Legacy `. ts`-Dateien (Qt Linguist) im `/translations`-Ordner sind **nicht nutzbar**
- ⚠️ Die alten Übersetzungsdateien können nur als **Textvorlage** dienen

### Ziel
- ✅ Mehrsprachige UI (mindestens: Englisch, Deutsch, Französisch)
- ✅ Einfaches Hinzufügen neuer Sprachen durch Community
- ✅ Automatische Spracherkennung basierend auf Systemeinstellungen
- ✅ Manuelle Sprachauswahl in den Einstellungen

---

## 🛠️ Technische Entscheidung

### Empfohlenes Crate: `rust-i18n`

| Kriterium | rust-i18n | fluent-rs | gettext-rs |
|-----------|-----------|-----------|------------|
| Einfachheit | ⭐⭐⭐ | ⭐⭐ | ⭐ |
| Compile-Time Checks | ✅ | ❌ | ❌ |
| YAML/TOML Support | ✅ | ❌ (eigenes Format) | ❌ (PO-Dateien) |
| Macro-basiert | ✅ | ❌ | ❌ |
| ImGui-kompatibel | ✅ | ✅ | ✅ |
| Community | Aktiv | Sehr aktiv | Stabil |

**Entscheidung:** `rust-i18n` v3 wegen Einfachheit und Compile-Time-Validierung.

---

## 📁 Geplante Dateistruktur

```
crates/
└── mapmap-ui/
    ├── Cargo.toml          # + rust-i18n dependency
    ├── src/
    │   └── lib.rs          # i18n!  macro initialization
    └── locales/
        ├── en.yml          # Englisch (Basis)
        ├── de.yml          # Deutsch
        ├── fr.yml          # Französisch
        ├── es.yml          # Spanisch
        ├── zh-CN.yml       # Chinesisch (vereinfacht)
        └── zh-TW.yml       # Chinesisch (traditionell)
```

---

## 📝 Implementierungsschritte

### Phase 1: Grundgerüst (Tag 1)

#### 1.1 Dependency hinzufügen
```toml
# crates/mapmap-ui/Cargo. toml
[dependencies]
rust-i18n = "3"
sys-locale = "0.3"  # Für automatische Spracherkennung
```

#### 1.2 Macro initialisieren
```rust
// crates/mapmap-ui/src/lib.rs
rust_i18n::i18n! ("locales", fallback = "en");

pub fn init_locale() {
    // Automatische Erkennung der Systemsprache
    if let Some(locale) = sys_locale::get_locale() {
        let lang = locale.split('-').next().unwrap_or("en");
        rust_i18n::set_locale(lang);
    }
}
```

#### 1.3 Basis-Übersetzungsdatei erstellen
```yaml
# crates/mapmap-ui/locales/en.yml
en:
  # Fenster-Titel
  window:
    playback_controls: "Playback Controls"
    transform_controls: "Transform Controls"
    layers: "Layers"
    paints: "Paints"
    mappings: "Mappings"
    color_calibration: "Color Calibration"
    outputs: "Outputs"
    edge_blending: "Edge Blending"

  # Buttons
  button:
    play: "Play"
    pause: "Pause"
    stop: "Stop"
    add: "Add"
    remove: "Remove"
    duplicate: "Duplicate"
    reset: "Reset"
    apply: "Apply"
    cancel: "Cancel"
    save: "Save"
    load: "Load"

  # Labels
  label:
    speed: "Speed"
    opacity: "Opacity"
    position: "Position"
    rotation: "Rotation"
    scale: "Scale"
    brightness: "Brightness"
    contrast: "Contrast"
    saturation: "Saturation"

  # Menü
  menu:
    file: "File"
    edit: "Edit"
    view: "View"
    help: "Help"
    new_project: "New Project"
    open_project: "Open Project..."
    save_project: "Save Project"
    save_as: "Save As..."
    import_media: "Import Media..."
    exit: "Exit"
    undo: "Undo"
    redo: "Redo"
    preferences: "Preferences..."
    about: "About MapFlow"

  # Statusmeldungen
  status:
    loading: "Loading..."
    saving: "Saving..."
    ready: "Ready"
    error: "Error"
    no_layer_selected: "No layer selected."
    no_output_selected: "No output selected."
```

---

### Phase 2: String-Extraktion (Tag 2-3)

#### Vorher (aktueller Code)
```rust
ui.button("Play");
ui.text("Playback Controls");
ui.slider("Speed", 0.1, 2.0, &mut self.playback_speed);
```

#### Nachher (mit i18n)
```rust
use rust_i18n::t;

ui.button(t! ("button.play"));
ui.text(t! ("window.playback_controls"));
ui.slider(t! ("label.speed"), 0.1, 2.0, &mut self.playback_speed);
```

#### Betroffene Dateien
| Datei | Geschätzte Strings | Aufwand |
|-------|-------------------|---------|
| `crates/mapmap-ui/src/lib.rs` | ~150 | 4h |
| `crates/mapmap-ui/src/dashboard.rs` | ~30 | 1h |
| `crates/mapmap/src/main.rs` | ~20 | 1h |
| **Gesamt** | ~200 | 6h |

---

### Phase 3: Übersetzungen (Tag 4)

#### Deutsche Übersetzung (`de.yml`)
```yaml
de:
  window:
    playback_controls: "Wiedergabesteuerung"
    transform_controls: "Transformationen"
    layers: "Ebenen"
    paints: "Quellen"
    mappings: "Mappings"
    color_calibration: "Farbkalibrierung"
    outputs: "Ausgänge"
    edge_blending: "Kantenüberblendung"

  button:
    play: "Abspielen"
    pause: "Pause"
    stop: "Stopp"
    add: "Hinzufügen"
    remove: "Entfernen"
    duplicate: "Duplizieren"
    reset: "Zurücksetzen"
    apply: "Anwenden"
    cancel: "Abbrechen"
    save: "Speichern"
    load: "Laden"

  label:
    speed: "Geschwindigkeit"
    opacity: "Deckkraft"
    position: "Position"
    rotation: "Drehung"
    scale: "Skalierung"
    brightness: "Helligkeit"
    contrast: "Kontrast"
    saturation: "Sättigung"

  menu:
    file: "Datei"
    edit: "Bearbeiten"
    view: "Ansicht"
    help: "Hilfe"
    new_project: "Neues Projekt"
    open_project: "Projekt öffnen..."
    save_project: "Projekt speichern"
    save_as: "Speichern unter..."
    import_media: "Medien importieren..."
    exit: "Beenden"
    undo: "Rückgängig"
    redo: "Wiederholen"
    preferences: "Einstellungen..."
    about: "Über MapFlow"

  status:
    loading: "Wird geladen..."
    saving: "Wird gespeichert..."
    ready: "Bereit"
    error: "Fehler"
    no_layer_selected: "Keine Ebene ausgewählt."
    no_output_selected: "Kein Ausgang ausgewählt."
```

---

### Phase 4: Sprachauswahl-UI (Tag 5)

```rust
// In Preferences-Dialog
ui.combo("##language", &mut self.selected_language, &[
    "English",
    "Deutsch",
    "Français",
    "Español",
    "中文 (简体)",
    "中文 (繁體)",
], |lang| std::borrow::Cow::Borrowed(*lang));

if ui.button(t! ("button.apply")) {
    let locale = match self.selected_language {
        0 => "en",
        1 => "de",
        2 => "fr",
        3 => "es",
        4 => "zh-CN",
        5 => "zh-TW",
        _ => "en",
    };
    rust_i18n::set_locale(locale);
    // Speichern in Config
}
```

---

## 🗑️ Legacy-Dateien

### Empfehlung: Löschen oder verschieben

Die alten Qt-Übersetzungsdateien im `/translations`-Ordner sollten:

**Option A (empfohlen):** Löschen
```bash
rm -rf translations/
```

**Option B:** In Legacy-Ordner verschieben
```bash
mv translations/ legacy/qt-translations/
```

Die Texte aus den `. ts`-Dateien können manuell als Referenz für neue YAML-Übersetzungen verwendet werden.

---

## ✅ Akzeptanzkriterien

- [ ] App startet mit Systemsprache (falls unterstützt)
- [ ] Fallback auf Englisch bei unbekannter Sprache
- [ ] Alle UI-Elemente sind übersetzt
- [ ] Sprachauswahl in Einstellungen funktioniert
- [ ] Neue Sprachen können durch YAML-Datei hinzugefügt werden
- [ ] Compile-Time-Fehler bei fehlenden Übersetzungsschlüsseln
- [ ] Dokumentation für Übersetzer vorhanden

---

## 🚀 Wann implementieren?

| Bedingung | Status |
|-----------|--------|
| UI-Layout stabil | ⏳ In Arbeit |
| Kern-Features fertig | ⏳ In Arbeit |
| App umbenannt zu MapFlow | ⏳ Ausstehend |
| Community-Interesse | ✅ Vorhanden |

**Empfohlener Zeitpunkt:** Nach Phase 2 Abschluss, wenn die UI weitgehend stabil ist.

---

## 📚 Ressourcen

- [rust-i18n Dokumentation](https://github.com/longbridgeapp/rust-i18n)
- [sys-locale Crate](https://crates.io/crates/sys-locale)
- [Fluent Project](https://projectfluent.org/) (Alternative)
- [ImGui Font Loading](https://github. com/ocornut/imgui/blob/master/docs/FONTS.md) (für CJK-Zeichen)

---

*Erstellt: 2025-12-04*  
*Autor: GitHub Copilot*  
*Review: @MrLongNight*
