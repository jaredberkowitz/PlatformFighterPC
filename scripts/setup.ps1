# Checks (and, with your OK, installs) everything needed to build and run the game from the source code on Windows:
#   1. Rust (rustup, stable toolchain)
#   2. Microsoft's C++ build tools (Rust's linker on Windows)
#   3. Godot 4.7.1 in tools\godot (downloaded from godotengine's GitHub releases)
#   4. The Rust simulation bridge, built
#   5. The Godot project's assets, imported (a fresh copy has none)
# Run it through setup.bat (double-click), or play.bat runs it for you. Safe to run again: it skips what is already there.
param([switch]$NoBuild)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$godotVersion = '4.7.1'
$godotDir = Join-Path $root 'tools\godot'
$godotExe = Join-Path $godotDir "Godot_v$godotVersion-stable_win64_console.exe"
$godotUrl = "https://github.com/godotengine/godot/releases/download/$godotVersion-stable/Godot_v$godotVersion-stable_win64.exe.zip"

function Say($text) { Write-Host $text }
function Ask($question) {
    $answer = Read-Host "$question [Y/n]"
    return ($answer -eq '' -or $answer -match '^[Yy]')
}
function Have-Winget { return [bool](Get-Command winget -ErrorAction SilentlyContinue) }
function Fail($text) {
    Write-Host ''
    Write-Host $text -ForegroundColor Red
    exit 1
}

# 1. Rust
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Say 'Rust is not installed (it builds the game''s simulation).'
    if (-not (Have-Winget)) { Fail 'Install Rust from https://rustup.rs (default options), then run this again.' }
    if (-not (Ask 'Install Rust now with winget?')) { Fail 'Rust is needed. Install it from https://rustup.rs, then run this again.' }
    winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail 'Rust was installed but cargo is not found yet. Close this window, open a new one and run it again.' }
    rustup default stable | Out-Null
}
Say ('Rust: ' + (cargo --version))

# 2. The C++ build tools (Rust's linker on Windows)
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$haveVc = $false
if (Test-Path $vswhere) {
    $found = & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    $haveVc = [bool]$found
}
if (-not $haveVc) {
    Say 'Microsoft''s C++ build tools are not installed (Rust needs their linker on Windows). It is a large download (a few GB).'
    if (-not (Have-Winget)) { Fail 'Install "Build Tools for Visual Studio" with the "Desktop development with C++" workload, then run this again.' }
    if (-not (Ask 'Install them now with winget?')) { Fail 'The C++ build tools are needed. Install "Build Tools for Visual Studio" (Desktop development with C++), then run this again.' }
    winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-source-agreements --accept-package-agreements --override '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
}
Say 'C++ build tools: found'

# 3. Godot
if (-not (Test-Path $godotExe)) {
    Say "Godot $godotVersion is not in tools\godot."
    if (-not (Ask "Download Godot $godotVersion (about 60 MB) from godotengine's GitHub releases?")) { Fail "Put Godot_v$godotVersion-stable_win64_console.exe in tools\godot, then run this again." }
    New-Item -ItemType Directory -Force -Path $godotDir | Out-Null
    $zip = Join-Path $env:TEMP "godot-$godotVersion.zip"
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $godotUrl -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $godotDir -Force
    Remove-Item $zip
    if (-not (Test-Path $godotExe)) { Fail "The download did not contain $godotExe." }
}
Say "Godot: $godotExe"

if ($NoBuild) { exit 0 }

# 4. The simulation bridge (debug for the editor, the game run from source uses it too)
Say 'Building the simulation (the first time takes a few minutes)...'
cargo build -p godot-bridge
if ($LASTEXITCODE -ne 0) { Fail 'The build failed. Scroll up to see why, and send the red lines to the team.' }

# 5. Import the Godot project's assets once (a fresh copy has none, and the game cannot load them otherwise). The first import
#    sometimes stops before it finishes writing everything, so it runs twice.
if (-not (Test-Path (Join-Path $root 'godot\.godot\imported'))) {
    Say 'Importing the game''s assets (first time only, a minute or two)...'
    foreach ($pass in 1..2) {
        Start-Process -FilePath $godotExe -ArgumentList '--headless', '--path', 'godot', '--import' -Wait -WindowStyle Hidden
    }
}
Say 'Ready.'
exit 0
