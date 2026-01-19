#!/bin/bash
set -e

echo "🔍 Running pre-commit checks..."

# 1. Formatierung
echo "  → cargo fmt"
cargo fmt --all

# 2. Clippy
echo "  → cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

# 3. Tests
echo "  → cargo test"
cargo test --workspace

# 4. Unused Dependencies
echo "  → cargo udeps"
cargo +nightly udeps --workspace || true

echo "✅ Pre-commit checks passed!"