@echo off
rem ---------------------------------------------------------------------------
rem A grid draft against a bot: you in your browser, the bot in the other
rem seat. Eighteen packs of nine from LSVCube, each laid out in a 3x3 grid, a
rem row or a column each; then each of you builds a deck from what you took
rem and you play the game with it. Through tools\scripts\draft.ps1.
rem
rem   draft-grid-vs-bot.cmd                        18 grids of LSVCube
rem   draft-grid-vs-bot.cmd -Cube PowerLSV         another cube
rem   draft-grid-vs-bot.cmd -Packs 12              fewer grids
rem   draft-grid-vs-bot.cmd -NoBuild               play on the last build
rem
rem The event is kept in .cantrip-data (or -Data, or CANTRIP_DATA); host it
rem again with tools\draft.cmd -Event ID. The bot's log is
rem %TEMP%\cantrip-draft-seat-2.log.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\draft.ps1" -Format Grid %*

if errorlevel 1 pause
exit /b %errorlevel%
