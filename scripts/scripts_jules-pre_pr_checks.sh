#!/usr/bin/env bash
# scripts/jules-pre_pr_checks.sh
# Kurzes Vor-PR-Check-Skript für Jules.
# Ausführen im geklonten Repo (Repo-Root). Bricht bei erstem Fehler ab.

set -euo pipefail
IFS=$'\n\t'

log()   { printf '\033[1;34m[INFO]\033[0m %s\n' "$*"; }
err()   { printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; }
ok()    { printf '\033[1;32m[OK]\033[0m %s\n' "$*"; }

# Wenn möglich ins Git-Repo-Root wechseln (falls im Unterordner gestartet)
if git_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  cd "$git_root"
fi

# Prüfungen: Reihenfolge bewusst so, dass schnell abgebrochen wird bei Problemen.

# 0) Prüfe Werkzeuge
if ! command -v cargo >/dev/null 2>&1; then
  err "cargo nicht gefunden. Bitte Rust/Cargo in der VM installieren."
  exit 1
fi

log "1/5: cargo fmt --all"
if ! cargo fmt --all; then
  err "cargo fmt hat Änderungen/Fehler. Ausführen und Änderungen committen."
  exit 2
fi
ok "Formatierung OK"

log "2/5: cargo clippy --all-targets --all-features -- -D warnings"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
  err "Clippy hat Fehler/Warnungen (behandelt als Fehler). Bitte beheben."
  exit 3
fi
ok "Clippy OK"

log "3/5: cargo check --all-targets --all-features"
if ! cargo check --all-targets --all-features; then
  err "cargo check fehlgeschlagen. Bitte Build-Probleme beheben."
  exit 4
fi
ok "Build-Check OK"

log "4/5: cargo test --workspace --all-features"
if ! cargo test --workspace --all-features; then
  err "Tests fehlgeschlagen. Bitte Tests zum Passen bringen."
  exit 5
fi
ok "Tests OK"

log "5/5: git diff --check (whitespace / missing newline checks)"
# git diff --check gibt non-zero bei Problemen; falls kein Git (sehr unwahrscheinlich), überspringen wir sinnvoll
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if ! git diff --check --exit-code; then
    err "Whitespace- oder End-of-file-Probleme (git diff --check). Bitte beheben."
    exit 6
  fi
  ok "Whitespace-Checks OK"
else
  log "Kein Git-Repository erkannt — whitespace-Checks übersprungen."
fi

ok "Alle Checks bestanden. Ready for PR."