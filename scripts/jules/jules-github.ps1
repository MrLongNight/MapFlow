Set-StrictMode -Version Latest

$script:JulesIssueBlockStart = "<!-- jules-session:begin -->"
$script:JulesIssueBlockEnd = "<!-- jules-session:end -->"
$script:ManagedIssueStatusLabels = @(
    "status: in-progress",
    "status: blocked",
    "status: needs-review"
)
$script:VorceProjectContextCache = @{}

function Assert-GitHubCli {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        throw "gh CLI wurde nicht gefunden."
    }
}

function Get-RepositoryFromGitRemote {
    $remoteUrl = git config --get remote.origin.url 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($remoteUrl)) {
        return $null
    }

    if ($remoteUrl -match "github\.com[:/](?<owner>[^/]+)/(?<repo>[^/.]+?)(?:\.git)?$") {
        return "{0}/{1}" -f $Matches["owner"], $Matches["repo"]
    }

    return $null
}

function Resolve-GitHubRepository {
    param([string]$Repository)

    if (-not [string]::IsNullOrWhiteSpace($Repository)) {
        return $Repository.Trim()
    }

    $detected = Get-RepositoryFromGitRemote
    if (-not [string]::IsNullOrWhiteSpace($detected)) {
        return $detected
    }

    throw "GitHub-Repository konnte nicht ermittelt werden. Bitte -Repository owner/repo angeben."
}

function Get-JulesSourceNameForRepository {
    param([string]$Repository, [string]$SourceName)

    if (-not [string]::IsNullOrWhiteSpace($SourceName)) {
        return $SourceName.Trim()
    }

    return "sources/github/$(Resolve-GitHubRepository -Repository $Repository)"
}

function Invoke-GitHubApiJson {
    param(
        [Parameter(Mandatory)][string[]]$Arguments,
        [AllowNull()][object]$Body,
        [switch]$AllowEmptyResponse
    )

    Assert-GitHubCli

    $tempFile = $null
    try {
        $allArgs = @($Arguments)
        if ($PSBoundParameters.ContainsKey("Body")) {
            $tempFile = [System.IO.Path]::GetTempFileName()
            $jsonBody = $Body | ConvertTo-Json -Depth 50
            $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
            [System.IO.File]::WriteAllText($tempFile, $jsonBody, $utf8NoBom)
            $allArgs += @("--input", $tempFile)
        }

        $output = & gh @allArgs 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw (($output | Out-String).Trim())
        }

        $text = ($output | Out-String).Trim()
        if ([string]::IsNullOrWhiteSpace($text)) {
            if ($AllowEmptyResponse) { return $null }
            return $null
        }

        return $text | ConvertFrom-Json
    } finally {
        if ($tempFile -and (Test-Path $tempFile)) {
            Remove-Item -Path $tempFile -Force
        }
    }
}

function Get-GitHubIssue {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber)

    Assert-GitHubCli
    $output = & gh issue view $IssueNumber --repo $Repository --json number,title,body,url,state,updatedAt,labels 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw (($output | Out-String).Trim())
    }

    return (($output | Out-String) | ConvertFrom-Json)
}

function Get-GitHubIssues {
    param([Parameter(Mandatory)][string]$Repository, [ValidateSet("open", "closed", "all")][string]$State = "open", [int]$Limit = 200)

    Assert-GitHubCli
    $output = & gh issue list --repo $Repository --state $State --limit $Limit --json number,title,body,url,state,updatedAt,labels 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw (($output | Out-String).Trim())
    }

    $items = (($output | Out-String) | ConvertFrom-Json)
    if ($null -eq $items) {
        return @()
    }

    return @($items)
}

function Get-GitHubPullRequest {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [string]$PullRequestUrl,
        [int]$PullRequestNumber
    )

    Assert-GitHubCli

    $target = $null
    if (-not [string]::IsNullOrWhiteSpace($PullRequestUrl)) {
        $target = $PullRequestUrl.Trim()
    } elseif ($PullRequestNumber -gt 0) {
        $target = [string]$PullRequestNumber
    } else {
        return $null
    }

    $output = & gh pr view $target --repo $Repository --json number,title,url,state,isDraft,headRefName,updatedAt,mergeable,reviewDecision,labels 2>&1
    if ($LASTEXITCODE -ne 0) {
        return $null
    }

    return (($output | Out-String) | ConvertFrom-Json)
}

function Get-GitHubPullRequestChecks {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [string]$PullRequestUrl,
        [int]$PullRequestNumber
    )

    Assert-GitHubCli

    $target = $null
    if (-not [string]::IsNullOrWhiteSpace($PullRequestUrl)) {
        $target = $PullRequestUrl.Trim()
    } elseif ($PullRequestNumber -gt 0) {
        $target = [string]$PullRequestNumber
    } else {
        return @()
    }

    $output = & gh pr checks $target --repo $Repository --required --json bucket,name,state,workflow,startedAt,completedAt 2>&1
    if ($LASTEXITCODE -ne 0) {
        return @()
    }

    $items = (($output | Out-String) | ConvertFrom-Json)
    if ($null -eq $items) {
        return @()
    }

    return @($items)
}

function Find-GitHubPullRequestForIssue {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][int]$IssueNumber,
        [AllowNull()][string]$SessionId
    )

    Assert-GitHubCli

    $queries = @()
    if (-not [string]::IsNullOrWhiteSpace($SessionId)) {
        $queries += ('"{0}" in:body' -f $SessionId.Trim())
    }

    $queries += @(
        ('"Fixes #{0}" in:body' -f $IssueNumber),
        ('"Closes #{0}" in:body' -f $IssueNumber),
        ('"Resolves #{0}" in:body' -f $IssueNumber),
        ('"#{0}" in:body' -f $IssueNumber)
    )

    foreach ($query in ($queries | Select-Object -Unique)) {
        $output = & gh pr list --repo $Repository --state all --search $query --json number,title,url,state,isDraft,headRefName,updatedAt,mergeable,reviewDecision,labels 2>&1
        if ($LASTEXITCODE -ne 0) {
            continue
        }

        $items = (($output | Out-String) | ConvertFrom-Json)
        if ($null -eq $items) {
            continue
        }

        $matches = @($items)
        if ($matches.Count -eq 0) {
            continue
        }

        $ordered = @(
            $matches |
                Sort-Object {
                    try {
                        [datetimeoffset]([string]$_.updatedAt)
                    } catch {
                        [datetimeoffset]::MinValue
                    }
                } -Descending
        )
        if ($ordered.Count -gt 0) {
            return $ordered[0]
        }
    }

    return $null
}

