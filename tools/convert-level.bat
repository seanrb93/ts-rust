@echo off
rem Double-click: asks for the map or model, then makes the level (see docs/level-converter.md).
python "%~dp0obj2level.py"
if errorlevel 1 pause
