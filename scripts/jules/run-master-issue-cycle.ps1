[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [int]$MasterIssueNumber,

    [string]$Repository,
    [string]$ApiKey,
    [int]$InitialWaitMinutes = 15,
    [int]$PollMinutes = 5,
    [switch]$AutoCreatePr,
    [switch]$DryRun,
    [string]$GeminiPromptTemplate,
    [string]$GeminiWorktreePath
)

Set-StrictMode -Version Latest
$ScriptDir = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptDir "jules-api.ps1")
. (Join-Path $ScriptDir "jules-github.ps1")

function Write-Step {
    param([string]$Message)
    Write-Host "[MASTER] $Message" -ForegroundColor Cyan
}

function Get-IssueSectionNumbers {
    param(
        [Parameter(Mandatory)][string]$Body,
        [Parameter(Mandatory)][string]$SectionHeading
    )

    $headingPattern = [regex]::Escape("## $SectionHeading")
    $sectionPattern = "(?ms)^$headingPattern\s*$\s*(?<content>.*?)(?=^##\s+|\z)"
    $match = [regex]::Match($Body, $sectionPattern)
    if (-not $match.Success) {
        throw "Sektion '$SectionHeading' wurde im Master-Issue nicht gefunden."
    }

    $numbers = @()
    foreach ($line in (($match.Groups["content"].Value) -split "`r?`n")) {
        if ($line -match '#(?<number>\d+)') {
            $numbers += [int]$Matches.number
        }
    }

    return @($numbers)
}

function Wait-ForJulesCompletion {
    param(
        [Parameter(Mandatory)][string]$SessionId,
        [Parameter(Mandatory)][int]$InitialWaitMinutes,
        [Parameter(Mandatory)][int]$PollMinutes,
        [string]$ApiKey
    )

    Write-Step "Warte $InitialWaitMinutes Minuten vor dem ersten Statuscheck."
    Start-Sleep -Seconds ($InitialWaitMinutes * 60)

    while ($true) {
        $session = Get-JulesSession -SessionIdOrName $SessionId -ApiKey $ApiKey
        Write-Step ("Jules Session {0}: {1}" -f $SessionId, [string]$session.state)
        if ([string]$session.state -eq "COMPLETED") {
            return $session
        }

        if (@("QUEUED", "PLANNING", "AWAITING_PLAN_APPROVAL", "AWAITING_USER_FEEDBACK", "IN_PROGRESS", "PAUSED") -notcontains [string]$session.state) {
            throw "Jules Session $SessionId ended with unexpected state '$($session.state)'."
        }

        Start-Sleep -Seconds ($PollMinutes * 60)
    }
}

function Wait-ForJulesPullRequestUrl {
    param(
        [Parameter(Mandatory)][string]$SessionId,
        [string]$ApiKey,
        [Parameter(Mandatory)][int]$PollMinutes
    )

    while ($true) {
        $session = Get-JulesSession -SessionIdOrName $SessionId -ApiKey $ApiKey
        $prUrl = Get-JulesSessionPullRequestUrl -Session $session
        if ($prUrl) {
            return [string]$prUrl
        }

        Write-Step "PR-Link noch nicht verfuegbar, warte $PollMinutes Minuten."
        Start-Sleep -Seconds ($PollMinutes * 60)
    }
}

function Wait-ForPullRequestMerge {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][string]$PullRequestUrl,
        [Parameter(Mandatory)][int]$PollMinutes
    )

    $prNumber = $PullRequestUrl -replace '.*/pull/(\d+)$', '$1'
    while ($true) {
        $pr = gh pr view $prNumber --repo $Repository --json number,state,mergedAt,url,title | ConvertFrom-Json
        if ($pr.state -eq "MERGED") {
            return $pr
        }

        $checks = @(gh pr checks $prNumber --repo $Repository --json name,state,conclusion 2>$null | ConvertFrom-Json)
        $failing = @($checks | Where-Object { $_.state -eq "FAILED" -or $_.conclusion -eq "FAILURE" })
        if ($failing.Count -gt 0) {
            throw "PR-Checks sind fehlgeschlagen: $((($failing | ForEach-Object { $_.name }) -join ', '))"
        }

        Write-Step "PR noch nicht gemerged. Nächster Check in $PollMinutes Minuten."
        Start-Sleep -Seconds ($PollMinutes * 60)
    }
}

function Invoke-GeminiVerification {
    param(
        [Parameter(Mandatory)][string]$WorktreePath,
        [Parameter(Mandatory)][int]$ImplementationIssueNumber,
        [Parameter(Mandatory)][int]$VerifyIssueNumber,
        [string]$PromptTemplate
    )

    if (-not (Test-Path $WorktreePath)) {
        Write-Step "Erstelle sauberen Gemini-Worktree unter $WorktreePath"
        git worktree add $WorktreePath origin/main | Out-Null
    }

    $prompt = if (-not [string]::IsNullOrWhiteSpace($PromptTemplate)) {
        $PromptTemplate
    } else {
        @"
Verify GitHub issue #$VerifyIssueNumber as the review issue for implementation issue #$ImplementationIssueNumber.
Use the clean repository state at $WorktreePath.
Return a concise PASS or REJECT verdict and cite any blocking file or wording if you reject.
"@
    }

    Push-Location $WorktreePath
    try {
        return (gemini -p $prompt --approval-mode plan --output-format text 2>&1 | Out-String)
    } finally {
        Pop-Location
    }
}

