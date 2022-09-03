@echo off
chim "%~dp0\node" %* ; Runs `chim .\node`
EXIT /B %ERRORLEVEL% ; Return the errorlevel of the last command
