                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             @echo off
setlocal

:: ============================================================
:: build_and_deploy_W.bat - Build on W: drive (for sync-drive workaround)
:: ASCII only, NO Japanese characters.
:: Use when repo is on W: (e.g. W:\work\My-AE-Plugins). Build output
:: is placed in repo's target folder on W: to avoid G: sync issues.
:: Run from W: repo root. Visual Studio 2022 is required.
::
:: vcvars64.bat の場所（VS 2022 が使われているか確認するときの目安）:
::   Community:  C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat
::   BuildTools: C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
:: 実行時に "Using: ..." で表示されるパスが 2022 なら VS 2022 でビルドされている。
:: ============================================================

set "VS_2022=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
set "VS_2022_BUILDTOOLS=C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
set "VS_18=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"

set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins\AOD"
set "PNAME=island_id_color"

if exist "%VS_2022%" (set "VCVARS=%VS_2022%") else if exist "%VS_2022_BUILDTOOLS%" (set "VCVARS=%VS_2022_BUILDTOOLS%") else (set "VCVARS=%VS_18%")

cd /d "%~dp0"

:: Build output on same drive as repo (W:) to avoid sync-drive DLL load issues
set "CARGO_TARGET_DIR=%~dp0target"
if not exist "%CARGO_TARGET_DIR%" mkdir "%CARGO_TARGET_DIR%"
echo [W:] CARGO_TARGET_DIR=%CARGO_TARGET_DIR%

echo [1/4] Loading Visual Studio Build Environment...
echo   Using: %VCVARS%
if not exist "%VCVARS%" (
    echo ERROR: vcvars64.bat not found.
    pause
    exit /b 1
)
call "%VCVARS%"

echo.
echo [2/4] Purging stale cache...
powershell -NoProfile -Command "Get-ChildItem -Path 'C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore' -Filter '*%PNAME%*' -Recurse -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue; Write-Host '  MediaCore cleanup done.'"
powershell -NoProfile -Command "Remove-Item -Path 'HKCU:\Software\Adobe\After Effects\24.6\PluginCache' -Recurse -ErrorAction SilentlyContinue; Write-Host '  Registry cleanup done.'"

echo.
echo [3/4] Building plugin (cargo clean + release build)...
cargo clean
cargo build --release -p %PNAME%
if errorlevel 1 (
    echo ERROR: Build failed.
    pause
    exit /b 1
)

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
echo Built from W: (or current drive) target. Launch AE to verify.
echo ============================================================
pause