function Get-GitHubIssueComments {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber)

    $comments = Invoke-GitHubApiJson -Arguments @("api", "repos/$Repository/issues/$IssueNumber/comments")
    if ($null -eq $comments) { return @() }
    return @($comments)
}

function Set-GitHubIssueBody {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [Parameter(Mandatory)][string]$Body)

    Invoke-GitHubApiJson -Arguments @("api", "repos/$Repository/issues/$IssueNumber", "--method", "PATCH") -Body @{ body = $Body } -AllowEmptyResponse | Out-Null
}

function Add-GitHubIssueComment {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [Parameter(Mandatory)][string]$Body)

    Invoke-GitHubApiJson -Arguments @("api", "repos/$Repository/issues/$IssueNumber/comments", "--method", "POST") -Body @{ body = $Body } | Out-Null
}

function Add-GitHubIssueLabels {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [string[]]$LabelNames)

    $labels = @($LabelNames | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
    if ($labels.Count -eq 0) {
        return
    }

    Invoke-GitHubApiJson -Arguments @("api", "repos/$Repository/issues/$IssueNumber/labels", "--method", "POST") -Body @{ labels = $labels } -AllowEmptyResponse | Out-Null
}

function Remove-GitHubIssueLabel {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [Parameter(Mandatory)][string]$LabelName)

    try {
        Invoke-GitHubApiJson -Arguments @("api", "repos/$Repository/issues/$IssueNumber/labels/$([uri]::EscapeDataString($LabelName))", "--method", "DELETE") -AllowEmptyResponse | Out-Null
    } catch {
        Write-JulesWarn "Label '$LabelName' konnte nicht entfernt werden: $($_.Exception.Message)"
    }
}

function Get-GitHubIssueLabelNames {
    param([AllowNull()][object]$Issue)

    if ($null -eq $Issue -or -not $Issue.labels) {
        return @()
    }

    return @(
        $Issue.labels |
            ForEach-Object {
                if ($_ -is [string]) { $_ } else { [string]$_.name }
            } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            Select-Object -Unique
    )
}

function Normalize-TrackingText {
    param([AllowNull()][object]$Value, [int]$MaxLength = 180)

    if ($null -eq $Value) {
        return $null
    }

    $text = [string]$Value
    if ([string]::IsNullOrWhiteSpace($text)) {
        return $null
    }

    $text = $text.Replace("`r", " ").Replace("`n", " ").Replace("`t", " ").Replace("`"", "'").Trim()
    $text = [regex]::Replace($text, "\s+", " ")

    if ($MaxLength -gt 0 -and $text.Length -gt $MaxLength) {
        $text = $text.Substring(0, $MaxLength - 3).TrimEnd() + "..."
    }

    return $text
}

function Format-TrackingTimestamp {
    param([AllowNull()][string]$Timestamp)

    if ([string]::IsNullOrWhiteSpace($Timestamp)) {
        return $null
    }

    try {
        return ([datetimeoffset]$Timestamp).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    } catch {
        return $Timestamp.Trim()
    }
}

function Resolve-LatestTrackingTimestamp {
    param([string[]]$Candidates)

    $bestParsed = $null
    $bestText = $null

    foreach ($candidate in @($Candidates)) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }

        try {
            $parsed = [datetimeoffset]$candidate
            if ($null -eq $bestParsed -or $parsed -gt $bestParsed) {
                $bestParsed = $parsed
                $bestText = $parsed.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            }
        } catch {
            if ($null -eq $bestText) {
                $bestText = $candidate.Trim()
            }
        }
    }

    return $bestText
}

function Get-VorceQueueState {
    param([AllowNull()][object]$Issue, [AllowNull()][object]$Session)

    if ($null -eq $Issue) {
        return "unknown"
    }

    if ([string]$Issue.state -eq "CLOSED") {
        return "closed"
    }

    $labels = Get-GitHubIssueLabelNames -Issue $Issue
    if ($labels -contains "Todo-UserISU") {
        return "user-review"
    }

    if ($null -eq $Session) {
        if ($labels -contains "jules-task") {
            return "approved-awaiting-dispatch"
        }

        return "issue-only"
    }

    return "dispatched"
}

function Get-VorceRemoteState {
    param([AllowNull()][object]$Issue, [AllowNull()][object]$Session, [AllowNull()][object]$PullRequest)

    if ($null -ne $PullRequest) {
        switch ([string]$PullRequest.state) {
            "MERGED" { return "merged" }
            "OPEN" { return "pr-open" }
        }
    }

    if ($null -eq $Session) {
        if ($null -ne $Issue) {
            $labels = Get-GitHubIssueLabelNames -Issue $Issue
            if ($labels -contains "Todo-UserISU") {
                return "not-started"
            }
            if ($labels -contains "jules-task") {
                return "awaiting-session"
            }
            if ([string]$Issue.state -eq "CLOSED") {
                return "closed"
            }
        }

        return "issue-only"
    }

    switch ([string]$Session.state) {
        "QUEUED" { return "queued" }
        "PLANNING" { return "planning" }
        "AWAITING_PLAN_APPROVAL" { return "awaiting-plan-approval" }
        "AWAITING_USER_FEEDBACK" { return "awaiting-user-feedback" }
        "IN_PROGRESS" { return "in-progress" }
        "PAUSED" { return "paused" }
        "FAILED" { return "failed" }
        "COMPLETED" { return "completed" }
        default { return (Normalize-TrackingText -Value ([string]$Session.state) -MaxLength 60) }
    }
}

function Get-VorceNeedsAttention {
    param([AllowNull()][object]$Issue, [AllowNull()][object]$Session, [AllowNull()][object]$PullRequest)

    if ($null -ne $Issue -and [string]$Issue.state -eq "CLOSED") {
        return "no"
    }

    if ($null -ne $PullRequest -and @("MERGED", "CLOSED") -contains [string]$PullRequest.state) {
        return "no"
    }

    if ($null -ne $Session) {
        if (@("AWAITING_PLAN_APPROVAL", "AWAITING_USER_FEEDBACK", "PAUSED", "FAILED") -contains [string]$Session.state) {
            return "yes"
        }
    }

    if ($null -ne $PullRequest -and [string]$PullRequest.reviewDecision -eq "CHANGES_REQUESTED") {
        return "yes"
    }

    return "no"
}

