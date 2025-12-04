# AGENTS.md - Anweisungen für KI-Agenten

Hallo Jules! Dieses Dokument enthält technische Anweisungen für die Arbeit am VjMapper-Projekt.

## Projektübersicht

VjMapper ist ein Rewrite einer C++/Qt-Anwendung in Rust. Ziel ist eine hochperformante, speichersichere Projection-Mapping-Software. Der gesamte Quellcode befindet sich im `crates/`-Verzeichnis, organisiert als Cargo Workspace.

## Wichtigste Anweisung

**Kommuniziere mit dem Benutzer ausschließlich auf Deutsch.** Alle Pläne, Fragen und Antworten müssen auf Deutsch sein.

## Setup & Build-Befehle

-   **Abhängigkeiten installieren:** (Siehe `README.md` für plattformspezifische Bibliotheken)
-   **Projekt bauen (Entwicklung):**
    ```bash
    cargo build
    ```
-   **Projekt bauen (Optimiert für Release):**
    ```bash
    cargo build --release
    ```
-   **Anwendung starten:**
    ```bash
    cargo run --release
    ```

## Code-Stil & Konventionen

-   **Formatierung:** Der Code muss mit `cargo fmt` formatiert werden.
-   **Linting:** Führen Sie `cargo clippy` aus, um häufige Fehler zu vermeiden.
-   **API-Design:** Halten Sie sich an die [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
-   **Dokumentation:** Alle öffentlichen Funktionen und Module müssen mit `///` dokumentiert werden.

## Test-Anweisungen

-   **Alle Tests ausführen:**
    ```bash
    cargo test
    ```
-   **Anforderung:** Fügen Sie für jede neue Funktion oder Fehlerbehebung entsprechende Unit-Tests hinzu. Bestehende Tests dürfen nicht fehlschlagen.

## Pull Request (PR) Prozess

1.  **Vorbereitung:** Stellen Sie vor dem Einreichen sicher, dass die folgenden Befehle ohne Fehler durchlaufen:
    ```bash
    cargo fmt
    cargo clippy
    cargo test
    ```
2.  **Titel-Format:** Verwenden Sie klare und prägnante Titel, die die Änderungen zusammenfassen.
3.  **Kommunikation:** Erwähnen Sie `@MrLongNight` im PR, falls strategische Fragen offen sind. Feedback von Reviewern wird über PR-Kommentare gegeben.
