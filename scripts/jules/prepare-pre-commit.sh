#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREPARE_PRECOMMIT_PROFILE="jules" exec "${SCRIPT_DIR}/../dev-tools/prepare-pre-commit.sh" "$@"