function Get-VorceWorkBranch {
    param([AllowNull()][object]$PullRequest, [string]$StartingBranch)

    if ($null -ne $PullRequest -and -not [string]::IsNullOrWhiteSpace([string]$PullRequest.headRefName)) {
        return [string]$PullRequest.headRefName
    }

    if (-not [string]::IsNullOrWhiteSpace($StartingBranch)) {
        return $StartingBranch.Trim()
    }

    return $null
}

function Get-VorceLastActivitySummary {
    param([AllowNull()][object]$Issue, [AllowNull()][object]$Session, [AllowNull()][object]$LatestActivity)

    $summary = Normalize-TrackingText -Value (Get-JulesActivitySummary -Activity $LatestActivity) -MaxLength 180
    if (-not [string]::IsNullOrWhiteSpace($summary)) {
        return $summary
    }

    if ($null -ne $Session) {
        switch ([string]$Session.state) {
            "QUEUED" { return "Session wartet in der Queue." }
            "PLANNING" { return "Jules erstellt aktuell den Plan." }
            "AWAITING_PLAN_APPROVAL" { return "Wartet auf Plan-Freigabe." }
            "AWAITING_USER_FEEDBACK" { return "Wartet auf Rueckmeldung." }
            "IN_PROGRESS" { return "Jules arbeitet am Issue." }
            "PAUSED" { return "Session ist pausiert." }
            "FAILED" { return "Session ist fehlgeschlagen." }
            "COMPLETED" { return "Session abgeschlossen." }
        }
    }

    if ($null -ne $Issue) {
        $labels = Get-GitHubIssueLabelNames -Issue $Issue
        if ($labels -contains "Todo-UserISU") {
            return "Issue wartet auf Freigabe vor dem Dispatch."
        }
    }

    return "Noch keine Remote-Aktivitaet erfasst."
}

function Get-DesiredIssueStatusLabels {
    param([AllowNull()][object]$Issue, [AllowNull()][object]$Session, [AllowNull()][object]$PullRequest)

    if ($null -eq $Issue -or [string]$Issue.state -eq "CLOSED") {
        return @()
    }

    $labels = Get-GitHubIssueLabelNames -Issue $Issue
    if ($labels -contains "Todo-UserISU") {
        return @()
    }

    if ($null -ne $PullRequest -and [string]$PullRequest.state -eq "OPEN") {
        return @("status: needs-review")
    }

    if ($null -ne $Session -and @("AWAITING_PLAN_APPROVAL", "AWAITING_USER_FEEDBACK", "PAUSED", "FAILED") -contains [string]$Session.state) {
        return @("status: blocked")
    }

    if ($null -ne $Session) {
        return @("status: in-progress")
    }

    return @()
}

function Sync-GitHubIssueStatusLabels {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][int]$IssueNumber,
        [Parameter(Mandatory)][object]$Issue,
        [string[]]$DesiredLabels
    )

    $existing = Get-GitHubIssueLabelNames -Issue $Issue
    $desired = @($DesiredLabels | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)

    foreach ($managedLabel in $script:ManagedIssueStatusLabels) {
        if (($existing -contains $managedLabel) -and ($desired -notcontains $managedLabel)) {
            Remove-GitHubIssueLabel -Repository $Repository -IssueNumber $IssueNumber -LabelName $managedLabel
        }
    }

    $missing = @($desired | Where-Object { $existing -notcontains $_ })
    if ($missing.Count -gt 0) {
        Add-GitHubIssueLabels -Repository $Repository -IssueNumber $IssueNumber -LabelNames $missing
    }
}

function Invoke-GitHubGraphQl {
    param([Parameter(Mandatory)][string]$Query, [hashtable]$Variables)

    $body = @{ query = $Query }
    if ($Variables) {
        $body["variables"] = $Variables
    }

    $response = Invoke-GitHubApiJson -Arguments @("api", "graphql", "--method", "POST") -Body $body
    if ($null -ne $response -and $response.PSObject.Properties.Name -contains "errors" -and $null -ne $response.errors) {
        $messages = @($response.errors | ForEach-Object { Normalize-TrackingText -Value $_.message -MaxLength 200 })
        throw ("GitHub GraphQL Fehler: {0}" -f ($messages -join " | "))
    }

    return $response.data
}

function Get-VorceProjectConfig {
    param([Parameter(Mandatory)][string]$Repository)

    $projectNumberValue = $env:VORCE_PROJECT_NUMBER
    if ([string]::IsNullOrWhiteSpace($projectNumberValue)) {
        $repositoryParts = (Resolve-GitHubRepository -Repository $Repository).Split("/")
        $projectOwnerFallback = if (-not [string]::IsNullOrWhiteSpace($env:MAPFLOW_PROJECT_OWNER)) {
            $env:MAPFLOW_PROJECT_OWNER.Trim()
        } else {
            $repositoryParts[0]
        }

        try {
            $projectList = Invoke-GitHubApiJson -Arguments @("project", "list", "--owner", $projectOwnerFallback, "--format", "json")
            $projects = if ($null -eq $projectList) { @() } else { @($projectList.projects) }
            $preferred = @(
                $projects |
                    Where-Object {
                        $_ -and
                        ($_.closed -eq $false) -and
                        ([string]$_.title -eq "@MapFlow Project Manager")
                    } |
                    Select-Object -First 1
            )

            if ($preferred.Count -gt 0) {
                $projectNumberValue = [string]$preferred[0].number
            } else {
                $openProjects = @($projects | Where-Object { $_ -and ($_.closed -eq $false) })
                if ($openProjects.Count -eq 1) {
                    $projectNumberValue = [string]$openProjects[0].number
                }
            }
        } catch {
            Write-JulesWarn "GitHub Project konnte nicht automatisch erkannt werden: $($_.Exception.Message)"
        }

        if ([string]::IsNullOrWhiteSpace($projectNumberValue)) {
            return $null
        }
    }

    $projectNumber = 0
    if (-not [int]::TryParse($projectNumberValue, [ref]$projectNumber) -or $projectNumber -le 0) {
        Write-JulesWarn "VORCE_PROJECT_NUMBER ist ungueltig und wird ignoriert."
        return $null
    }

    $repositoryParts = (Resolve-GitHubRepository -Repository $Repository).Split("/")
    $projectOwner = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_OWNER)) {
        $env:VORCE_PROJECT_OWNER.Trim()
    } else {
        $repositoryParts[0]
    }

    return @{
        Owner              = $projectOwner
        Number             = $projectNumber
        StatusFieldName    = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_STATUS_FIELD)) { $env:VORCE_PROJECT_STATUS_FIELD.Trim() } else { "Status" }
        QueueFieldName     = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_QUEUE_STATE_FIELD)) { $env:VORCE_PROJECT_QUEUE_STATE_FIELD.Trim() } else { "Queue State" }
        JulesSessionStatusFieldName = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_JULES_SESSION_STATUS_FIELD)) { $env:VORCE_PROJECT_JULES_SESSION_STATUS_FIELD.Trim() } else { "jules_session_status" }
        PrChecksStatusFieldName = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_PR_CHECKS_STATUS_FIELD)) { $env:VORCE_PROJECT_PR_CHECKS_STATUS_FIELD.Trim() } else { "pr_checks_status" }
        WorkBranchFieldName = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_WORK_BRANCH_FIELD)) { $env:VORCE_PROJECT_WORK_BRANCH_FIELD.Trim() } else { "Work Branch" }
        LastUpdateFieldName = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_LAST_UPDATE_FIELD)) { $env:VORCE_PROJECT_LAST_UPDATE_FIELD.Trim() } else { "Last Update" }
        LinkedPrFieldName   = if (-not [string]::IsNullOrWhiteSpace($env:VORCE_PROJECT_LINKED_PR_FIELD)) { $env:VORCE_PROJECT_LINKED_PR_FIELD.Trim() } else { "Linked PR" }
    }
}

function Get-VorceProjectContext {
    param([Parameter(Mandatory)][string]$Repository)

    $config = Get-VorceProjectConfig -Repository $Repository
    if ($null -eq $config) {
        return $null
    }

    $cacheKey = "{0}#{1}" -f $config.Owner, $config.Number
    if ($script:VorceProjectContextCache.ContainsKey($cacheKey)) {
        return $script:VorceProjectContextCache[$cacheKey]
    }

    $userQuery = @'
query($owner: String!, $number: Int!) {
  user(login: $owner) {
    projectV2(number: $number) {
      id
      title
      fields(first: 100) {
        nodes {
          __typename
          ... on ProjectV2Field {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
          }
        }
      }
    }
  }
}
'@

    $orgQuery = @'
query($owner: String!, $number: Int!) {
  organization(login: $owner) {
    projectV2(number: $number) {
      id
      title
      fields(first: 100) {
        nodes {
          __typename
          ... on ProjectV2Field {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
          }
        }
      }
    }
  }
}
'@

    $project = $null
    try {
        $userData = Invoke-GitHubGraphQl -Query $userQuery -Variables @{
            owner  = $config.Owner
            number = $config.Number
        }
        if ($null -ne $userData.user.projectV2) {
            $project = $userData.user.projectV2
        }
    } catch {
        $project = $null
    }

    if ($null -eq $project) {
        try {
            $orgData = Invoke-GitHubGraphQl -Query $orgQuery -Variables @{
                owner  = $config.Owner
                number = $config.Number
            }
            if ($null -ne $orgData.organization.projectV2) {
                $project = $orgData.organization.projectV2
            }
        } catch {
            $project = $null
        }
    }

    if ($null -eq $project) {
        throw "GitHub Project V2 '$($config.Owner)#$($config.Number)' wurde nicht gefunden."
    }

    $fieldsByName = @{}
    foreach ($field in @($project.fields.nodes)) {
        if ($null -eq $field -or [string]::IsNullOrWhiteSpace([string]$field.name)) {
            continue
        }

        $dataType = if ($field.PSObject.Properties.Name -contains "dataType" -and -not [string]::IsNullOrWhiteSpace([string]$field.dataType)) {
            [string]$field.dataType
        } elseif ([string]$field.__typename -eq "ProjectV2SingleSelectField") {
            "SINGLE_SELECT"
        } else {
            [string]$field.__typename
        }

        $fieldsByName[[string]$field.name] = [pscustomobject]@{
            Id       = [string]$field.id
            Name     = [string]$field.name
            DataType = $dataType
            Options  = $(if ($field.PSObject.Properties.Name -contains "options") { @($field.options) } else { @() })
        }
    }

    $context = [pscustomobject]@{
        Owner              = $config.Owner
        Number             = $config.Number
        ProjectId          = [string]$project.id
        Title              = [string]$project.title
        StatusFieldName    = [string]$config.StatusFieldName
        QueueFieldName     = [string]$config.QueueFieldName
        JulesSessionStatusFieldName = [string]$config.JulesSessionStatusFieldName
        PrChecksStatusFieldName = [string]$config.PrChecksStatusFieldName
        WorkBranchFieldName = [string]$config.WorkBranchFieldName
        LastUpdateFieldName = [string]$config.LastUpdateFieldName
        LinkedPrFieldName   = [string]$config.LinkedPrFieldName
        FieldsByName       = $fieldsByName
        ItemIdsByContentId = @{}
        ItemMapLoaded      = $false
    }

    $script:VorceProjectContextCache[$cacheKey] = $context
    return $context
}

function Get-GitHubIssueContentId {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber)

    $repositoryParts = (Resolve-GitHubRepository -Repository $Repository).Split("/")
    $query = @'
query($owner: String!, $repo: String!, $issueNumber: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $issueNumber) {
      id
    }
  }
}
'@

    $data = Invoke-GitHubGraphQl -Query $query -Variables @{
        owner       = $repositoryParts[0]
        repo        = $repositoryParts[1]
        issueNumber = $IssueNumber
    }

    return [string]$data.repository.issue.id
}

function Initialize-VorceProjectItemMap {
    param([Parameter(Mandatory)][object]$Context)

    if ($Context.ItemMapLoaded) {
        return
    }

    $itemsByContentId = @{}
    $cursor = $null

    do {
        $query = @'
query($projectId: ID!, $cursor: String) {
  node(id: $projectId) {
    ... on ProjectV2 {
      items(first: 100, after: $cursor) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          id
          content {
            __typename
            ... on Issue {
              id
            }
          }
        }
      }
    }
  }
}
'@

        $data = Invoke-GitHubGraphQl -Query $query -Variables @{
            projectId = $Context.ProjectId
            cursor    = $cursor
        }

        $items = @($data.node.items.nodes)
        foreach ($item in $items) {
            if ($null -eq $item.content -or [string]$item.content.__typename -ne "Issue") {
                continue
            }

            $itemsByContentId[[string]$item.content.id] = [string]$item.id
        }

        $pageInfo = $data.node.items.pageInfo
        $cursor = if ($pageInfo.hasNextPage) { [string]$pageInfo.endCursor } else { $null }
    } while (-not [string]::IsNullOrWhiteSpace($cursor))

    $Context.ItemIdsByContentId = $itemsByContentId
    $Context.ItemMapLoaded = $true
}

