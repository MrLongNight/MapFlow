# scripts/monitor_mapflow.ps1
# Setze das Encoding fÃ¼r Pipes und Konsole auf UTF-8
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding  = [System.Text.Encoding]::UTF8

Write-Host "--- MapFlow Orchestrator WÃ¤chter (Offizieller Modus) ---" -ForegroundColor Cyan

# Parameter-Verarbeitung
$Interval = 300
$SessionId = $null

for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq "-Interval") { $Interval = [int]$args[$i+1]; $i++ }
    if ($args[$i] -eq "-SessionId") { $SessionId = $args[$i+1]; $i++ }
}

if ($null -eq $SessionId -or $SessionId -eq "") {
    $SessionId = $env:GEMINI_SESSION_ID
}

if ($null -eq $SessionId -or $SessionId -eq "") {
    Write-Host "FEHLER: Keine Session-ID gefunden!"
    exit 1
}

Write-Host "Ziel-Session: $SessionId"
Write-Host "Intervall: $Interval Sekunden"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Monitoring startet..."

while ($true) {
    Write-Host "`n[$(Get-Date -Format 'HH:mm:ss')] === STATUS CHECK START ===" -ForegroundColor Green

    # Sammele Daten
    $prStatus = try { gh pr status | Out-String } catch { "Fehler GitHub Status" }
    $openPrs = try { gh pr list --limit 5 | Out-String } catch { "Fehler GitHub List" }
    $branches = git branch --remotes --no-merged origin/main | Select-Object -First 5 | Out-String

    # Erstelle den Bericht
    $report = @"
------------------------------------------------------------
[MONITOR-UPDATE] $(Get-Date -Format 'HH:mm:ss')
------------------------------------------------------------
GITHUB PR STATUS:
$prStatus

OFFENE PULL REQUESTS (Top 5):
$openPrs

UNMERGED BRANCHES (Top 5):
$branches
------------------------------------------------------------
"@

    # 1. GIB DEN BERICHT DIREKT IN DER KONSOLE AUS (sichtbar im Ctrl+B Fenster)
    Write-Host $report -ForegroundColor Gray

    # 2. SENDE DEN BERICHT AN DEN CHAT (aktualisiert die Historie)
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Sende Update an Chat ($SessionId)..."
    try {
        # Wir geben den Bericht als Argument an gemini -r
        $aiPrompt = "Hier ist ein automatisches Monitor-Update für den Chat-Verlauf. Bitte nimm es zur Kenntnis: `n$report"
        & gemini -r $SessionId "$aiPrompt" --approval-mode yolo --raw-output 2>&1 | Out-Null
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Update erfolgreich gesendet."
    } catch {
        Write-Host "FEHLER beim Senden an Gemini: $_" -ForegroundColor Red
    }

    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] N├ñchster Check in $Interval Sekunden..." -ForegroundColor Yellow
    Start-Sleep -Seconds $Interval
}
