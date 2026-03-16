[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [string]$HelpText
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name not found. $HelpText"
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $repoRoot

Write-Host "MapFlow self-hosted post-merge validation"
Write-Host "PR number: $env:MAPFLOW_PR_NUMBER"
Write-Host "PR head SHA: $env:MAPFLOW_PR_HEAD_SHA"
Write-Host "Merge commit SHA: $env:MAPFLOW_MERGE_COMMIT_SHA"

Assert-Command -Name "git" -HelpText "Install Git on the self-hosted Windows runner."
Assert-Command -Name "cargo" -HelpText "Install Rust with rustup on the self-hosted Windows runner."
Assert-Command -Name "rustup" -HelpText "Install rustup on the self-hosted Windows runner."

if (-not $env:VCPKG_ROOT) {
    $env:VCPKG_ROOT = Join-Path $repoRoot "vcpkg"
}
if (-not $env:VCPKG_DEFAULT_TRIPLET) {
    $env:VCPKG_DEFAULT_TRIPLET = "x64-windows"
}
if (-not $env:VCPKG_INSTALLED_DIR) {
    $env:VCPKG_INSTALLED_DIR = Join-Path $repoRoot "vcpkg_installed"
}

$vcpkgExe = Join-Path $env:VCPKG_ROOT "vcpkg.exe"
$bootstrapScript = Join-Path $env:VCPKG_ROOT "bootstrap-vcpkg.bat"
if (-not (Test-Path $vcpkgExe)) {
    if (-not (Test-Path $bootstrapScript)) {
        throw "vcpkg was not found at '$($env:VCPKG_ROOT)'. See docs/A3_PROJECT/B4_CICD/DOC-C5_SELF_HOSTED_RUNNER_WINDOWS.md."
    }

    Write-Host "Bootstrapping vcpkg..."
    & $bootstrapScript
}

$llvmBin = $env:LIBCLANG_PATH
if (-not $llvmBin) {
    $defaultLlvmBin = "C:\Program Files\LLVM\bin"
    if (Test-Path $defaultLlvmBin) {
        $llvmBin = $defaultLlvmBin
    }
}

if (-not $llvmBin -or -not (Test-Path $llvmBin)) {
    throw "LLVM/Clang not found. Set LIBCLANG_PATH or install LLVM to 'C:\Program Files\LLVM\bin'."
}

$env:LIBCLANG_PATH = $llvmBin
if (-not $env:CLANG_PATH) {
    $env:CLANG_PATH = Join-Path $llvmBin "clang.exe"
}

if (-not (Test-Path $env:CLANG_PATH)) {
    throw "clang.exe not found at '$($env:CLANG_PATH)'."
}

$env:Path = "$llvmBin;$env:Path"

Write-Host "Installing manifest dependencies with vcpkg..."
& $vcpkgExe install --triplet $env:VCPKG_DEFAULT_TRIPLET --x-manifest-root $repoRoot

$installedRoot = Join-Path $env:VCPKG_INSTALLED_DIR $env:VCPKG_DEFAULT_TRIPLET
if (-not (Test-Path $installedRoot)) {
    throw "Expected vcpkg installed directory '$installedRoot' was not created."
}

$includePath = Join-Path $installedRoot "include"
$libPath = Join-Path $installedRoot "lib"
$binPath = Join-Path $installedRoot "bin"
$pkgConfigPath = Join-Path $libPath "pkgconfig"
$installedRootUnix = $installedRoot.Replace("\", "/")
$includePathUnix = $includePath.Replace("\", "/")

$env:FFMPEG_DIR = $installedRootUnix
$env:FFMPEG_INCLUDE_DIR = $includePath
$env:FFMPEG_LIB_DIR = $libPath
$env:FFMPEG_VERSION = "7.1"
$env:PKG_CONFIG_PATH = $pkgConfigPath
$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-pc-windows-msvc -I`"$includePathUnix`""
$env:Path = "$binPath;$env:Path"

Write-Host "Tool versions"
cargo --version
rustc --version
& $env:CLANG_PATH --version

Write-Host "Running Windows smoke build for the full desktop app path"
cargo build --release --verbose -p mapmap --features "audio,ffmpeg"

if ($env:MAPFLOW_SELF_HOSTED_RUN_IGNORED_GPU_TESTS -eq "true") {
    Write-Host "Running ignored GPU tests on the self-hosted runner"
    cargo test -p mapmap-render --test effect_chain_tests -- --ignored
    cargo test -p mapmap-render --test effect_chain_integration_tests -- --ignored
} else {
    Write-Host "Ignored GPU tests are disabled. Set MAPFLOW_SELF_HOSTED_RUN_IGNORED_GPU_TESTS=true later to enable them."
}

if ($env:MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION -eq "true") {
    Write-Host "Running local visual capture regression tests"
    $visualArtifactRoot = Join-Path $repoRoot "artifacts\visual-capture"
    New-Item -ItemType Directory -Force -Path $visualArtifactRoot | Out-Null
    $env:MAPFLOW_VISUAL_CAPTURE_OUTPUT_DIR = $visualArtifactRoot
    Write-Host "Visual artifacts will be written to $visualArtifactRoot"

    $metadataPath = Join-Path $visualArtifactRoot "run_metadata.json"
    $metadata = @{
        pr_number = $env:MAPFLOW_PR_NUMBER
        pr_head_sha = $env:MAPFLOW_PR_HEAD_SHA
        merge_commit_sha = $env:MAPFLOW_MERGE_COMMIT_SHA
        timestamp = (Get-Date -AsUTC).ToString("yyyy-MM-ddTHH:mm:ssZ")
    }
    $metadata | ConvertTo-Json -Depth 2 | Out-File -FilePath $metadataPath -Encoding utf8

    Write-Host "Wrote run_metadata.json hook for multimodal evaluation"

    cargo test -p mapmap --no-default-features --test visual_capture_tests -- --ignored --nocapture
} else {
    Write-Host "Visual automation is disabled. Set MAPFLOW_SELF_HOSTED_RUN_VISUAL_AUTOMATION=true to run the local screenshot regression tests."
}

if ($env:MAPFLOW_SELF_HOSTED_RUN_PERFORMANCE_CHECK -eq "true") {
    Write-Host "Running performance benchmark on the self-hosted runner"
    $perfArgs = @("scripts/dev-tools/run_performance_benchmark.py")

    if ($env:MAPFLOW_PERFORMANCE_THRESHOLD) {
        $perfArgs += "--threshold"
        $perfArgs += $env:MAPFLOW_PERFORMANCE_THRESHOLD
        $perfArgs += "--fail-on-regression"
    }

    python @perfArgs
} else {
    Write-Host "Performance benchmark is disabled. Set MAPFLOW_SELF_HOSTED_RUN_PERFORMANCE_CHECK=true to enable it."
}
