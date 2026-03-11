# scripts/gemini-cli/monitor-mapflow.ps1
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8

Write-Host "--- MapFlow Orchestrator Monitor ---" -ForegroundColor Cyan

$Interval = 300
$SessionId = $null

for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq "-Interval") {
        $Interval = [int]$args[$i + 1]
        $i++
    }
    if ($args[$i] -eq "-SessionId") {
        $SessionId = $args[$i + 1]
        $i++
    }
}

if ([string]::IsNullOrWhiteSpace($SessionId)) {
    $SessionId = $env:GEMINI_SESSION_ID
}

if ([string]::IsNullOrWhiteSpace($SessionId)) {
    Write-Host "ERROR: No session id found." -ForegroundColor Red
    exit 1
}

Write-Host "Target session: $SessionId"
Write-Host "Interval: $Interval seconds"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Monitoring started..."

while ($true) {
    Write-Host "`n[$(Get-Date -Format 'HH:mm:ss')] === STATUS CHECK START ===" -ForegroundColor Green

    $prStatus = try { gh pr status | Out-String } catch { "GitHub status failed." }
    $openPrs = try { gh pr list --limit 5 | Out-String } catch { "GitHub PR list failed." }
    $branches = git branch --remotes --no-merged origin/main | Select-Object -First 5 | Out-String

    $report = @"
------------------------------------------------------------
[MONITOR-UPDATE] $(Get-Date -Format 'HH:mm:ss')
------------------------------------------------------------
GITHUB PR STATUS:
$prStatus

OPEN PULL REQUESTS (TOP 5):
$openPrs

UNMERGED BRANCHES (TOP 5):
$branches
------------------------------------------------------------
"@

    Write-Host $report -ForegroundColor Gray
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Sending update to chat ($SessionId)..."

    try {
        $aiPrompt = "Automated monitor update for chat history. Please acknowledge:`n$report"
        & gemini -r $SessionId "$aiPrompt" --approval-mode yolo --raw-output 2>&1 | Out-Null
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Update sent successfully."
    } catch {
        Write-Host "ERROR while sending to Gemini: $_" -ForegroundColor Red
    }

    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Next check in $Interval seconds..." -ForegroundColor Yellow
    Start-Sleep -Seconds $Interval
}
