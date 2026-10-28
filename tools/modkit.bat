@echo off
rem Double-click: the mod kit (menus for making levels, characters, weapons, game modes, sounds, music and pictures).  See docs/modding-guide.md.
python "%~dp0modkit-entry.py"
if errorlevel 1 pause
