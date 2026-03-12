
$ErrorActionPreference = "Stop"

$repoRoot = Join-Path "$PSScriptRoot/../.." "."
$vcpkgDir = Join-Path $repoRoot "vcpkg"
$vcpkgExe = Join-Path $vcpkgDir "vcpkg.exe"

Write-Host "Checking for vcpkg in $vcpkgDir..."

if (-not (Test-Path $vcpkgDir)) {
    Write-Host "Cloning vcpkg..."
    git clone https://github.com/microsoft/vcpkg.git $vcpkgDir
}

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Bootstrapping vcpkg..."
    Push-Location $vcpkgDir
    .\bootstrap-vcpkg.bat
    Pop-Location
}

Write-Host "Installing FFmpeg dependencies via vcpkg (this may take a while)..."
# Install specific features to save time if possible, but full ffmpeg is safer for compatibility
& $vcpkgExe install ffmpeg:x64-windows

Write-Host "integrating vcpkg user-wide..."
& $vcpkgExe integrate install

Write-Host "Done! Please restart your terminal/VSCode to ensure environment variables are picked up."
