[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [Alias("TaskDetails")]
    [string]$Prompt,
    [int]$IssueNumber,
    [string]$Repository,
    [string]$Title,
    [string]$SourceName,
    [string]$StartingBranch = "main",
    [string]$ApiKey,
    [switch]$AutoCreatePr,
    [switch]$RequirePlanApproval,
    [bool]$UpdateIssueBody = $true,
    [bool]$PostIssueComment = $true,
    [bool]$ValidateSource = $true,
    [bool]$RemoveTodoUserLabel = $true
)

Set-StrictMode -Version Latest
$ScriptDir = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptDir "jules-api.ps1")
. (Join-Path $ScriptDir "jules-github.ps1")

$resolvedRepository = $null
$issue = $null

if ($IssueNumber -gt 0) {
    $resolvedRepository = Resolve-GitHubRepository -Repository $Repository
    $issue = Get-GitHubIssue -Repository $resolvedRepository -IssueNumber $IssueNumber
    $Prompt = Convert-IssueToJulesPrompt -Issue $issue -Repository $resolvedRepository -AdditionalPrompt $Prompt -AutoCreatePr:$AutoCreatePr.IsPresent
    if ([string]::IsNullOrWhiteSpace($Title)) {
        $Title = "Issue #$($issue.number): $($issue.title)"
    }
} elseif ([string]::IsNullOrWhiteSpace($Prompt)) {
    throw "Bitte entweder -IssueNumber oder -Prompt angeben."
}

if ([string]::IsNullOrWhiteSpace($Title)) {
    $Title = ($Prompt -split "`r?`n" | Select-Object -First 1)
}

$Title = $Title.Trim()
if ($Title.Length -gt 100) {
    $Title = $Title.Substring(0, 100)
}

if ([string]::IsNullOrWhiteSpace($resolvedRepository) -and (-not [string]::IsNullOrWhiteSpace($Repository))) {
    $resolvedRepository = Resolve-GitHubRepository -Repository $Repository
}

$resolvedSourceName = Get-JulesSourceNameForRepository -Repository $resolvedRepository -SourceName $SourceName
if ($ValidateSource) {
    Confirm-JulesSourceExists -SourceName $resolvedSourceName -ApiKey $ApiKey | Out-Null
}

$payload = @{
    prompt = $Prompt
    title = $Title
    sourceContext = @{
        source = $resolvedSourceName
        githubRepoContext = @{
            startingBranch = $StartingBranch
        }
    }
}

if ($AutoCreatePr.IsPresent) {
    $payload["automationMode"] = "AUTO_CREATE_PR"
}

if ($RequirePlanApproval.IsPresent) {
    $payload["requirePlanApproval"] = $true
}

Write-JulesInfo "Erstelle Jules Session fuer '$Title'..."
$createdSession = Invoke-JulesApiRequest -Method POST -Path "sessions" -Body $payload -ApiKey $ApiKey
$sessionId = Resolve-JulesSessionId -SessionIdOrName ([string]$createdSession.name)
$session = Get-JulesSession -SessionIdOrName ([string]$createdSession.name) -ApiKey $ApiKey
if ($null -ne $createdSession -and [string]::IsNullOrWhiteSpace([string]$session.url) -and -not [string]::IsNullOrWhiteSpace([string]$createdSession.url)) {
    $session | Add-Member -NotePropertyName "url" -NotePropertyValue ([string]$createdSession.url) -Force
}
Write-JulesInfo "Session erstellt: $sessionId"

if ($IssueNumber -gt 0 -and $resolvedRepository) {
    if ($UpdateIssueBody) {
        Sync-JulesIssueTracking -Repository $resolvedRepository -IssueNumber $IssueNumber -Session $session -LatestActivity $null -StartingBranch $StartingBranch -SourceName $resolvedSourceName
    }

    if ($PostIssueComment) {
        $commentLines = @(
            "## Jules Session erstellt",
            "",
            ('- Session ID: `{0}`' -f $sessionId),
            ('- Status: `{0}`' -f [string]$session.state),
            "- Session URL: $($session.url)"
        )

        if ($AutoCreatePr.IsPresent) {
            $commentLines += '- Auto-Create-PR: `AUTO_CREATE_PR`'
        }

        if ($RequirePlanApproval.IsPresent) {
            $commentLines += '- Plan Approval: `required`'
        }

        Add-GitHubIssueComment -Repository $resolvedRepository -IssueNumber $IssueNumber -Body ($commentLines -join "`n")
    }

    if ($RemoveTodoUserLabel) {
        Remove-GitHubIssueLabel -Repository $resolvedRepository -IssueNumber $IssueNumber -LabelName "Todo-UserISU"
    }
}

[pscustomobject]@{
    IssueNumber         = if ($IssueNumber -gt 0) { $IssueNumber } else { $null }
    Repository          = $resolvedRepository
    SessionId           = $sessionId
    SessionName         = [string]$session.name
    SessionUrl          = [string]$session.url
    Title               = [string]$session.title
    State               = [string]$session.state
    AutomationMode      = [string]$session.automationMode
    RequirePlanApproval = [bool]$RequirePlanApproval.IsPresent
    SourceName          = $resolvedSourceName
    StartingBranch      = $StartingBranch
}
