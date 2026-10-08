@echo off
rem Double-click to open the editors. Builds the Rust sim bridge, then opens the editor scene in Godot.
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

"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot res://editor.tscn
