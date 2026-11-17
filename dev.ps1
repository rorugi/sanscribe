# Runs Sanscribe in development mode (debug build, console with log output).
# Usage:  powershell -ExecutionPolicy Bypass -File .\dev.ps1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
if (-not (Test-Path "node_modules")) { npm install }
npm run dev
