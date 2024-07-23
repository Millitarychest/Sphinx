@echo off
REM This is the layout of the start script executed by Sphinx to run a project
REM Replace "{command}" with the actual command you want to execute
cd %~dp0
{command}

if errorlevel 1 (
    echo No build found
)
pause