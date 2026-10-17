@echo off
rem Double-click to open the editors. Checks the tools, builds the simulation, then opens the editor scene in Godot.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1"
if errorlevel 1 (
  echo.
  echo Could not start. See the message above.
  pause
  exit /b 1
)
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot res://editor.tscn
