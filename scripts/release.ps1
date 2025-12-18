# Release script for Claude Code Usage Tracker
# This script updates version numbers, commits changes, and creates a git tag

$ErrorActionPreference = "Stop"

# Get the project root directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

$TauriConf = Join-Path $ProjectRoot "src-tauri\tauri.conf.json"
$CargoToml = Join-Path $ProjectRoot "src-tauri\Cargo.toml"

# Check if files exist
if (-not (Test-Path $TauriConf)) {
    Write-Host "Error: tauri.conf.json not found at $TauriConf" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $CargoToml)) {
    Write-Host "Error: Cargo.toml not found at $CargoToml" -ForegroundColor Red
    exit 1
}

# Get current version from tauri.conf.json
$TauriContent = Get-Content $TauriConf -Raw
if ($TauriContent -match '"version":\s*"([^"]+)"') {
    $CurrentVersion = $Matches[1]
} else {
    Write-Host "Error: Could not find version in tauri.conf.json" -ForegroundColor Red
    exit 1
}

Write-Host "Current version: " -NoNewline -ForegroundColor Yellow
Write-Host $CurrentVersion -ForegroundColor Green

# Prompt for new version
$NewVersion = Read-Host "Enter new version (without 'v' prefix)"

# Validate version input
if ([string]::IsNullOrWhiteSpace($NewVersion)) {
    Write-Host "Error: Version cannot be empty" -ForegroundColor Red
    exit 1
}

# Confirm
Write-Host ""
Write-Host "Will update version from " -NoNewline -ForegroundColor Yellow
Write-Host $CurrentVersion -NoNewline -ForegroundColor Green
Write-Host " to " -NoNewline -ForegroundColor Yellow
Write-Host $NewVersion -ForegroundColor Green

$Confirm = Read-Host "Continue? (y/n)"
if ($Confirm -ne "y" -and $Confirm -ne "Y") {
    Write-Host "Cancelled" -ForegroundColor Yellow
    exit 0
}

# Update tauri.conf.json
Write-Host ""
Write-Host "Updating tauri.conf.json..." -ForegroundColor Yellow
$TauriContent = $TauriContent -replace """version"":\s*""$CurrentVersion""", """version"": ""$NewVersion"""
Set-Content -Path $TauriConf -Value $TauriContent -NoNewline
Write-Host "✓ Updated tauri.conf.json" -ForegroundColor Green

# Update Cargo.toml (only the first occurrence - package version)
Write-Host "Updating Cargo.toml..." -ForegroundColor Yellow
$CargoContent = Get-Content $CargoToml -Raw
$CargoContent = $CargoContent -replace "(?m)^version = ""$CurrentVersion""", "version = ""$NewVersion"""
Set-Content -Path $CargoToml -Value $CargoContent -NoNewline
Write-Host "✓ Updated Cargo.toml" -ForegroundColor Green

# Git operations
Write-Host ""
Write-Host "Committing changes..." -ForegroundColor Yellow
Push-Location $ProjectRoot
try {
    git add $TauriConf $CargoToml
    git commit -m "chore: bump version to $NewVersion"
    Write-Host "✓ Committed changes" -ForegroundColor Green

    # Create tag
    $TagName = "v$NewVersion"
    Write-Host "Creating tag $TagName..." -ForegroundColor Yellow
    git tag $TagName
    Write-Host "✓ Created tag $TagName" -ForegroundColor Green
} finally {
    Pop-Location
}

# Done
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Release preparation complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "To push the release, run:"
Write-Host "  git push && git push origin $TagName" -ForegroundColor Yellow