$resolvedRepository = Resolve-GitHubRepository -Repository $Repository
$masterIssue = Get-GitHubIssue -Repository $resolvedRepository -IssueNumber $MasterIssueNumber
$masterBody = if ($null -eq $masterIssue.body) { "" } else { [string]$masterIssue.body }

$implementationNumbers = Get-IssueSectionNumbers -Body $masterBody -SectionHeading "Implementation Subissues (Jules)"
$verificationNumbers = Get-IssueSectionNumbers -Body $masterBody -SectionHeading "Verification Subissues (4-eyes/Review immediately after each implementation)"

if ($implementationNumbers.Count -eq 0) {
    throw "Keine Implementierungs-Subissues im Master-Issue #$MasterIssueNumber gefunden."
}

if ($implementationNumbers.Count -ne $verificationNumbers.Count) {
    throw "Anzahl der Implementierungs- und Verify-Subissues stimmt nicht ueberein: $($implementationNumbers.Count) vs. $($verificationNumbers.Count)."
}

$pairs = @()
$skipped = @()
for ($i = 0; $i -lt $implementationNumbers.Count; $i++) {
    $implNumber = [int]$implementationNumbers[$i]
    $verifyNumber = [int]$verificationNumbers[$i]
    $implIssue = Get-GitHubIssue -Repository $resolvedRepository -IssueNumber $implNumber
    $verifyIssue = Get-GitHubIssue -Repository $resolvedRepository -IssueNumber $verifyNumber

    if ([string]$implIssue.state -eq "CLOSED" -or [string]$verifyIssue.state -eq "CLOSED") {
        $skipped += '{0}:{1}' -f $implNumber, $verifyNumber
        continue
    }

    $pairs += '{0}:{1}' -f $implNumber, $verifyNumber
}

Write-Step ("Master-Issue #{0}: {1}" -f $MasterIssueNumber, $masterIssue.title)
Write-Step ("Aktive Paare: {0}" -f ($pairs -join ', '))
if ($skipped.Count -gt 0) {
    Write-Step ("Uebersprungene Paare: {0}" -f ($skipped -join ', '))
}

if ($DryRun.IsPresent) {
    Write-Step "DRY RUN"
    return
}

foreach ($pair in $pairs) {
    $implNumber, $verifyNumber = $pair -split ':'
    Write-Step ("Starte Paar {0}:{1}" -f $implNumber, $verifyNumber)

    $sessionResult = & "$ScriptDir\create-jules-session.ps1" -IssueNumber ([int]$implNumber) -Repository $resolvedRepository -AutoCreatePr:([bool]$AutoCreatePr) -ApiKey $ApiKey
    $sessionId = [string]$sessionResult.SessionId
    if ([string]::IsNullOrWhiteSpace($sessionId)) {
        throw "Konnte keine Jules Session-ID fuer Issue #$implNumber ermitteln."
    }

    $session = Wait-ForJulesCompletion -SessionId $sessionId -InitialWaitMinutes $InitialWaitMinutes -PollMinutes $PollMinutes -ApiKey $ApiKey
    $pullRequestUrl = Wait-ForJulesPullRequestUrl -SessionId $sessionId -ApiKey $ApiKey -PollMinutes $PollMinutes
    $mergedPr = Wait-ForPullRequestMerge -Repository $resolvedRepository -PullRequestUrl $pullRequestUrl -PollMinutes $PollMinutes

    gh issue comment $implNumber --repo $resolvedRepository --body "Implementation completed. Linked PR: #$($mergedPr.number)"
    gh issue close $implNumber --repo $resolvedRepository | Out-Null

    $geminiWorktree = if (-not [string]::IsNullOrWhiteSpace($GeminiWorktreePath)) { $GeminiWorktreePath } else { Join-Path (Split-Path (Resolve-Path (Join-Path $ScriptDir "..\..")).Path -Parent) "VjMapper-gemini" }
    $geminiResult = Invoke-GeminiVerification -WorktreePath $geminiWorktree -ImplementationIssueNumber ([int]$implNumber) -VerifyIssueNumber ([int]$verifyNumber) -PromptTemplate $GeminiPromptTemplate
    Write-Host $geminiResult

    if ($geminiResult -match '(?im)\bPASS\b') {
        gh issue comment $verifyNumber --repo $resolvedRepository --body "Verification completed for #$implNumber using Gemini CLI. Result: PASS."
        gh issue close $verifyNumber --repo $resolvedRepository | Out-Null
        continue
    }

    gh issue comment $verifyNumber --repo $resolvedRepository --body "Verification result for #$implNumber is not PASS. Review output:`n`n$geminiResult"
    throw ("Gemini verification fuer Paar {0}:{1} ist fehlgeschlagen." -f $implNumber, $verifyNumber)
}
