# AGENTS.md - Anweisungen für KI-Agenten

Hallo Jules! Dieses Dokument enthält technische Anweisungen für die Arbeit am VjMapper-Projekt.

## Projektübersicht

VjMapper ist ein Rewrite einer C++/Qt-Anwendung in Rust. Ziel ist eine hochperformante, speichersichere Projection-Mapping-Software. Der gesamte Quellcode befindet sich im `crates/`-Verzeichnis, organisiert als Cargo Workspace.

## Wichtigste Anweisung

**Kommuniziere mit dem Benutzer ausschließlich auf Deutsch.** Alle Pläne, Fragen und Antworten müssen auf Deutsch sein.

> **WICHTIG:**  
> Seit der Einführung von Jules/Copilot und automatischer CI/CD müssen alle Entwicklungen, Automatisierungen, Bugfixes und Tasks **immer in CHANGELOG.md eingetragen werden! Alle Einträge basieren auf Issues, PRs, Roadmap und echten Workflow-Aktivitäten.**

> Code- und Dokumentationsdateien, die durch Issues, Pull Requests, Automatisierungen oder Änderungen (egal ob manuell oder KI-basiert) angepasst werden,  
> **müssen immer als vollständige, neue Datei generiert und ersetzt werden**!  
>  
> _Keine Teilstücke, keine Diffs, keine Patches!_  
>  
> **Nur vollständige, konsistente Dateiinhalte sichern, dass keine zuvor bestehenden Informationen verloren gehen oder versehentlich gelöscht werden.**

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

## Audio-Features und Native Abhängigkeiten

Das Projekt unterstützt optionale Audio-Features für Audio-Reaktivität:

-   **Ohne Audio (Standard):** Das Projekt baut standardmäßig ohne native Audio-Abhängigkeiten:
    ```bash
    cargo build
    cargo test
    ```

-   **Mit Audio-Unterstützung:** Für Audio-Funktionalität müssen native Bibliotheken installiert werden:
    
    **Linux:**
    ```bash
    sudo apt-get update
    sudo apt-get install -y libasound2-dev pkg-config build-essential
    cargo build --features audio
    cargo test --features audio
    ```
    
    **macOS/Windows:** Audio-Features werden derzeit nur unter Linux mit ALSA unterstützt.

-   **CI/CD:** Die CI-Pipeline testet beide Varianten:
    - Linux mit Audio (`--all-features`)
    - Linux ohne Audio (`--no-default-features`)
    - macOS und Windows (ohne Audio)

## Pull Request (PR) Prozess

1.  **Vorbereitung:** Stellen Sie vor dem Einreichen sicher, dass die folgenden Befehle ohne Fehler durchlaufen:
    ```bash
    cargo fmt
    cargo clippy
    cargo test
    ```
2.  **Titel-Format:** Verwenden Sie klare und prägnante Titel, die die Änderungen zusammenfassen.
3.  **Kommunikation:** Erwähnen Sie `@MrLongNight` im PR, falls strategische Fragen offen sind. Feedback von Reviewern wird über PR-Kommentare gegeben.
