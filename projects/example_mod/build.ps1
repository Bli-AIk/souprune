<#
.SYNOPSIS
    Compiles the example mod for Windows.

.DESCRIPTION
    This script builds the mod_example DLLs.
    It primarily targets the MSVC toolchain (mod_example_msvc.dll).
    It can optionally build the GNU toolchain version (mod_example_gnu.dll) if MinGW is installed.

.PARAMETER Release
    Builds in release mode (optimized).

.PARAMETER All
    Attempts to build both MSVC and GNU versions.

.EXAMPLE
    .\build.ps1
    Builds debug MSVC version.

.EXAMPLE
    .\build.ps1 -Release
    Builds release MSVC version.
#>

param (
    [Switch]$Release,
    [Switch]$All
)

$ErrorActionPreference = "Stop"

# Configuration
$ScriptDir = $PSScriptRoot
$ModSourceDir = Join-Path $ScriptDir "code\mod_example"
$DestinationDir = $ScriptDir
$BuildMode = if ($Release) { "release" } else { "debug" }
$CargoFlags = if ($Release) { "--release" } else { "" }

# Helper: Check if a command exists
function Test-Command {
    param ($Command)
    return (Get-Command $Command -ErrorAction SilentlyContinue) -ne $null
}

# Helper: Check if a rust target is installed
function Test-RustTarget {
    param ($Target)
    $Installed = rustup target list --installed
    return $Installed -contains $Target
}

# Build Function
function Build-Target {
    param (
        [string]$Target,
        [string]$CargoOutputName,
        [string]$FinalOutputName,
        [string]$EnvVarName,
        [string]$EnvVarValue
    )

    $TargetDir = "target/$BuildMode"
    if (-not [string]::IsNullOrEmpty($Target)) {
        $TargetDir = "target/$Target/$BuildMode"
        Write-Host "--- Building mod_example ($BuildMode) Target: $Target ---" -ForegroundColor Cyan
        
        # Check Rust Target
        if (-not (Test-RustTarget $Target)) {
            Write-Warning "Rust target '$Target' is not installed."
            Write-Host "Trying to install '$Target'..." -ForegroundColor Yellow
            rustup target add $Target
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Failed to install target '$Target'."
            }
        }
    } else {
        Write-Host "--- Building mod_example ($BuildMode) Native ---" -ForegroundColor Cyan
    }

    Push-Location $ModSourceDir

    # Set Environment Variable for this build if needed
    if (-not [string]::IsNullOrEmpty($EnvVarName)) {
        $OldEnvValue = (Get-Item env:$EnvVarName -ErrorAction SilentlyContinue).Value
        Set-Item env:$EnvVarName $EnvVarValue
    }

    # Build Command
    $BuildArgs = @("build")
    if ($Release) { $BuildArgs += "--release" }
    if (-not [string]::IsNullOrEmpty($Target)) { $BuildArgs += "--target", $Target }

    Write-Host "Running: cargo $BuildArgs" -ForegroundColor DarkGray
    cargo $BuildArgs
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Error "Build failed for target '$Target'."
    }

    # Restore Env Var
    if (-not [string]::IsNullOrEmpty($EnvVarName)) {
        if ($OldEnvValue -eq $null) {
            Remove-Item env:$EnvVarName
        } else {
            Set-Item env:$EnvVarName $OldEnvValue
        }
    }

    Pop-Location

    # Find and Copy Artifact
    $SourcePath = Join-Path $ModSourceDir "$TargetDir/$CargoOutputName"
    if (-not (Test-Path $SourcePath)) {
        # Check deps folder as fallback
        $SourcePath = Join-Path $ModSourceDir "$TargetDir/deps/$CargoOutputName"
    }

    if (Test-Path $SourcePath) {
        $DestPath = Join-Path $DestinationDir $FinalOutputName
        Write-Host "Syncing $CargoOutputName to $FinalOutputName ..." -ForegroundColor Green
        Copy-Item -Force $SourcePath $DestPath
    } else {
        Write-Error "Build artifact not found at: $SourcePath"
    }
}

# --- Main Checks ---

if (-not (Test-Path $ModSourceDir)) {
    Write-Error "Source directory not found: $ModSourceDir"
}

if (-not (Test-Command "cargo")) {
    Write-Error "Cargo (Rust) is not installed or not in PATH. Please install Rust: https://rustup.rs/"
}

# --- Execution ---

# 1. Build MSVC (Default for Windows)
Write-Host "=== Building MSVC Version ===" -ForegroundColor Magenta
# Default Windows target implies MSVC usually, but let's be explicit if we want mod_example_msvc.dll
# Note: x86_64-pc-windows-msvc is usually the default, but we'll specify it to be safe and consistent with file naming.
Build-Target "x86_64-pc-windows-msvc" "mod_example.dll" "example_mod_msvc.dll"

# 2. Build GNU (Optional)
if ($All) {
    Write-Host "`n=== Building GNU Version ===" -ForegroundColor Magenta
    if (Test-RustTarget "x86_64-pc-windows-gnu") {
        # Check for MinGW/GCC roughly
        if ((Test-Command "gcc") -or (Test-Command "x86_64-w64-mingw32-gcc")) {
             # Use static-crt to avoid dependency hell
             Build-Target "x86_64-pc-windows-gnu" "mod_example.dll" "example_mod_gnu.dll" "RUSTFLAGS" "-C target-feature=+crt-static"
        } else {
            Write-Warning "GNU target installed but GCC/MinGW linker not found in PATH. Skipping GNU build."
            Write-Host "To fix: Install MinGW-w64."
        }
    } else {
        Write-Warning "Target 'x86_64-pc-windows-gnu' not installed. Skipping GNU build."
        Write-Host "To enable: 'rustup target add x86_64-pc-windows-gnu' and install MinGW."
    }
}

Write-Host "`nBuild Complete!" -ForegroundColor Green
