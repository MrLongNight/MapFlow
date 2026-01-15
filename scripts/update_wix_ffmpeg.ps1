param (
    [string]$VcpkgBinDir,
    [string]$WixFile
)

Write-Host "Updating WiX file: $WixFile with DLLs from $VcpkgBinDir"

if (-not (Test-Path $VcpkgBinDir)) {
    Write-Error "vcpkg bin directory not found: $VcpkgBinDir"
    exit 1
}

if (-not (Test-Path $WixFile)) {
    Write-Error "WiX file not found: $WixFile"
    exit 1
}

# Define the libraries we expect (without version numbers)
$libs = @("avcodec", "avdevice", "avfilter", "avformat", "avutil", "swresample", "swscale")
$content = Get-Content -Path $WixFile -Raw

foreach ($lib in $libs) {
    # Find the actual file in the bin dir
    $pattern = "${lib}-*.dll"
    $files = Get-ChildItem -Path $VcpkgBinDir -Filter $pattern

    if ($files.Count -eq 0) {
        Write-Error "Could not find DLL for $lib in $VcpkgBinDir"
        exit 1
    }

    # Take the first match (usually there's only one active version)
    $actualFile = $files[0].Name
    Write-Host "Found $lib -> $actualFile"

    # Regex replace in the WiX file
    # We look for Name='avcodec-*.dll' and Source='...avcodec-*.dll'
    # Pattern to match: Name='avcodec-\d+\.dll'
    $regexName = "Name=['`"]${lib}-\d+\.dll['`"]"
    $replaceName = "Name='$actualFile'"

    $regexSource = "Source=['`"](.*)\\${lib}-\d+\.dll['`"]"
    # We need to keep the captured path part
    # PowerShell regex replace is a bit tricky with groups, so we iterate

    # Simple string replacement for the filename part might be safer if the structure is known
    # But let's use regex for precision

    $content = $content -replace "Name=['`"]${lib}-\d+\.dll['`"]", "Name='$actualFile'"

    # For Source, we just replace the filename at the end of the path
    $content = $content -replace "\\${lib}-\d+\.dll['`"]", "\$actualFile'"
}

Set-Content -Path $WixFile -Value $content
Write-Host "Successfully updated $WixFile"
