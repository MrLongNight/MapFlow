# scripts/gemini-cli/taskmon-summary.ps1
# Dieses Skript sammelt Informationen für Maestro Task Monitor.

Write-Host "--- Jules Sessions (Gemini CLI) ---" -ForegroundColor Yellow
gemini --list-sessions | Select-Object -First 10

Write-Host "`n--- Offene GitHub Pull Requests ---" -ForegroundColor Yellow
if (Get-Command gh -ErrorAction SilentlyContinue) {
    # Hole PRs im JSON Format für bessere Analyse (optional), hier reicht Tabelle
    gh pr list --limit 10
} else {
    Write-Warning "gh-cli nicht gefunden."
}

Write-Host "`n--- Unmerged Branches ohne PR (PR Manager Fokus) ---" -ForegroundColor Yellow
if (Get-Command gh -ErrorAction SilentlyContinue) {
    # Hole alle Remote Branches, die nicht in main sind
    $branches = git branch --remotes --no-merged origin/main | ForEach-Object { $_.Trim() }

    # Hole alle Branches, die bereits einen PR haben
    $prBranches = gh pr list --json headRefName --jq '.[].headRefName'

    foreach ($branch in $branches) {
        $branchName = $branch -replace "^origin/", ""
        if ($prBranches -notcontains $branchName -and $branchName -ne "origin/HEAD") {
            Write-Host "[!] Branch ohne PR: $branchName" -ForegroundColor Cyan
        }
    }
} else {
    git branch --remotes --no-merged origin/main | Select-Object -First 10
}

Write-Host "`n--- Aktive Mapflow Monitore ---" -ForegroundColor Yellow
$monitors = Get-Process -Name "powershell" -ErrorAction SilentlyContinue | Where-Object {
    $_.CommandLine -like "*monitor_mapflow.ps1*" -or $_.CommandLine -like "*monitor-mapflow.ps1*"
}
if ($monitors) {
    $monitors | Select-Object Id, @{Name="CPU(s)"; Expression={$_.CPU}}, StartTime | Format-Table
} else {
    Write-Host "Keine aktiven Monitore gefunden."
}
