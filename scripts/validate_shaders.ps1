# scripts/validate_shaders.ps1
# Dieses Skript validiert alle .wgsl Shader-Dateien im Projekt mit naga.

$shaderDir = "shaders"
$nagaExe = "naga"

if (-not (Get-Command $nagaExe -ErrorAction SilentlyContinue)) {
    Write-Warning "naga-cli ist nicht installiert. Shader-Validierung wird übersprungen."
    exit 0
}

$shaders = Get-ChildItem -Path $shaderDir -Filter *.wgsl -Recurse

$allPassed = $true

foreach ($shader in $shaders) {
    Write-Host "Validierung: $($shader.FullName)..." -ForegroundColor Cyan
    & $nagaExe $shader.FullName | Out-Null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Shader-Fehler in: $($shader.FullName)"
        $allPassed = $false
    }
}

if ($allPassed) {
    Write-Host "Alle Shader erfolgreich validiert!" -ForegroundColor Green
} else {
    Write-Error "Einige Shader-Validierungen sind fehlgeschlagen."
    exit 1
}
