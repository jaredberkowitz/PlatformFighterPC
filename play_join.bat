@echo off
rem Join an online match hosted by someone else.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1"
if errorlevel 1 (
  echo.
  echo Could not start. See the message above.
  pause
  exit /b 1
)
set /p HOST=Host address (ip:port, for example 192.168.1.20:47000):
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot -- --join=%HOST%
