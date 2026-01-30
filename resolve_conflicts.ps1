$prs = @(407, 406, 405, 404, 403, 402, 401, 400, 399, 398, 397, 396, 395, 394, 393, 392, 384, 372)

foreach ($id in $prs) {
    Write-Host "Processing PR #$id..." -ForegroundColor Cyan
    gh pr checkout $id
    if ($LASTEXITCODE -ne 0) { Write-Host "Failed to checkout #$id"; continue }

    $mergeOut = git merge main 2>&1
    if ($mergeOut -match "Already up to date") {
        Write-Host "Already up to date."
        gh pr merge $id --auto --merge
        continue
    }

    # Resolve CI conflicts specifically
    $ci1 = ".github/workflows/CI-01_build&test.yml"
    $ci9 = ".github/workflows/CI-09_create-releases.yml"
    
    # Try to checkout 'theirs' (main) for CI files
    git checkout --theirs $ci1 2>$null
    git checkout --theirs $ci9 2>$null
    
    git add $ci1 $ci9

    # Check for remaining conflicts
    $conflicts = git diff --name-only --diff-filter=U
    if ($conflicts) {
        Write-Host "  -> Complex conflicts detected in: $conflicts" -ForegroundColor Red
        Write-Host "  -> Skipping #$id for manual resolution."
        git merge --abort
        continue
    }

    git commit -m "Merge main into PR #${id}: Resolve CI conflicts"
    if ($LASTEXITCODE -eq 0) {
        git push
        if ($LASTEXITCODE -eq 0) {
            gh pr merge $id --auto --merge
             Write-Host "  -> PR #$id Resolved and Auto-Merge enabled." -ForegroundColor Green
        }
    }
}
