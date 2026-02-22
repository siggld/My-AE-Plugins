@echo off
setlocal

:: ============================================================
:: build_and_deploy.bat - ASCII only, NO Japanese characters
:: Builds island_id_color plugin and deploys to AE 2024
:: Run as Administrator for registry/MediaCore cleanup
:: Visual Studio 2022 is the designated build environment (matches GitHub CI).
:: ============================================================

:: --- Paths ---
set "VS_2022=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
set "VS_2022_BUILDTOOLS=C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
set "VS_18=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"

set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins\AOD"
set "PNAME=island_id_color"

:: Use VS 2022 (Community or Build Tools); fallback to VS 18 only if 2022 is not installed
if exist "%VS_2022%" (
    set "VCVARS=%VS_2022%"
) else if exist "%VS_2022_BUILDTOOLS%" (
    set "VCVARS=%VS_2022_BUILDTOOLS%"
) else (
    set "VCVARS=%VS_18%"
)

cd /d "%~dp0"

:: Build output on C: to avoid "output path is not a writable directory" on sync drives (e.g. G:)
set "CARGO_TARGET_DIR=%LOCALAPPDATA%\AEPluginBuild"
if not exist "%CARGO_TARGET_DIR%" mkdir "%CARGO_TARGET_DIR%"

:: ============================================================
:: STEP 1: Load Visual Studio Build Environment
:: ============================================================
echo [1/4] Loading Visual Studio Build Environment...
echo   Using: %VCVARS%
if not exist "%VCVARS%" (
    echo ERROR: vcvars64.bat not found.
    echo Checked: VS 2022 Community, VS 2022 Build Tools, VS 18
    pause
    exit /b 1
)
call "%VCVARS%"

:: ============================================================
:: STEP 2: Purge ghost cache files (MediaCore + Registry)
:: ============================================================
echo.
echo [2/4] Purging stale cache...

echo   - Removing MediaCore entries...
powershell -NoProfile -Command "Get-ChildItem -Path 'C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore' -Filter '*%PNAME%*' -Recurse -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue; Write-Host '  MediaCore cleanup done.'"

echo   - Removing AE PluginCache registry key...
powershell -NoProfile -Command "Remove-Item -Path 'HKCU:\Software\Adobe\After Effects\24.6\PluginCache' -Recurse -ErrorAction SilentlyContinue; Write-Host '  Registry cleanup done.'"

:: ============================================================
:: STEP 3: Build
:: ============================================================
echo.
echo [3/4] Building plugin (cargo clean + release build)...
cargo clean
cargo build --release -p %PNAME%
if errorlevel 1 (
    echo ERROR: Build failed.
    pause
    exit /b 1
)

:: ============================================================
:: STEP 4: Deploy to AE Plug-ins folder
:: ============================================================
echo.
echo [4/4] Deploying to After Effects...
if not exist "%CARGO_TARGET_DIR%\release\%PNAME%.dll" (
    echo ERROR: Build output not found: %CARGO_TARGET_DIR%\release\%PNAME%.dll
    pause
    exit /b 1
)

if not exist "%AE_PLUGINS%" mkdir "%AE_PLUGINS%"
if exist "%AE_PLUGINS%\%PNAME%.aex" del /f /q "%AE_PLUGINS%\%PNAME%.aex"
copy /Y "%CARGO_TARGET_DIR%\release\%PNAME%.dll" "%AE_PLUGINS%\%PNAME%.aex"
if errorlevel 1 (
    echo ERROR: Copy failed. Try running as Administrator.
    pause
    exit /b 1
)

echo.
echo ============================================================
echo DONE! Deployed: %AE_PLUGINS%\%PNAME%.aex
echo Next: Launch AE and verify plugin loads under Aodaruma/AOD_IslandIdColor
echo ============================================================
pause