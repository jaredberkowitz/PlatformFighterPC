@echo off
rem Double-click once after downloading the code: checks for Rust, the C++ build tools and Godot (offering to install what is
rem missing), builds the simulation and imports the game's assets. play.bat runs the same checks every time, quickly.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1"
if errorlevel 1 (
  echo.
  echo Setup did not finish. See the message above.
  pause
  exit /b 1
)
echo.
echo All set. Double-click play.bat to play, or play_editor.bat for the editors.
pause