function Ensure-VorceProjectItem {
    param([Parameter(Mandatory)][object]$Context, [Parameter(Mandatory)][string]$IssueContentId)

    Initialize-VorceProjectItemMap -Context $Context

    if ($Context.ItemIdsByContentId.ContainsKey($IssueContentId)) {
        return [string]$Context.ItemIdsByContentId[$IssueContentId]
    }

    $mutation = @'
mutation($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
    item {
      id
    }
  }
}
'@

    $data = Invoke-GitHubGraphQl -Query $mutation -Variables @{
        projectId = $Context.ProjectId
        contentId = $IssueContentId
    }

    $itemId = [string]$data.addProjectV2ItemById.item.id
    $Context.ItemIdsByContentId[$IssueContentId] = $itemId
    return $itemId
}

function Get-VorceProjectField {
    param([AllowNull()][object]$Context, [string]$FieldName)

    if ($null -eq $Context -or [string]::IsNullOrWhiteSpace($FieldName)) {
        return $null
    }

    if ($Context.FieldsByName.ContainsKey($FieldName)) {
        return $Context.FieldsByName[$FieldName]
    }

    return $null
}

function Resolve-ProjectSingleSelectOption {
    param([AllowNull()][object[]]$Options, [string[]]$Candidates)

    $optionList = @($Options | Where-Object { $null -ne $_ -and -not [string]::IsNullOrWhiteSpace([string]$_.name) })
    if ($optionList.Count -eq 0) {
        return $null
    }

    foreach ($candidate in @($Candidates)) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }

        $exact = @($optionList | Where-Object { [string]$_.name -ieq $candidate } | Select-Object -First 1)
        if ($exact.Count -gt 0) {
            return $exact[0]
        }
    }

    foreach ($candidate in @($Candidates)) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }

        $match = @(
            $optionList |
                Where-Object {
                    ([string]$_.name).ToLowerInvariant().Contains($candidate.ToLowerInvariant()) -or
                    $candidate.ToLowerInvariant().Contains(([string]$_.name).ToLowerInvariant())
                } |
                Select-Object -First 1
        )
        if ($match.Count -gt 0) {
            return $match[0]
        }
    }

    return $null
}

function Get-VorceProjectStatusCandidateNames {
    param([Parameter(Mandatory)][hashtable]$Fields)

    if (@("closed", "done", "completed", "merged") -contains (([string]$Fields.QueueState).Trim()).ToLowerInvariant()) {
        return @("Done", "Completed", "Closed", "Merged")
    }

    if (([string]$Fields.IssueState).Trim().ToUpperInvariant() -eq "CLOSED") {
        return @("Done", "Completed", "Closed", "Merged")
    }

    if (@("merged", "completed", "closed") -contains [string]$Fields.RemoteState) {
        return @("Done", "Completed", "Closed", "Merged")
    }

    if ([string]$Fields.NeedsAttention -eq "yes") {
        return @("Blocked", "On Hold", "Needs Input", "In Progress")
    }

    if ([string]$Fields.PullRequestUrl) {
        return @("In Review", "Review", "Needs Review", "In Progress")
    }

    if ([string]$Fields.QueueState -in @("user-review", "approved-awaiting-dispatch", "issue-only")) {
        return @("Todo", "Backlog", "Ready", "Inbox")
    }

    return @("In Progress", "Doing", "Active")
}

function Get-VorceJulesSessionStatusValue {
    param(
        [AllowNull()][object]$Session,
        [AllowNull()][object]$Issue
    )

    if ($null -ne $Issue -and [string]$Issue.state -eq "CLOSED") {
        return "completed"
    }

    if ($null -ne $Session) {
        switch ([string]$Session.state) {
            "QUEUED" { return "queued" }
            "PLANNING" { return "planning" }
            "IN_PROGRESS" { return "running" }
            "AWAITING_PLAN_APPROVAL" { return "waiting" }
            "AWAITING_USER_FEEDBACK" { return "waiting" }
            "PAUSED" { return "waiting" }
            "FAILED" { return "failed" }
            "COMPLETED" { return "completed" }
            default { return "unknown" }
        }
    }

    if ($null -ne $Issue) {
        $labels = Get-GitHubIssueLabelNames -Issue $Issue
        if ($labels -contains "jules-task") {
            return "not_started"
        }
    }

    return "n_a"
}

function Get-VorcePrChecksStatusValue {
    param(
        [AllowNull()][object]$PullRequest,
        [AllowNull()][object[]]$Checks
    )

    if ($null -eq $PullRequest) {
        return "n_a"
    }

    if ([string]$PullRequest.state -eq "MERGED") {
        return "merged"
    }

    if ([string]$PullRequest.state -eq "CLOSED") {
        return "closed"
    }

    if ($PullRequest.PSObject.Properties.Name -contains "isDraft" -and [bool]$PullRequest.isDraft) {
        return "draft"
    }

    $checkList = @($Checks)
    if ($checkList.Count -eq 0) {
        return "pending"
    }

    $failed = @(
        $checkList |
            Where-Object {
                [string]$_.bucket -eq "fail" -or
                @("FAILURE", "FAILED", "ERROR", "TIMED_OUT", "CANCELLED", "ACTION_REQUIRED") -contains ([string]$_.state).ToUpperInvariant()
            }
    )
    if ($failed.Count -gt 0) {
        return "failed"
    }

    $pending = @(
        $checkList |
            Where-Object {
                @("PENDING", "QUEUED", "IN_PROGRESS", "STARTUP_FAILURE", "WAITING") -contains ([string]$_.state).ToUpperInvariant()
            }
    )
    if ($pending.Count -gt 0) {
        return "pending"
    }

    return "passed"
}

