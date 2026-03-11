# Pre-Commit Check Script for Rust Projects
# Run this script before every commit to ensure code quality

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Running Pre-Commit Checks" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorCount = 0

# 1. cargo fmt - Format code
Write-Host "[1/4] Running cargo fmt..." -ForegroundColor Yellow
cargo fmt --all
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: cargo fmt failed!" -ForegroundColor Red
    $ErrorCount++
} else {
    Write-Host "  OK: Code formatted" -ForegroundColor Green
}

# 2. Check for uncommitted format changes
Write-Host ""
Write-Host "[2/4] Checking for format changes..." -ForegroundColor Yellow
$formatChanges = git diff --name-only
if ($formatChanges) {
    Write-Host "  WARNING: cargo fmt made changes to files:" -ForegroundColor Yellow
    $formatChanges | ForEach-Object { Write-Host "    - $_" -ForegroundColor Yellow }
    Write-Host "  Files will be included in the commit." -ForegroundColor Yellow
} else {
    Write-Host "  OK: No format changes needed" -ForegroundColor Green
}

# 3. cargo check - Compile check
Write-Host ""
Write-Host "[3/4] Running cargo check..." -ForegroundColor Yellow
cargo check --all 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: cargo check failed! Run 'cargo check --all' for details." -ForegroundColor Red
    $ErrorCount++
} else {
    Write-Host "  OK: Compilation successful" -ForegroundColor Green
}

# 4. cargo clippy - Linting (warnings only, not errors)
Write-Host ""
Write-Host "[4/4] Running cargo clippy..." -ForegroundColor Yellow
cargo clippy --all 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  WARNING: cargo clippy found issues. Run 'cargo clippy --all' for details." -ForegroundColor Yellow
} else {
    Write-Host "  OK: No clippy warnings" -ForegroundColor Green
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
if ($ErrorCount -eq 0) {
    Write-Host "   All checks passed! Ready to commit." -ForegroundColor Green
    exit 0
} else {
    Write-Host "   $ErrorCount error(s) found. Fix before commit." -ForegroundColor Red
    exit 1
}
