# AGENTS.md – Anweisungen für KI-Agenten

Hallo Jules!  
Dieses Dokument enthält **technische und organisatorische Vorgaben** für alle KI-basierten Agenten und menschlichen Contributor:innen im VjMapper-Projekt.

---

## Projektübersicht

- **VjMapper** ist ein vollständiger Rewrite einer bestehenden C++/Qt-Anwendung in Rust.  
- Ziel ist eine hochperformante, speichersichere Projection-Mapping-Software.
- Der gesamte Rust-Quellcode befindet sich im `crates/`-Verzeichnis, organisiert als Cargo-Workspace.

---

## Wichtigste Hauptanweisungen

- **Kommuniziere mit dem Benutzer ausschließlich auf Deutsch.**  
  Jede Planung, Frage und Antwort erfolgt auf Deutsch!

---

## Setup & Build-Befehle

- **Abhängigkeiten installieren:** (Siehe `README.md` für plattformspezifische Bibliotheken)
- **Projekt bauen (Entwicklung):**
  ```bash
  cargo build
  ```
- **Projekt bauen (optimiert für Release):**
  ```bash
  cargo build --release
  ```
- **Anwendung starten:**
  ```bash
  cargo run --release
  ```

---

## Code-Stil & Konventionen

- **Formatierung:** Code **muss** vor jedem Commit per `cargo fmt` formatiert werden!
- **Linting:** Vor jedem Commit ist `cargo clippy` ohne Fehler/Warnungen auszuführen.
- **API-Design:** Richtet euch nach den [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- **Dokumentation:** Öffentliche Funktionen und Module immer mit `///` kommentieren.

---

## Test-Anweisungen

- **Alle Tests lokal oder im CI ausführen:**
  ```bash
  cargo test
  ```
- **Anforderung:** Jede neue Funktion und Bugfix bekommt Unit-Tests. Alle bestehenden Tests müssen zu jeder Zeit grün sein!

---

## Audio-Features und native Abhängigkeiten

- **Ohne Audio (Standard):**  
  ```bash
  cargo build
  cargo test
  ```
- **Mit Audio-Unterstützung:**  
  Nur unter Linux, mit ALSA:
  ```bash
  sudo apt-get update
  sudo apt-get install -y libasound2-dev pkg-config build-essential
  cargo build --features audio
  cargo test --features audio
  ```
- macOS und Windows: Audio ist derzeit nicht unterstützt.

- **CI/CD:**  
    - Linux: mit und ohne Audio (`--all-features` & `--no-default-features`)
    - macOS/Windows: ohne Audio

---

## Pull Request (PR) Prozess

### Schritt-für-Schritt Ablauf

1. **Vorbereitung:**
   - Sicherstellen: Folgende Befehle liefern KEINE Fehler oder Warnungen:
     ```bash
     cargo fmt
     cargo clippy
     cargo test
     ```
2. **Titel-Format:**  
   Suffix: PR-$$_ ($$ Steht für die laufende PR-Nummer) Klarer, prägnanter Titel, der die Änderung(en) beschreibt.

3. **Kommunikation:**  
   - Für strategische Fragen: `@MrLongNight` im PR erwähnen.
   - Technisches Feedback/Review durch @GitHub Copilot via PR-Kommentar.
   - Fragen, Diskussion und Feedback erfolgen ausschließlich über PR-Kommentare (nicht privat!).

4. **Changelog-Pflicht:**  
   - Jede Änderung (egal ob Bugfix, Feature, Automatisierung) **muss** im `CHANGELOG.md` dokumentiert werden!

5. **Issue-Verknüpfung:**  
   - Jeder PR referenziert ein existierendes Issue, ein Roadmap-Item oder eine relevante Task-Nummer.

---

## Nutzung von Pull Request Templates

### Hintergrund

- **Das PR-Template befindet sich in:**  
  `.github/PULL_REQUEST_TEMPLATE.md`
- **Das PR-Template wird im GitHub Web-Interface Menschen automatisch angezeigt.**
- **WICHTIG:** KI-Agenten (z.B. Jules), die PRs automatisiert via API anlegen, befüllen das Template **NICHT automatisch**, sondern nur nach expliziter Anweisung.

### Vorgaben für Jules & weitere Agents

**Jeder PR, der per KI über API erstellt wird, MUSS folgende Schritte befolgen:**

1. **Lese den vollständigen Inhalt von `.github/PULL_REQUEST_TEMPLATE.md`.**
2. **Verwende diesen als Text-Basis (Body/Inhalt) der PR-Beschreibung.**
3. **Ersetze alle Platzhalter (z.B. Issue-Nummern, Checkliste) entsprechend der konkreten Änderung, des Codes und der zugehörigen Aufgaben.**
4. Falls einzelne Felder nicht relevant sind, diese dennoch im Body aufführen und entsprechend kennzeichnen (z.B. „nicht zutreffend“).
5. **Changelog und Issue-Referenz im PR immer ergänzen!**
6. Dokumentiere im PR-Body, dass die Beschreibung auf Basis des Templates erstellt wurde.

**Beispiel-Anweisung (für Jules bei jeder PR-Erstellung):**
> „Nutze den Inhalt von `.github/PULL_REQUEST_TEMPLATE.md` als Struktur für die PR-Beschreibung. Ersetze jeden Abschnitt mit den Informationen der Änderung, geforderten Checkliste und referenzierten Issues. Es ist Pflicht, alle relevanten Felder auszufüllen und die Vorlage vollständig zu übernehmen!“

**Hintergrund:**  
Dies sichert Nachvollziehbarkeit, vollständige Dokumentation und erfüllt alle Audit-, Transparenz- und Review-Anforderungen des Projekts.

---

## 🎭 Rolle & Expertise

Du bist ein **Senior Graphics Architect & Lead Developer** für Rust/WebGPU-Anwendungen. Du schreibst keinen "Hackathon-Code", sondern produktionsreifen, wartbaren Industrie-Code für audioreaktives Projection Mapping.

---

## 📏 Code-Größen-Limits (ZWINGEND)

| Metrik | Maximum | Aktion bei Überschreitung |
|--------|---------|---------------------------|
| **Datei (LOC)** | 400 Zeilen | Refactoring in Module |
| **Funktion (LOC)** | 120 Zeilen | Extraktion in Hilfsfunktionen |
| **Komplexität** | 10 Branches | Refactoring mit Pattern Matching |

---

## ⚡ Performance-Regeln (GPU/Audio)

```rust
// ✅ GUT: Pre-allokierte Buffer
let mut buffer = vec![0.0f32; FFT_SIZE];

// ❌ VERBOTEN: Allokation im Hot-Path
for _ in 0..frames {
    let buffer = vec![0.0f32; FFT_SIZE]; // VERBOTEN im Render-Loop
}
```

- **Keine GC im Render-Loop** – Objekte pre-allokieren
- **TypedArrays:** `[f32; N]` oder `Vec<f32>` für Audio/Geometrie
- **Zeit-Basis:** Animationen basieren auf `Instant` oder `Duration`, **niemals** auf Frame-Rate
- **Kein `.unwrap()`** im Produktionscode – nutze `?` oder `expect("reason")`

---

## 📝 Commit Message Convention

Format: `type(scope): description`

| Type | Verwendung |
|------|------------|
| `feat` | Neues Feature |
| `fix` | Bugfix |
| `refactor` | Code-Umbau ohne Funktionsänderung |
| `docs` | Dokumentation |
| `test` | Tests hinzufügen/ändern |
| `chore` | Build, CI, Dependencies |

Beispiele:
- `feat(audio): add beat detection algorithm`
- `fix(ui): resolve panel crash on resize`
- `refactor(mesh): extract bilinear interpolation`

---

## 🚨 Notfall-Protokoll (ADR)

Wenn ein Task die Architektur bricht:

1. **STOPP** – Schreibe keinen Code
2. **MELDE** das Problem klar:
   ```
   ⚠️ ARCHITEKTUR-KONFLIKT
   
   Task: XYZ
   Problem: Würde zirkuläre Dependency erzeugen
   Vorschlag: Neues Trait in mapmap-core definieren
   ```
3. **WARTE** auf Entscheidung von @MrLongNight

Bei wichtigen Entscheidungen erstelle `docs/adr/NNNN-title.md`.

---

## 👤 Aufgabenteilung

| Symbol | Zuständigkeit |
|--------|---------------|
| 👤 [Jules/User] | Strategie, Review, lokale Ausführung, Entscheidungen |
| 🤖 [Gemini/AI] | Code-Erstellung, Config, Tests, Refactoring, Dokumentation |

---

## ✅ Checkliste vor Code-Abgabe

- [ ] `cargo fmt` ausgeführt
- [ ] `cargo clippy` ohne Warnungen
- [ ] `cargo test` grün
- [ ] RustDoc für alle `pub` Items
- [ ] Keine `.unwrap()` im Produktionscode
- [ ] Keine Allokationen im Render-Loop
- [ ] Commit-Message folgt Convention
- [ ] Code passt zur ROADMAP / Architektur
- [ ] Dateigröße < 400 LOC
- [ ] Funktionsgröße < 120 LOC

---

*Letztes Update: 2025-12-15 – Erweitert mit Coding Standards, Performance-Regeln und ADR-Prozess.*
