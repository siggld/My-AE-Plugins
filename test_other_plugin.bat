@echo off
setlocal
set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins\AOD"
set "CARGO_TARGET_DIR=%LOCALAPPDATA%\AEPluginBuild"

echo [1] Build red_noise and deploy to AOD folder (for crash isolation test)
echo [2] Remove island_id_color.aex first - only red_noise.aex will be in folder
echo [3] Start AE - if AE starts, crash is specific to island_id_color binary
echo.
cd /d "%~dp0"

if exist "%AE_PLUGINS%\island_id_color.aex" (
    ren "%AE_PLUGINS%\island_id_color.aex" "island_id_color.aex.bak"
    echo Renamed island_id_color.aex to .bak
)

set "CARGO_TARGET_DIR=%LOCALAPPDATA%\AEPluginBuild"
cargo build --release -p red_noise
if errorlevel 1 (echo Build failed. & pause & exit /b 1)

if not exist "%AE_PLUGINS%" mkdir "%AE_PLUGINS%"
copy /Y "%CARGO_TARGET_DIR%\release\red_noise.dll" "%AE_PLUGINS%\red_noise.aex"
echo Deployed red_noise.aex. Start AE - if it starts, the crash is specific to island_id_color.
pause