function Set-VorceProjectFieldValue {
    param(
        [Parameter(Mandatory)][object]$Context,
        [Parameter(Mandatory)][string]$ItemId,
        [AllowNull()][object]$Field,
        [AllowNull()][string]$Value
    )

    if ($null -eq $Field) {
        return
    }

    if ([string]::IsNullOrWhiteSpace($Value)) {
        $mutation = @'
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!) {
  clearProjectV2ItemFieldValue(input: { projectId: $projectId, itemId: $itemId, fieldId: $fieldId }) {
    projectV2Item {
      id
    }
  }
}
'@

        Invoke-GitHubGraphQl -Query $mutation -Variables @{
            projectId = $Context.ProjectId
            itemId    = $ItemId
            fieldId   = $Field.Id
        } | Out-Null
        return
    }

    $normalizedDataType = [string]$Field.DataType
    switch ($normalizedDataType) {
        "SINGLE_SELECT" {
            $option = Resolve-ProjectSingleSelectOption -Options $Field.Options -Candidates @($Value)
            if ($null -eq $option) {
                Write-JulesWarn "Project-Feld '$($Field.Name)' enthaelt keine passende Option fuer '$Value'."
                return
            }

            $mutation = @'
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
  updateProjectV2ItemFieldValue(
    input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: { singleSelectOptionId: $optionId }
    }
  ) {
    projectV2Item {
      id
    }
  }
}
'@

            Invoke-GitHubGraphQl -Query $mutation -Variables @{
                projectId = $Context.ProjectId
                itemId    = $ItemId
                fieldId   = $Field.Id
                optionId  = [string]$option.id
            } | Out-Null
            return
        }
        "DATE" {
            $dateValue = $null
            try {
                $dateValue = ([datetimeoffset]$Value).ToUniversalTime().ToString("yyyy-MM-dd")
            } catch {
                $dateValue = $null
            }

            if ([string]::IsNullOrWhiteSpace($dateValue)) {
                Write-JulesWarn "Project-Feld '$($Field.Name)' erwartet ein Datum, '$Value' ist aber nicht konvertierbar."
                return
            }

            $mutation = @'
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $dateValue: Date!) {
  updateProjectV2ItemFieldValue(
    input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: { date: $dateValue }
    }
  ) {
    projectV2Item {
      id
    }
  }
}
'@

            Invoke-GitHubGraphQl -Query $mutation -Variables @{
                projectId = $Context.ProjectId
                itemId    = $ItemId
                fieldId   = $Field.Id
                dateValue = $dateValue
            } | Out-Null
            return
        }
        default {
            $mutation = @'
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $textValue: String!) {
  updateProjectV2ItemFieldValue(
    input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: { text: $textValue }
    }
  ) {
    projectV2Item {
      id
    }
  }
}
'@

            Invoke-GitHubGraphQl -Query $mutation -Variables @{
                projectId = $Context.ProjectId
                itemId    = $ItemId
                fieldId   = $Field.Id
                textValue = $Value
            } | Out-Null
            return
        }
    }
}

function Sync-VorceProjectFields {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [Parameter(Mandatory)][hashtable]$Fields)

    $context = Get-VorceProjectContext -Repository $Repository
    if ($null -eq $context) {
        return
    }

    $issueContentId = Get-GitHubIssueContentId -Repository $Repository -IssueNumber $IssueNumber
    if ([string]::IsNullOrWhiteSpace($issueContentId)) {
        return
    }

    $itemId = Ensure-VorceProjectItem -Context $context -IssueContentId $issueContentId
    $statusField = Get-VorceProjectField -Context $context -FieldName $context.StatusFieldName
    if ($null -ne $statusField) {
        $statusOption = Resolve-ProjectSingleSelectOption -Options $statusField.Options -Candidates (Get-VorceProjectStatusCandidateNames -Fields $Fields)
        if ($null -ne $statusOption) {
            Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field $statusField -Value ([string]$statusOption.name)
        }
    }

    $julesSessionStatus = if ($Fields.ContainsKey("JulesSessionStatus") -and -not [string]::IsNullOrWhiteSpace([string]$Fields["JulesSessionStatus"])) {
        [string]$Fields["JulesSessionStatus"]
    } elseif ($Fields.ContainsKey("SessionState") -and -not [string]::IsNullOrWhiteSpace([string]$Fields["SessionState"])) {
        Get-VorceJulesSessionStatusValue -Session ([pscustomobject]@{ state = [string]$Fields["SessionState"] }) -Issue $null
    } else {
        switch ((([string]$Fields["RemoteState"]).Trim()).ToLowerInvariant()) {
            "queued" { "queued" }
            "planning" { "planning" }
            "in-progress" { "running" }
            "in_progress" { "running" }
            "awaiting-plan-approval" { "waiting" }
            "awaiting-user-feedback" { "waiting" }
            "paused" { "waiting" }
            "failed" { "failed" }
            "completed" { "completed" }
            "pr_open" { "completed" }
            "pr-open" { "completed" }
            "pr_checks_pending" { "completed" }
            "pr_failed" { "completed" }
            "pr-failed" { "completed" }
            "pr_draft" { "completed" }
            "pr_closed" { "completed" }
            "pr-closed" { "completed" }
            "merged" { "completed" }
            default { "n_a" }
        }
    }

    $prChecksStatus = if ($Fields.ContainsKey("PrChecksStatus") -and -not [string]::IsNullOrWhiteSpace([string]$Fields["PrChecksStatus"])) {
        [string]$Fields["PrChecksStatus"]
    } elseif (-not [string]::IsNullOrWhiteSpace([string]$Fields["PullRequestUrl"])) {
        $syncPr = Get-GitHubPullRequest -Repository $Repository -PullRequestUrl ([string]$Fields["PullRequestUrl"])
        $syncChecks = if ($null -eq $syncPr) { @() } else { Get-GitHubPullRequestChecks -Repository $Repository -PullRequestUrl ([string]$Fields["PullRequestUrl"]) }
        Get-VorcePrChecksStatusValue -PullRequest $syncPr -Checks $syncChecks
    } else {
        "n_a"
    }

    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.QueueFieldName) -Value $Fields.QueueState
    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.JulesSessionStatusFieldName) -Value $julesSessionStatus
    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.PrChecksStatusFieldName) -Value $prChecksStatus
    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.WorkBranchFieldName) -Value $Fields.WorkBranch
    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.LastUpdateFieldName) -Value $Fields.LastUpdate
    Set-VorceProjectFieldValue -Context $context -ItemId $itemId -Field (Get-VorceProjectField -Context $context -FieldName $context.LinkedPrFieldName) -Value $Fields.PullRequestUrl
}

function Format-MarkdownValue {
    param([AllowNull()][object]$Value)

    if ($null -eq $Value) { return "_n/a_" }

    $text = [string]$Value
    if ([string]::IsNullOrWhiteSpace($text)) { return "_n/a_" }
    if ($text -match "^https?://") { return $text }

    return ('`{0}`' -f $text)
}

