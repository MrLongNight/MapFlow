# Final-Prepare-PreCommit.ps1
Write-Host "🚀 Starting Pre-Commit Preparation..."
cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged --workspace -- -D warnings
cargo check --workspace
Write-Host "🎉 Pre-Commit Preparation Complete!"
