# Powershell script to manage PRs
$prs = gh pr list --limit 100 --json number,title,headRefName,mergeable,statusCheckRollup,url | ConvertFrom-Json

foreach ($pr in $prs) {
    Write-Host "Processing PR #$($pr.number): $($pr.title)" -ForegroundColor Cyan
    
    # Check for conflicts
    if ($pr.mergeable -eq "CONFLICTING") {
        Write-Host "  -> CONFLICT DETECTED! Adding to manual fix list." -ForegroundColor Red
        # Logic to add to a list or just output for now
        continue
    }

    # Check status
    $status = "SUCCESS"
    if ($pr.statusCheckRollup -and $pr.statusCheckRollup.Length -gt 0) {
        foreach ($check in $pr.statusCheckRollup) {
            if ($check.conclusion -eq "FAILURE") {
                $status = "FAILURE"
                break
            }
        }
    }

    if ($status -eq "FAILURE") {
        Write-Host "  -> Status FAIL. Commenting @Jules fix..." -ForegroundColor Yellow
        gh pr comment $pr.number --body "@Jules fix"
    } else {
        Write-Host "  -> Status OK or Pending." -ForegroundColor Green
    }

    # Enable Auto-Merge
    Write-Host "  -> Enabling Auto-Merge..."
    gh pr merge $pr.number --auto --merge
}