function Format-JulesIssueTrackingBlock {
    param([Parameter(Mandatory)][hashtable]$Fields)

    $lines = @(
        $script:JulesIssueBlockStart,
        "<!-- jules-session-id: $($Fields.SessionId) -->",
        "<!-- jules-session-name: $($Fields.SessionName) -->",
        "<!-- vorce-queue-state: $($Fields.QueueState) -->",
        "<!-- vorce-remote-state: $($Fields.RemoteState) -->",
        "<!-- vorce-work-branch: $($Fields.WorkBranch) -->",
        "<!-- vorce-last-update: $($Fields.LastUpdate) -->",
        "## Vorce Project Manager",
        "- Queue State: $(Format-MarkdownValue -Value $Fields.QueueState)",
        "- Remote State: $(Format-MarkdownValue -Value $Fields.RemoteState)",
        "- Work Branch: $(Format-MarkdownValue -Value $Fields.WorkBranch)",
        "- Linked PR: $(Format-MarkdownValue -Value $Fields.PullRequestUrl)",
        "- Last Update: $(Format-MarkdownValue -Value $Fields.LastUpdate)",
        $script:JulesIssueBlockEnd
    )

    return ($lines -join "`n")
}

function Upsert-JulesIssueTrackingBlock {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber, [Parameter(Mandatory)][hashtable]$Fields)

    $issue = Get-GitHubIssue -Repository $Repository -IssueNumber $IssueNumber
    $body = if ($null -eq $issue.body) { "" } else { [string]$issue.body }
    $block = Format-JulesIssueTrackingBlock -Fields $Fields
    $pattern = [regex]::Escape($script:JulesIssueBlockStart) + ".*?" + [regex]::Escape($script:JulesIssueBlockEnd)

    $cleanBody = [regex]::Replace(
        $body,
        "(?:\s*)$pattern(?:\s*)",
        "",
        [System.Text.RegularExpressions.RegexOptions]::Singleline
    ).Trim()

    if ([string]::IsNullOrWhiteSpace($cleanBody)) {
        $updatedBody = $block
    } else {
        $updatedBody = "{0}`n`n{1}" -f $cleanBody, $block
    }

    Set-GitHubIssueBody -Repository $Repository -IssueNumber $IssueNumber -Body $updatedBody
}

function Get-JulesSessionReferenceFromIssue {
    param([Parameter(Mandatory)][string]$Repository, [Parameter(Mandatory)][int]$IssueNumber)

    $issue = Get-GitHubIssue -Repository $Repository -IssueNumber $IssueNumber
    $body = if ($null -eq $issue.body) { "" } else { [string]$issue.body }

    if ($body -match "<!-- jules-session-name: (?<name>[^>]+?) -->") {
        $name = $Matches["name"].Trim()
        if (-not [string]::IsNullOrWhiteSpace($name)) {
            return @{
                SessionName = $name
                SessionId   = Resolve-JulesSessionId -SessionIdOrName $name
            }
        }
    }

    if ($body -match "<!-- jules-session-id: (?<id>[^>]+?) -->") {
        $id = $Matches["id"].Trim()
        if (-not [string]::IsNullOrWhiteSpace($id)) {
            return @{
                SessionName = Resolve-JulesSessionName -SessionIdOrName $id
                SessionId   = $id
            }
        }
    }

    foreach ($comment in (Get-GitHubIssueComments -Repository $Repository -IssueNumber $IssueNumber | Sort-Object created_at -Descending)) {
        $commentBody = [string]$comment.body
        if ($commentBody -match 'sessions/(?<id>[^)\s`]+)') {
            $id = $Matches["id"].Trim()
            return @{
                SessionName = "sessions/$id"
                SessionId   = $id
            }
        }
    }

    return $null
}

function Get-JulesPreferredPrTitle {
    param([Parameter(Mandatory)][string]$IssueTitle)

    return ("PR{0}" -f $IssueTitle.Trim())
}

function Get-JulesPreferredWorkBranch {
    param([Parameter(Mandatory)][string]$IssueTitle)

    return ("B-Jules/{0}" -f $IssueTitle.Trim())
}

