# 🗂️ Archivist's Journal

Kritische Erkenntnisse aus Repository-Verwaltungsaktivitäten.

---

## Eintragsformat

```
## YYYY-MM-DD - [Titel]
**Erkenntnis:** [Was gelernt]
**Aktion:** [Wie beim nächsten Mal anwenden]
```

---

## 2026-01-28 - Documentation Reorganization

**Erkenntnis:** Mehrere Dokumentationsdateien befanden sich im `.github/` Verzeichnis oder verstreut im Root, anstatt in den dedizierten Ordnern `docs/` oder `.jules/`. Dies widersprach den Archivist-Regeln, dass `.github/` nur für GitHub-spezifische Konfigurationen genutzt werden soll.

**Aktion:**
- `CODE-OF-CONDUCT.md` vom Root nach `.github/` verschoben (Ausnahme, da GitHub-Standard).
- `JULES_*.md` und `SETUP_GUIDE.md` von `.github/` nach `.jules/` verschoben.
- CI/CD und technische Dokumentation (`README_CICD.md`, `GMAIL_API_SETUP.md`, `WORKFLOW_*.md`) von `.github/` nach `docs/08-TECHNICAL/` verschoben.
- `docs/agent_rules/ci_cd_strategy.md` nach `docs/08-TECHNICAL/CI_CD_STRATEGY.md` konsolidiert und `docs/agent_rules/` entfernt.

## 2026-01-02 - Root Directory Cleanup

**Erkenntnis:** Das Root-Verzeichnis enthielt mehrere temporäre Dateien (`check_*.txt`, `test_results.txt`, `core_error.txt`) sowie falsch platzierte Dokumentation (`SECURITY.md`, `knowledge.md`) und redundante Dateien (`VERSION.txt`).

**Aktion:**
- `SECURITY.md` nach `.github/` verschoben.
- `knowledge.md` nach `.jules/` verschoben.
- Temporäre Dateien nach `.temp-archive/2026-01-02-*` archiviert.
- `VjMapper.code-workspace` archiviert (Legacy-Name, nicht erlaubt im Root).

**Zusatz:** Merge-Konflikte in `module.rs`, `main.rs`, `module_eval.rs` behoben (HEAD priorisiert). Syntaxfehler in `module_canvas.rs` korrigiert.

## 2025-01-19 - WGSL Shader Cleanup

**Erkenntnis:** `crates/mapmap-render/shaders/` enthielt 10 `.wgsl` Dateien, die gegen die Projektstruktur verstoßen, da alle Shader in `shaders/` liegen sollten. Dies führte zu einer Inkonsistenz in der Shader-Verwaltung.

**Aktion:**
- Alle `.wgsl` Dateien aus `crates/mapmap-render/shaders/` nach `shaders/` verschoben.
- `crates/mapmap-render/src/effect_chain_renderer.rs` aktualisiert, um die Shader aus dem neuen Pfad (`../../../shaders/`) zu laden.
- `crates/mapmap-render/shaders/` Verzeichnis gelöscht.
- Build mit `cargo check` verifiziert.
