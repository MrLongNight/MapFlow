
# Final-Prepare-PreCommit.ps1
# Automates code quality checks and fixes before commit.

Write-Host "🚀 Starting Pre-Commit Preparation..." -ForegroundColor Cyan

# Check if cargo is available
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo Not Found!"
    exit 1
}

# 1. Format Code
Write-Host "📝 Running cargo fmt..." -ForegroundColor Yellow
cargo fmt --all
if ($LASTEXITCODE -ne 0) { Write-Error "Formatting failed!"; exit 1 }

# 2. Sort Dependencies (if cargo-sort is installed)
if (Get-Command "cargo-sort" -ErrorAction SilentlyContinue) {
    Write-Host "📚 Running cargo sort..." -ForegroundColor Yellow
    cargo sort --workspace
} else {
    Write-Warning "cargo-sort not found. Skipping dependency sorting."
}

# 3. Clippy Types/Lints (Auto-Fix)
Write-Host "🛠️ Running cargo clippy (Auto-Fix)..." -ForegroundColor Yellow
# Using settings similar to CI: workspace, all targets, audio feature (common dev feature)
cargo clippy --fix --allow-dirty --allow-staged --workspace --features "mapmap-io/ci-linux" -- -D warnings
if ($LASTEXITCODE -ne 0) { 
    Write-Warning "Clippy found issues that couldn't be automatically fixed. Please check manually."
    # We don't exit here to allow user to see errors, but typically this implies manual intervention needed.
}

# 4. Final Check
Write-Host "✅ Running final cargo check..." -ForegroundColor Yellow
cargo check --workspace --features "mapmap-io/ci-linux"
if ($LASTEXITCODE -ne 0) { Write-Error "Check failed!"; exit 1 }

Write-Host "🎉 Pre-Commit Preparation Complete! You are ready to commit." -ForegroundColor Green
