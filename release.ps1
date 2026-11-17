# Starts a release: sets the version everywhere, commits, tags and pushes.
# GitHub Actions then builds and signs the installer and creates a DRAFT
# release with the updater file (latest.json). Test it, then publish the
# draft on GitHub: only then do installed copies see the update.
#
# Usage:  powershell -ExecutionPolicy Bypass -File .\release.ps1 0.2.0
param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$Version = $Version.TrimStart("v")
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version must look like 1.2.3" }

if (git status --porcelain) { throw "Commit or discard your changes first (git status)." }
git fetch --tags --quiet
if (git tag --list "v$Version") { throw "Tag v$Version already exists." }

function Set-Version($path, $pattern, $replacement) {
    $text = [IO.File]::ReadAllText((Resolve-Path $path))
    $new = [regex]::Replace($text, $pattern, $replacement, 'Multiline')
    if ($new -eq $text) { throw "Version not found in $path" }
    [IO.File]::WriteAllText((Resolve-Path $path), $new)
}

$old = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
if ([version]$Version -le [version]$old) { throw "New version $Version must be higher than $old." }

Set-Version "src-tauri\tauri.conf.json" '("version":\s*")[^"]+(")' "`${1}$Version`${2}"
Set-Version "src-tauri\Cargo.toml" '^(version\s*=\s*")[^"]+(")' "`${1}$Version`${2}"
Set-Version "package.json" '^(\s*"version":\s*")[^"]+(")' "`${1}$Version`${2}"
# Update the version of the app itself in Cargo.lock.
cargo metadata --manifest-path src-tauri\Cargo.toml --format-version 1 --offline | Out-Null

git add -A
git commit -m "Release v$Version"
git tag "v$Version"
git push origin HEAD --tags

Write-Host "`nv$Version pushed. GitHub is now building the release:" -ForegroundColor Green
Write-Host "  https://github.com/rorugi/sanscribe/actions"
Write-Host "When it's done, test the installer and press 'Publish release' here:"
Write-Host "  https://github.com/rorugi/sanscribe/releases"
