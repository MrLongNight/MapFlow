param(
    [string]$Query,
    [string]$Style = "",
    [int]$Limit = 10
)

$apiKey = $env:ICONS_API_KEY
if (-not $apiKey) {
    Write-Error "ICONS_API_KEY environment variable not set"
    exit 1
}

# Build request body
$styleParam = ""
if ($Style) {
    $styleParam = ",`"style`":`"$Style`""
}

$body = @"
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"search","arguments":{"productType":"icons","query":"$Query","limit":$Limit,"productTier":"free"$styleParam}},"id":1}
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
            $results = ($obj.result.content[0].text | ConvertFrom-Json).results
            return $results
        }
    }
} catch {
    Write-Error "API request failed: $_"
}
