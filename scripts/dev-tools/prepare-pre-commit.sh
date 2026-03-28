#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

PROFILE="${PREPARE_PRECOMMIT_PROFILE:-generic}"

log()  { printf '\033[1;34m[%s]\033[0m %s\n' "$PROFILE" "$*"; }
warn() { printf '\033[1;33m[%s]\033[0m %s\n' "$PROFILE" "$*"; }

if git_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    cd "$git_root"
fi

if ! command -v cargo >/dev/null 2>&1; then
    printf 'cargo not found.\n' >&2
    exit 1
fi

log "Running cargo fmt --all"
cargo fmt --all

if command -v cargo-sort >/dev/null 2>&1; then
    log "Running cargo sort --workspace"
    cargo sort --workspace
else
    warn "cargo-sort not found; skipping dependency sorting"
fi

log "Running cargo clippy --fix"
if ! cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets --features "vorce-io/ci-linux" -- -D warnings; then
    warn "Auto-fix could not solve every clippy issue; running strict validation next"
fi

log "Running strict cargo clippy validation"
cargo clippy --workspace --all-targets --features "vorce-io/ci-linux" -- -D warnings

log "Running cargo check --workspace --all-targets"
cargo check --workspace --all-targets --features "vorce-io/ci-linux"

log "Running git diff --check"
git diff --check --exit-code

log "Pre-commit preparation complete"
