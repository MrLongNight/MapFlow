#!/bin/bash
set -e

# Final-Prepare-PreCommit.sh
# Automates code quality checks and fixes before commit (Linux/CI/Jules).

echo -e "\033[0;36m🚀 Starting Pre-Commit Preparation...\033[0m"

# 1. Format Code
echo -e "\033[0;33m📝 Running cargo fmt...\033[0m"
cargo fmt --all

# 2. Sort Dependencies
if command -v cargo-sort &> /dev/null; then
    echo -e "\033[0;33m📚 Running cargo sort...\033[0m"
    cargo sort --workspace
else
    echo -e "\033[0;33m⚠️ cargo-sort not found. Skipping dependency sorting.\033[0m"
fi

# 3. Clippy Auto-Fix
echo -e "\033[0;33m🛠️ Running cargo clippy (Auto-Fix)...\033[0m"
# Using settings similar to CI
cargo clippy --fix --allow-dirty --allow-staged --workspace --features "mapmap-io/ci-linux" -- -D warnings || {
    echo -e "\033[0;31m⚠️ Clippy found issues that couldn't be automatically fixed.\033[0m"
}

# 4. Final Check
echo -e "\033[0;33m✅ Running final cargo check...\033[0m"
cargo check --workspace --features "mapmap-io/ci-linux"

echo -e "\033[0;32m🎉 Pre-Commit Preparation Complete!\033[0m"
