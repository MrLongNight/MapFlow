# MapFlow Architecture Generation Script
# Generiert eine Übersicht der Modul-Struktur und Abhängigkeiten

$OutputDir = "docs/A1_SYSTEM"
if (!(Test-Path $OutputDir)) { New-Item -ItemType Directory -Path $OutputDir }
$OutputFile = "$OutputDir/DOC-B5_MODULE_TREE.md"

Write-Host "Generating MapFlow Architecture Info..." -ForegroundColor Cyan

$Content = "# MapFlow Module Tree (Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm'))`n`n"
$Content += "Diese Datei zeigt die physische und logische Struktur des Projekts.`n`n"

# 1. Physische Struktur (File Tree)
$Content += "## 1. Physische Crate-Struktur`n`n"
$Content += "````text`n"
$Tree = tree crates /f /a | Out-String
$Content += $Tree
$Content += "`````n`n"

# 2. Crate Abhängigkeiten aus Cargo.toml
$Content += "## 2. Workspace Crates`n`n"
$Content += "| Crate | Pfad | Beschreibung |`n"
$Content += "|-------|------|--------------|`n"

$Crates = Get-ChildItem -Path "crates" -Directory
foreach ($Crate in $Crates) {
    $ManifestPath = Join-Path $Crate.FullName "Cargo.toml"
    if (Test-Path $ManifestPath) {
        $Name = $Crate.Name
        $Path = "crates/$Name"
        $Content += "| $Name | $Path | $( (Get-Content $ManifestPath | Select-String "description =" | ForEach-Object { $_.ToString().Split('"')[1] } ) ) |`n"
    }
}

# 3. Logische Modul-Struktur (Versuche cargo-modules)
$Content += "`n## 3. Logische Modul-Hierarchie`n`n"
$CargoModulesCheck = Get-Command cargo-modules -ErrorAction SilentlyContinue

if ($CargoModulesCheck) {
    Write-Host "cargo-modules found, generating logical tree..." -ForegroundColor Green
    foreach ($Crate in $Crates) {
        $Content += "### Crate: $($Crate.Name)`n"
        $Content += "````text`n"
        # We run it for each crate to get detail
        Set-Location $Crate.FullName
        $LogicalTree = cargo modules generate tree | Out-String
        Set-Location ../..
        $Content += $LogicalTree
        $Content += "`````n"
    }
} else {
    $Content += "> Hinweis: Installiere `cargo-modules` (`cargo install cargo-modules`), um hier einen detaillierten logischen Modul-Graph zu sehen.`n"
    $Content += "> Aktuell wird nur die Datei-Struktur oben angezeigt.`n"
}

$Content | Out-File -FilePath $OutputFile -Encoding utf8
Write-Host "Done! Architecture info saved to $OutputFile" -ForegroundColor Green
