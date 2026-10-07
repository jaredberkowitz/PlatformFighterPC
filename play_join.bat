@echo off
rem Join an online match hosted by someone else.
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
set /p HOST=Host address (ip:port, for example 192.168.1.20:47000): 
echo Building the simulation...
cargo build -p godot-bridge
if errorlevel 1 (
  pause
  exit /b 1
)
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --path godot -- --join=%HOST%
