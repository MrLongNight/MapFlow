# CI/CD Automatisierungs-Strategie

Diese Strategie definiert die Standards für Continuous Integration und Continuous Delivery im VjMapper Projekt.

## 🛠️ Tool-Stack

| Tool | Zweck | Auto-Fix | Kritisch |
|------|-------|----------|----------|
| `cargo fmt` | Code-Formatierung | ✅ Ja | ✅ Ja |
| `cargo clippy` | Linting & Best Practices | ⚠️ Teilweise | ✅ Ja |
| `cargo nextest` | Schnellere Tests | ❌ Nein | ✅ Ja |
| `cargo audit` | Sicherheits-Audit | ❌ Nein | ✅ Ja |
| `cargo deny` | Lizenz & Supply Chain | ❌ Nein | ⚠️ Optional |
| `cargo udeps` | Unused Dependencies | ❌ Nein | ⚠️ Optional |
| `cargo tarpaulin` | Code Coverage | ❌ Nein | ⚠️ Optional |
| `cargo vet` | Dependency Vetting | ❌ Nein | ⚠️ Optional |
| `cargo-sort` | Cargo.toml Sorting | ✅ Ja | ⚠️ Optional |

## 🔄 Workflow Phasen (CI-01)

### 1. Pre-Checks & Auto-Fixes
- **Ziel**: Einfache Fehler automatisch beheben bevor teure Tests laufen.
- **Tools**: `cargo fmt`, `cargo clippy --fix`, `cargo-sort`.
- **Aktion**: Commit & Push von Fixes durch `github-actions[bot]`.

### 2. Code Quality & Linting
- **Ziel**: Statische Analyse und Einhaltung von Standards.
- **Tools**: `clippy` (via reviewdog für PR-Kommentare), `cargo-udeps`, `cargo-deny`.
- **Cache**: `Swatinem/rust-cache` (Shared Key: `quality-cache`).

### 3. Build & Test
- **Ziel**: Funktionale Korrektheit sicherstellen.
- **Tools**: `cargo nextest` (Parallelisierung), `cargo tarpaulin` (Coverage).
- **Environment**: Ubuntu Latest mit Audio/Video-Libs (FFmpeg, NDI SDK).

### 4. Windows Build
- **Ziel**: Cross-Platform Kompatibilität prüfen.
- **Einschränkung**: Ohne NDI SDK (manuelle Interaktion nötig), eingeschränkte Tests.

### 5. Security & Supply Chain
- **Ziel**: Sicherheitslücken in Dependencies finden.
- **Tools**: `cargo audit`, `cargo vet`, `dependabot`.

### 6. Performance
- **Ziel**: Regressionen verhindern.
- **Tools**: `cargo bench` (nur bei PRs relevant).

## 💻 Lokale Entwicklung

### Pre-Commit Hooks
Einrichtung empfohlen via Skript:
```bash
ln -s .github/pre-commit-hook.sh .git/hooks/pre-commit
```

### Cargo Make
Verwendung von `Makefile.toml` zur lokalen Ausführung der Pipeline:
```bash
cargo make check-all  # Format, Lint, Test
cargo make ci-local   # Full Pipeline inkl. Docs & Audit
```

## 🤖 Agenten Regeln

1. **Kein direkter Push auf Main**: Immer Feature-Branches und PRs nutzen.
2. **CI-Fixes**: Bei CI-Fehlern zuerst lokal `cargo make ci-local` oder `cargo check` ausführen.
3. **Neue Dependencies**: Immer `cargo sort` und `cargo deny check licenses` beachten.
4. **Dokumentation**: Änderungen an der Pipeline hier dokumentieren.
