# Final-Prepare-PreCommit.ps1
Write-Host "Starting Pre-Commit Preparation..." -ForegroundColor Cyan

if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo Not Found!"
    exit 1
}

Write-Host "Running cargo fmt..." -ForegroundColor Yellow
cargo fmt --all

if (Get-Command "cargo-sort" -ErrorAction SilentlyContinue) {
    Write-Host "Running cargo sort..." -ForegroundColor Yellow
    cargo sort --workspace
}

Write-Host "Running cargo clippy..." -ForegroundColor Yellow
cargo clippy --fix --allow-dirty --allow-staged --workspace --features "mapmap-io/ci-linux" -- -D warnings

Write-Host "Running final cargo check..." -ForegroundColor Yellow
cargo check --workspace --features "mapmap-io/ci-linux"

Write-Host "Complete!" -ForegroundColor Green
