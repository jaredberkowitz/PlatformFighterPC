@echo off
rem Host an online match. Tell the other player your IP address and the port below.
cd /d "%~dp0"
set PORT=47000
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1"
if errorlevel 1 (
  echo.
  echo Could not start. See the message above.
  pause
  exit /b 1
)
echo.
echo Hosting on UDP port %PORT%. Your addresses on this network:
ipconfig | findstr /i "IPv4"
echo For play over the internet, forward UDP port %PORT% on your router to this PC, or use a relay (see README).
echo.
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot -- --host=%PORT%
