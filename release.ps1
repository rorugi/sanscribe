# Builds a release on this PC and prepares it for GitHub.
#
# 1. sets the version (tauri.conf.json, Cargo.toml, package.json) and the README download link
# 2. builds and signs the installer locally (build.ps1)
# 3. writes latest.json for the updater
# 4. commits, tags vX.Y.Z and pushes
# 5. opens GitHub's "new release" page and the folder with the three files to upload
#
# Nothing is built on GitHub. Installed copies see the update once you press "Publish release".
#
# Usage:  powershell -ExecutionPolicy Bypass -File .\release.ps1 0.3.0
#         powershell -ExecutionPolicy Bypass -File .\release.ps1 0.3.0 -Notes "Diacritic keys"
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [string]$Notes = "See the release page for what's new."
)
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$Repo = "rorugi/sanscribe"
$Version = $Version.TrimStart("v")
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version must look like 1.2.3" }

if (git status --porcelain) { throw "Commit or discard your changes first (git status)." }
git pull --ff-only --quiet
if ($LASTEXITCODE -ne 0) { throw "git pull failed." }
git fetch --tags --quiet
if (git tag --list "v$Version") { throw "Tag v$Version already exists." }

$old = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
if ([version]$Version -le [version]$old) { throw "New version $Version must be higher than $old." }

$utf8 = New-Object System.Text.UTF8Encoding $false   # UTF-8 without BOM

function Set-Text($path, $pattern, $replacement) {
    $full = (Resolve-Path $path).Path
    $text = [IO.File]::ReadAllText($full)
    $new = [regex]::Replace($text, $pattern, $replacement, 'Multiline')
    if ($new -eq $text) { throw "Pattern not found in $path" }
    [IO.File]::WriteAllText($full, $new, $utf8)
}

$changed = "src-tauri/tauri.conf.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "package.json", "README.md"

try {
    # --- 1. version and README link ---
    Set-Text "src-tauri\tauri.conf.json" '("version":\s*")[^"]+(")' "`${1}$Version`${2}"
    Set-Text "src-tauri\Cargo.toml" '^(version\s*=\s*")[^"]+(")' "`${1}$Version`${2}"
    Set-Text "package.json" '^(\s*"version":\s*")[^"]+(")' "`${1}$Version`${2}"
    Set-Text "README.md" 'releases/download/v\d+\.\d+\.\d+/Sanscribe_\d+\.\d+\.\d+_x64-setup\.exe' "releases/download/v$Version/Sanscribe_${Version}_x64-setup.exe"
    Set-Text "README.md" 'Download Sanscribe \d+\.\d+\.\d+ for Windows' "Download Sanscribe $Version for Windows"

    # --- 2. build and sign ---
    & "$PSScriptRoot\build.ps1" -NoOpen

    $bundle = Join-Path $PSScriptRoot "src-tauri\target\release\bundle\nsis"
    $exeName = "Sanscribe_${Version}_x64-setup.exe"
    $exe = Join-Path $bundle $exeName
    $sig = "$exe.sig"
    if (-not (Test-Path $exe)) { throw "Installer not found: $exe" }
    if (-not (Test-Path $sig)) { throw "Signature not found: $sig (is the updater key in .secrets?)" }

    # --- 3. latest.json ---
    $out = Join-Path $PSScriptRoot "release\v$Version"
    New-Item -ItemType Directory -Force $out | Out-Null
    Copy-Item $exe, $sig $out -Force
    $latest = [ordered]@{
        version   = $Version
        notes     = $Notes
        pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        platforms = [ordered]@{
            "windows-x86_64" = [ordered]@{
                signature = (Get-Content $sig -Raw).Trim()
                url       = "https://github.com/$Repo/releases/download/v$Version/$exeName"
            }
        }
    }
    [IO.File]::WriteAllText((Join-Path $out "latest.json"), ($latest | ConvertTo-Json -Depth 5), $utf8)
}
catch {
    Write-Host "`nRelease aborted, undoing the version change." -ForegroundColor Red
    git checkout -- $changed
    throw
}

# --- 4. commit, tag, push ---
git add -A
git commit -m "Release v$Version"
git tag "v$Version"
git push origin HEAD
if ($LASTEXITCODE -ne 0) { throw "git push failed. Run 'git push origin HEAD' and 'git push origin v$Version' yourself." }
git push origin "v$Version"
if ($LASTEXITCODE -ne 0) { throw "Pushing the tag failed. Run 'git push origin v$Version' yourself." }

# --- 5. hand over to the browser ---
Write-Host "`nv$Version is built and pushed." -ForegroundColor Green
Write-Host "Now on GitHub (the page opens automatically):"
Write-Host "  1. drag the 3 files from the folder that opens into the release page"
Write-Host "     ($exeName, $exeName.sig, latest.json)"
Write-Host "  2. write what's new and press 'Publish release'"
$title = [uri]::EscapeDataString("Sanscribe v$Version")
Start-Process "https://github.com/$Repo/releases/new?tag=v$Version&title=$title"
Invoke-Item $out
