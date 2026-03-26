param(
    [string]$Profile = "generic"
)

$ErrorActionPreference = "Stop"

function Write-Step([string]$Message) {
    Write-Host "[$Profile] $Message" -ForegroundColor Cyan
}

if ($gitRoot = git rev-parse --show-toplevel 2>$null) {
    Set-Location $gitRoot.Trim()
}

if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo not found."
    exit 1
}

Write-Step "Running cargo fmt --all"
cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (Get-Command "cargo-sort" -ErrorAction SilentlyContinue) {
    Write-Step "Running cargo sort --workspace"
    cargo sort --workspace
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} else {
    Write-Warning "[$Profile] cargo-sort not found; skipping dependency sorting."
}

Write-Step "Running cargo clippy --fix"
cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets --features "vorce-io/ci-linux" -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Warning "[$Profile] Auto-fix did not clear all clippy issues; running strict validation next."
}

Write-Step "Running strict cargo clippy validation"
cargo clippy --workspace --all-targets --features "vorce-io/ci-linux" -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step "Running cargo check --workspace --all-targets"
cargo check --workspace --all-targets --features "vorce-io/ci-linux"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step "Running git diff --check"
git diff --check --exit-code
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step "Pre-commit preparation complete"
