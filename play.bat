@echo off
rem Double-click to play. Builds the Rust sim bridge, then opens the game in Godot.
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

echo Building the simulation (first time takes a few minutes)...
cargo build -p godot-bridge
if errorlevel 1 (
  echo.
  echo Build failed. Scroll up to see why.
  pause
  exit /b 1
)

if not exist "tools\godot\Godot_v4.7.1-stable_win64_console.exe" (
  echo.
  echo Godot was not found at tools\godot\Godot_v4.7.1-stable_win64_console.exe
  pause
  exit /b 1
)

echo Starting the game...
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot
