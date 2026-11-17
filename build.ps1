# Builds the Sanscribe Windows installer (NSIS, per-user, no admin rights needed).
# Usage:  powershell -ExecutionPolicy Bypass -File .\build.ps1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

foreach ($cmd in "cargo", "npm") {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        throw "'$cmd' was not found. See DEVELOPMENT.md -> Prerequisites."
    }
}

if (-not (Test-Path "node_modules")) { npm install }

# Updater signing key (needed because the build creates signed update files).
$key = Join-Path $PSScriptRoot ".secrets\sanscribe-updater.key"
if (Test-Path $key) {
    $env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $key -Raw).Trim()
    # PowerShell can't hold an empty variable, so the key has a real password stored next to it.
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content "$key.password" -Raw).Trim()
} else {
    throw "Updater key not found at $key. Generate one with: npx tauri signer generate -w .secrets\sanscribe-updater.key (and save its password as .secrets\sanscribe-updater.key.password)"
}

Write-Host "Running engine tests..." -ForegroundColor Cyan
cargo test --manifest-path core/Cargo.toml --quiet
if ($LASTEXITCODE -ne 0) { throw "Engine tests failed." }

Write-Host "Building Sanscribe..." -ForegroundColor Cyan
npm run build
if ($LASTEXITCODE -ne 0) { throw "Build failed." }

$out = Join-Path $PSScriptRoot "src-tauri\target\release\bundle\nsis"
Write-Host "`nDone. Installer:" -ForegroundColor Green
Get-ChildItem $out -Filter *.exe | ForEach-Object { Write-Host "  $($_.FullName)" }
Invoke-Item $out
