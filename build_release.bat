@echo off
rem Builds a Windows alpha into the build folder: the release simulation bridge, the shipped content, and (if Godot's export
rem templates are installed) the exported game. Run from the repository root.
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

echo Building the simulation bridge (release)...
cargo build --release -p godot-bridge
if errorlevel 1 goto failed

if not exist build mkdir build
if not exist build\content mkdir build\content
copy /y content\*.pfc build\content\ >nul
copy /y content\blocklist.txt build\content\ >nul
copy /y target\release\pf_bridge.dll build\ >nul

if not exist "tools\godot\Godot_v4.7.1-stable_win64_console.exe" (
  echo Godot was not found at tools\godot\. The bridge and content are in build\, but the game was not exported.
  goto done
)
echo Exporting the game (needs Godot's export templates; install them from Godot: Editor, Manage Export Templates)...
"tools\godot\Godot_v4.7.1-stable_win64_console.exe" --headless --path godot --export-release "Windows Desktop" ..\build\PlatformFighter.exe
if errorlevel 1 (
  echo.
  echo The export failed. If it says templates are missing, install them in Godot first. The bridge and content are in build\.
  goto done
)
echo Done: build\PlatformFighter.exe
goto done

:failed
echo.
echo Build failed. Scroll up to see why.
:done
pause
