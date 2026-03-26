# self-hosted-post-merge.ps1
# This script runs on the self-hosted Windows runner to validate the build and run GPU tests.

$ErrorActionPreference = "Stop"

$repoRoot = Get-Item $PSScriptRoot | Select-Object -ExpandProperty Parent | Select-Object -ExpandProperty Parent | Select-Object -ExpandProperty FullName
Write-Host "Repo Root: $repoRoot"

function Assert-Command($Name, $HelpText) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' not found. $HelpText"
    }
}

Assert-Command -Name "git" -HelpText "Install Git for Windows."
Assert-Command -Name "cargo" -HelpText "Install Rust and ensure cargo is in PATH."

# vcpkg detection
if (-not $env:VCPKG_ROOT) {
    $parentDir = Split-Path -Parent $repoRoot
    $grandParentDir = Split-Path -Parent $parentDir

    $candidates = @(
        (Join-Path $repoRoot "vcpkg"),
        (Join-Path $parentDir "vcpkg"),
        (Join-Path $grandParentDir "vcpkg"),
        "C:\vcpkg",
        "D:\vcpkg",
        "C:\src\vcpkg",
        "C:\tools\vcpkg",
        "$HOME\vcpkg"
    )
    foreach ($p in $candidates) {
        if (-not $p) { continue }
        # Check if drive exists before Join-Path to avoid DriveNotFoundException
        if ($p.Contains(":")) {
            $drive = $p.Split(":")[0] + ":"
            if (-not (Test-Path $drive)) { continue }
        }

        if (Test-Path (Join-Path $p "vcpkg.exe")) {
            $env:VCPKG_ROOT = $p
            Write-Host "Detected vcpkg at: $p"
            break
        }
    }
}

# Fallback: Clone vcpkg if still not found
if (-not $env:VCPKG_ROOT -or -not (Test-Path $env:VCPKG_ROOT)) {
    Write-Warning "vcpkg not found in candidates. Cloning to $repoRoot\vcpkg as fallback..."
    $env:VCPKG_ROOT = Join-Path $repoRoot "vcpkg"
    if (-not (Test-Path $env:VCPKG_ROOT)) {
        git clone --depth 1 https://github.com/microsoft/vcpkg.git $env:VCPKG_ROOT
    }
}

$vcpkgExe = Join-Path $env:VCPKG_ROOT "vcpkg.exe"
$bootstrapScript = Join-Path $env:VCPKG_ROOT "bootstrap-vcpkg.bat"

# Bootstrap if executable is missing
if (-not (Test-Path $vcpkgExe)) {
    if (Test-Path $bootstrapScript) {
        Write-Host "Bootstrapping vcpkg..."
        Push-Location $env:VCPKG_ROOT
        & .\bootstrap-vcpkg.bat
        Pop-Location
    } else {
        throw "vcpkg was not found and could not be bootstrapped at $($env:VCPKG_ROOT)."
    }
}

if (-not $env:VCPKG_DEFAULT_TRIPLET) { $env:VCPKG_DEFAULT_TRIPLET = "x64-windows" }

# LLVM / Clang for bindgen
if (-not $env:LIBCLANG_PATH) {
    $llvmPaths = @("C:\Program Files\LLVM\bin", "C:\Program Files (x86)\LLVM\bin")
    foreach ($p in $llvmPaths) {
        if (Test-Path $p) {
            $env:LIBCLANG_PATH = $p
            $env:Path = "$p;$env:Path"
            Write-Host "Detected LLVM at: $p"
            break
        }
    }
}

# Ensure artifacts dir exists
$artifactsDir = Join-Path $repoRoot "artifacts\visual-capture"
if (-not (Test-Path $artifactsDir)) {
    New-Item -ItemType Directory -Force -Path $artifactsDir
}

Write-Host "--- Starting Build & Test (Limited to 4 threads) ---"
cargo build --workspace --release -j 4

# Run Visual Automation if enabled
if ($env:VORCE_SELF_HOSTED_RUN_VISUAL_AUTOMATION -eq "true") {
    Write-Host "Running Visual Automation Tests..."
    cargo test -p vorce --test visual_capture_tests --release -j 4 -- --ignored --nocapture
}

# Run GPU Tests if enabled
if ($env:VORCE_SELF_HOSTED_RUN_IGNORED_GPU_TESTS -eq "true") {
    Write-Host "Running GPU-bound tests..."
    cargo test --workspace --release -j 4 -- --ignored
}

Write-Host "Validation completed successfully."
