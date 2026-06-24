@echo off
setlocal
set "AE_PLUGINS=C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore"
set "PNAME=island_id_color"

echo This script RENAMES the plugin so AE will NOT load it.
echo Use this to confirm that the startup crash is caused by this plugin.
echo.
if exist "%AE_PLUGINS%\%PNAME%.aex" (
    ren "%AE_PLUGINS%\%PNAME%.aex" "%PNAME%.aex.bak"
    echo Renamed: %PNAME%.aex -^> %PNAME%.aex.bak
    echo Now start After Effects. If AE starts without crash, the plugin is the trigger.
    echo To restore: run build_and_deploy.bat or manually ren .aex.bak back to .aex
) else (
    echo No file: %AE_PLUGINS%\%PNAME%.aex
)
pause

