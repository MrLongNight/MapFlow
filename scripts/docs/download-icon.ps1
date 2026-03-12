param(
    [string]$IconHash,
    [string]$OutputPath,
    [int]$Size = 48
)

$apiKey = $env:ICONS_API_KEY
if (-not $apiKey) {
    Write-Error "ICONS_API_KEY environment variable not set"
    exit 1
}

$body = @"
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"download_svg","arguments":{"iconHash":"$IconHash","size":$Size}},"id":1}
"@

try {
    $response = Invoke-WebRequest -Uri "https://public-api.streamlinehq.com/mcp" `
        -Method POST `
        -Headers @{
            "X-API-Key" = $apiKey
            "Content-Type" = "application/json"
            "Accept" = "application/json, text/event-stream"
        } `
        -Body $body `
        -UseBasicParsing

    $lines = $response.Content -split "`n"
    foreach ($line in $lines) {
        if ($line -match "^data:") {
            $json = $line.Substring(5).Trim()
            $obj = $json | ConvertFrom-Json
            $svg = $obj.result.content[0].text

            if ($svg -match "^<svg") {
                $svg | Out-File -FilePath $OutputPath -Encoding UTF8
                Write-Host "Downloaded: $OutputPath"
                return $true
            } else {
                Write-Warning "Not SVG data: $($svg.Substring(0, [Math]::Min(100, $svg.Length)))"
                return $false
            }
        }
    }
} catch {
    Write-Error "API request failed: $_"
    return $false
}
