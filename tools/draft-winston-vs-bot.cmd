@echo off
rem ---------------------------------------------------------------------------
rem A Winston draft against a bot: you in your browser, the bot in the other
rem seat. 90 cards of LSVCube in three piles, then each of you builds a deck
rem from what you took and you play the game with it. Through
rem tools\scripts\draft.ps1.
rem
rem   draft-winston-vs-bot.cmd                     90 cards of LSVCube
rem   draft-winston-vs-bot.cmd -Cube PowerLSV      another cube
rem   draft-winston-vs-bot.cmd -Cards 60           a shorter draft
rem   draft-winston-vs-bot.cmd -NoBuild            play on the last build
rem
rem The event is kept in .cantrip-data (or -Data, or CANTRIP_DATA); host it
rem again with tools\draft.cmd -Event ID. The bot's log is
rem %TEMP%\cantrip-draft-seat-2.log.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\draft.ps1" -Format Winston %*

if errorlevel 1 pause
exit /b %errorlevel%
