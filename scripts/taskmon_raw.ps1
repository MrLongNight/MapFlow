# scripts/taskmon_raw.ps1
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "=== JULES SESSION LIST (RAW) ===" -ForegroundColor Cyan
gemini --list-sessions | Select-Object -First 10 | Out-String | Write-Host

Write-Host "`n=== GITHUB PR STATUS (RAW) ===" -ForegroundColor Cyan
if (Get-Command gh -ErrorAction SilentlyContinue) {
    gh pr list --limit 10 | Out-String | Write-Host
} else {
    Write-Host "gh CLI nicht verfügbar."
}

Write-Host "`n=== UNMERGED BRANCHES (RAW) ===" -ForegroundColor Cyan
git branch --remotes --no-merged origin/main | Select-Object -First 10 | Out-String | Write-Host