function Convert-IssueToJulesPrompt {
    param(
        [Parameter(Mandatory)][object]$Issue,
        [string]$Repository,
        [string]$AdditionalPrompt,
        [bool]$AutoCreatePr
    )

    $labels = @()
    if ($Issue.labels) {
        $labels = @($Issue.labels | ForEach-Object { $_.name } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    }

    $body = if ($null -eq $Issue.body) { "" } else { [string]$Issue.body }
    $parts = @(
        "Issue #$($Issue.number): $($Issue.title)",
        "Repository: $Repository",
        "Issue URL: $($Issue.url)",
        "Required PR Title: $(Get-JulesPreferredPrTitle -IssueTitle ([string]$Issue.title))",
        "Required Work Branch: $(Get-JulesPreferredWorkBranch -IssueTitle ([string]$Issue.title))"
    )

    if ($labels.Count -gt 0) {
        $parts += "Labels: $($labels -join ', ')"
    }

    $parts += ""
    $parts += $body

    if (-not [string]::IsNullOrWhiteSpace($AdditionalPrompt)) {
        $parts += ""
        $parts += "---"
        $parts += $AdditionalPrompt.Trim()
    }

    if ($AutoCreatePr) {
        $parts += ""
        $parts += "---"
        $parts += "**IMPORTANT:** Erstelle den Pull Request exakt mit diesem Titel: `$(Get-JulesPreferredPrTitle -IssueTitle ([string]$Issue.title))`."
        $parts += "**IMPORTANT:** Arbeite exakt auf diesem Branch-Namen: `$(Get-JulesPreferredWorkBranch -IssueTitle ([string]$Issue.title))`."
        $parts += "Verwende keine abweichenden, gekuerzten oder automatisch generierten Namen fuer Branch oder Pull Request."
        $parts += ""
        $parts += "**IMPORTANT:** Wenn du die Pull-Request-Beschreibung fuer dieses Issue erstellst, musst du exakt diesen Block mit der echten GitHub-Issue-Nummer aufnehmen:"
        $parts += "## Verlinktes Issue"
        $parts += "Fixes #$($Issue.number)"
        $parts += ""
        $parts += "Ersetze die GitHub-Issue-Nummer nicht durch eine ROADMAP- oder MF-Task-ID."
    }

    return (($parts -join "`n").Trim())
}

function Sync-VorceIssueTracking {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][int]$IssueNumber,
        [AllowNull()][object]$Session,
        [AllowNull()][object]$LatestActivity,
        [string]$StartingBranch,
        [string]$SourceName
    )

    $issue = Get-GitHubIssue -Repository $Repository -IssueNumber $IssueNumber
    $pullRequestUrl = if ($null -eq $Session) { $null } else { Get-JulesSessionPullRequestUrl -Session $Session }
    $allowFallbackPullRequestLookup = $null -eq $Session
    if ([string]::IsNullOrWhiteSpace($pullRequestUrl) -and $allowFallbackPullRequestLookup) {
        $fallbackSessionId = if ($null -eq $Session) { $null } else { Resolve-JulesSessionId -SessionIdOrName ([string]$Session.name) }
        $fallbackPullRequest = Find-GitHubPullRequestForIssue -Repository $Repository -IssueNumber $IssueNumber -SessionId $fallbackSessionId
        if ($null -ne $fallbackPullRequest -and -not [string]::IsNullOrWhiteSpace([string]$fallbackPullRequest.url)) {
            $pullRequestUrl = [string]$fallbackPullRequest.url
        }
    }
    $pullRequest = if ([string]::IsNullOrWhiteSpace($pullRequestUrl)) {
        $null
    } else {
        Get-GitHubPullRequest -Repository $Repository -PullRequestUrl $pullRequestUrl
    }
    $pullRequestChecks = if ($null -eq $pullRequest) {
        @()
    } else {
        Get-GitHubPullRequestChecks -Repository $Repository -PullRequestUrl ([string]$pullRequest.url)
    }

    $fields = @{
        SessionId           = if ($null -eq $Session) { $null } else { Resolve-JulesSessionId -SessionIdOrName ([string]$Session.name) }
        SessionName         = if ($null -eq $Session) { $null } else { [string]$Session.name }
        SessionUrl          = if ($null -eq $Session) { $null } else { [string]$Session.url }
        SessionState        = if ($null -eq $Session) { "not-started" } else { [string]$Session.state }
        JulesSessionStatus  = Get-VorceJulesSessionStatusValue -Session $Session -Issue $issue
        PrChecksStatus      = Get-VorcePrChecksStatusValue -PullRequest $pullRequest -Checks $pullRequestChecks
        QueueState          = Get-VorceQueueState -Issue $issue -Session $Session
        RemoteState         = Get-VorceRemoteState -Issue $issue -Session $Session -PullRequest $pullRequest
        WorkBranch          = Get-VorceWorkBranch -PullRequest $pullRequest -StartingBranch $StartingBranch
        SourceName          = if (-not [string]::IsNullOrWhiteSpace($SourceName)) { $SourceName } elseif ($null -ne $Session) { [string]$Session.sourceContext.source } else { $null }
        PullRequestUrl      = if ($null -ne $pullRequest) { [string]$pullRequest.url } else { $pullRequestUrl }
        NeedsAttention      = Get-VorceNeedsAttention -Issue $issue -Session $Session -PullRequest $pullRequest
        LastActivitySummary = Get-VorceLastActivitySummary -Issue $issue -Session $Session -LatestActivity $LatestActivity
        IssueState          = if ($null -eq $issue) { $null } else { [string]$issue.state }
        LastUpdate          = Resolve-LatestTrackingTimestamp -Candidates @(
            if ($null -ne $LatestActivity) { [string]$LatestActivity.createTime }
            if ($null -ne $pullRequest) { [string]$pullRequest.updatedAt }
            if ($null -ne $Session) { [string]$Session.updateTime }
            if ($null -ne $Session) { [string]$Session.createTime }
            [string]$issue.updatedAt
        )
    }

    $fields["LastActivitySummary"] = Normalize-TrackingText -Value $fields["LastActivitySummary"] -MaxLength 180
    $fields["WorkBranch"] = Normalize-TrackingText -Value $fields["WorkBranch"] -MaxLength 120
    $fields["SourceName"] = Normalize-TrackingText -Value $fields["SourceName"] -MaxLength 140
    $fields["QueueState"] = Normalize-TrackingText -Value $fields["QueueState"] -MaxLength 60
    $fields["RemoteState"] = Normalize-TrackingText -Value $fields["RemoteState"] -MaxLength 60
    $fields["NeedsAttention"] = Normalize-TrackingText -Value $fields["NeedsAttention"] -MaxLength 10
    $fields["LastUpdate"] = Format-TrackingTimestamp -Timestamp $fields["LastUpdate"]

    Upsert-JulesIssueTrackingBlock -Repository $Repository -IssueNumber $IssueNumber -Fields $fields
    Sync-GitHubIssueStatusLabels -Repository $Repository -IssueNumber $IssueNumber -Issue $issue -DesiredLabels (Get-DesiredIssueStatusLabels -Issue $issue -Session $Session -PullRequest $pullRequest)
    try {
        Sync-VorceProjectFields -Repository $Repository -IssueNumber $IssueNumber -Fields $fields
    } catch {
        Write-JulesWarn "Project-Feld-Sync fuer Issue #$IssueNumber fehlgeschlagen: $($_.Exception.Message)"
    }

    return [pscustomobject]@{
        SessionId          = $fields["SessionId"]
        SessionName        = $fields["SessionName"]
        IssueNumber         = $IssueNumber
        QueueState          = $fields["QueueState"]
        RemoteState         = $fields["RemoteState"]
        JulesSessionStatus  = $fields["JulesSessionStatus"]
        PrChecksStatus      = $fields["PrChecksStatus"]
        WorkBranch          = $fields["WorkBranch"]
        PullRequestUrl      = $fields["PullRequestUrl"]
        LastUpdate          = $fields["LastUpdate"]
        NeedsAttention      = $fields["NeedsAttention"]
        LastActivitySummary = $fields["LastActivitySummary"]
        SessionState        = $fields["SessionState"]
    }
}

function Sync-JulesIssueTracking {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][int]$IssueNumber,
        [Parameter(Mandatory)][object]$Session,
        [AllowNull()][object]$LatestActivity,
        [string]$StartingBranch,
        [string]$SourceName
    )

    Sync-VorceIssueTracking -Repository $Repository -IssueNumber $IssueNumber -Session $Session -LatestActivity $LatestActivity -StartingBranch $StartingBranch -SourceName $SourceName
}
