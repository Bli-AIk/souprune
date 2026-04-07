#!/usr/bin/env pwsh
param(
    [Parameter(Position=0)]
    [string]$ModName,
    
    [switch]$Help,
    [switch]$Clean,
    [switch]$List,
    [switch]$Update
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
$ModsToml = Join-Path $RepoRoot "mods.toml"
$ModRepo = Join-Path $RepoRoot ".mod-repo"

function Write-Info { Write-Host "[INFO] $args" -ForegroundColor Green }
function Write-Warn { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Write-Error { Write-Host "[ERROR] $args" -ForegroundColor Red }

function Show-Help {
    @"

Usage: .\setup_mods.ps1 [OPTIONS] [MOD_NAME]

Setup SoupRune example mods using git worktree.

Options:
    -Help           Show this help message
    -Clean          Remove all mod worktrees
    -List           List available mods
    -Update         Update mod repository and worktrees

Arguments:
    MOD_NAME        Specific mod to install (optional, installs all by default)

Examples:
    .\setup_mods.ps1                    # Install all mods
    .\setup_mods.ps1 example_mod        # Install only example_mod
    .\setup_mods.ps1 -Clean             # Remove all worktrees
    .\setup_mods.ps1 -Update            # Update from remote
"@
}

function Get-ModList {
    $content = Get-Content $ModsToml -Raw
    $matches = [regex]::Matches($content, '\[mods\.([^\]]+)\]')
    return $matches | ForEach-Object { $_.Groups[1].Value }
}

function Get-ModProperty {
    param($ModName, $Property)
    
    $content = Get-Content $ModsToml -Raw
    $pattern = "(?ms)\[mods\.$ModName\](.+?)(?=\[|\Z)"
    $match = [regex]::Match($content, $pattern)
    
    if ($match.Success) {
        $propMatch = [regex]::Match($match.Groups[1].Value, "$Property\s*=\s*""([^""]+)""")
        if ($propMatch.Success) {
            return $propMatch.Groups[1].Value
        }
    }
    return $null
}

function Get-RepoUrl {
    $content = Get-Content $ModsToml -Raw
    $match = [regex]::Match($content, 'url\s*=\s*"([^"]+)"')
    if ($match.Success) {
        return $match.Groups[1].Value
    }
    return $null
}

function Initialize-ModRepo {
    if (Test-Path $ModRepo) {
        Write-Info "Mod repository already exists at $ModRepo"
        return
    }

    $repoUrl = Get-RepoUrl
    Write-Info "Cloning mod repository to $ModRepo..."
    git clone --bare $repoUrl $ModRepo
    Write-Info "Mod repository initialized"
}

function Setup-Worktree {
    param($ModName)
    
    $branch = Get-ModProperty $ModName "branch"
    $path = Get-ModProperty $ModName "path"
    $fullPath = Join-Path $RepoRoot $path

    if (Test-Path $fullPath) {
        Write-Warn "Worktree already exists at $path"
        return
    }

    Write-Info "Creating worktree for $ModName (branch: $branch)..."
    Push-Location $ModRepo
    git worktree add "../$path" $branch
    Pop-Location
    Write-Info "Created worktree at $path"
}

function Remove-Worktree {
    param($ModName)
    
    $path = Get-ModProperty $ModName "path"
    $fullPath = Join-Path $RepoRoot $path

    if (-not (Test-Path $fullPath)) {
        Write-Warn "Worktree does not exist at $path"
        return
    }

    Write-Info "Removing worktree at $path..."
    Push-Location $ModRepo
    git worktree remove $fullPath --force 2>$null
    if (-not $?) {
        Remove-Item -Recurse -Force $fullPath
    }
    Pop-Location
    Write-Info "Removed worktree at $path"
}

function Update-ModRepo {
    if (-not (Test-Path $ModRepo)) {
        Write-Error "Mod repository not initialized. Run setup first."
        exit 1
    }

    Write-Info "Updating mod repository..."
    Push-Location $ModRepo
    git fetch origin
    Pop-Location
    Write-Info "Mod repository updated"
}

function Show-ModList {
    Write-Host "Available mods:"
    Write-Host ""
    foreach ($mod in Get-ModList) {
        $desc = Get-ModProperty $mod "description"
        Write-Host "  $($mod.PadRight(25)) $desc"
    }
}

function Clear-All {
    if (-not (Test-Path $ModRepo)) {
        Write-Warn "No mod repository found"
        return
    }

    Write-Info "Removing all worktrees..."
    Push-Location $ModRepo

    foreach ($mod in Get-ModList) {
        Remove-Worktree $mod
    }

    Pop-Location

    Write-Info "Removing mod repository..."
    Remove-Item -Recurse -Force $ModRepo
    Write-Info "Cleanup complete"
}

if ($Help) {
    Show-Help
    exit 0
}

if (-not (Test-Path $ModsToml)) {
    Write-Error "mods.toml not found at $ModsToml"
    exit 1
}

if ($List) {
    Show-ModList
    exit 0
}

if ($Clean) {
    Clear-All
    exit 0
}

if ($Update) {
    Update-ModRepo
    exit 0
}

Initialize-ModRepo

if ($ModName) {
    $modList = Get-ModList
    if ($modList -contains $ModName) {
        Setup-Worktree $ModName
    } else {
        Write-Error "Unknown mod: $ModName"
        Show-ModList
        exit 1
    }
} else {
    foreach ($mod in Get-ModList) {
        Setup-Worktree $mod
    }
}

Write-Info "Setup complete!"
Write-Info "Configure active mod in projects/config.toml"
