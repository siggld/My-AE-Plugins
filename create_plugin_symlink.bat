@echo off
setlocal
:: Create symbolic link from build output to AE Plug-ins folder (no copy on each build).
:: Run as Administrator: mklink requires elevation on Windows.

set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins\AOD"
set "PNAME=island_id_color"
set "CARGO_TARGET_DIR=%LOCALAPPDATA%\AEPluginBuild"
set "DLL_SRC=%CARGO_TARGET_DIR%\release\%PNAME%.dll"

if not exist "%DLL_SRC%" (
    echo ERROR: Build first. Not found: %DLL_SRC%
    echo Run build_and_deploy.bat once to build, or: set CARGO_TARGET_DIR=%%LOCALAPPDATA%%\AEPluginBuild && cargo build --release -p %PNAME%
    pause
    exit /b 1
)

if not exist "%AE_PLUGINS%" mkdir "%AE_PLUGINS%"

set "AEX_LINK=%AE_PLUGINS%\%PNAME%.aex"
if exist "%AEX_LINK%" (
    echo Target already exists: %AEX_LINK%
    echo Remove it first if you want to create a new link (del or unlink).
    pause
    exit /b 0
)

mklink "%AEX_LINK%" "%DLL_SRC%"
if errorlevel 1 (
    echo mklink failed. Run this script as Administrator.
    pause
    exit /b 1
)
echo Created symlink: %AEX_LINK% -> %DLL_SRC%
echo Rebuild with: build_and_deploy.bat (skip deploy step) or: cargo build --release -p %PNAME%
pause
