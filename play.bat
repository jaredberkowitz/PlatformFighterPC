@echo off
rem Double-click to play from the source code. Checks the tools (offering to install what is missing), builds the simulation,
rem then opens the game in Godot. (Just want to play? Use the latest GitHub release instead: no tools needed.)
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1"
if errorlevel 1 (
  echo.
  echo Could not start. See the message above.
  pause
  exit /b 1
)
echo Starting the game...
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot
