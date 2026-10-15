@echo off
rem ---------------------------------------------------------------------------
rem Puts a new build behind the Discord bot, through tools\scripts\discord-bot-release.ps1:
rem stops the bot and the release server (play-release-build.cmd's), publishes the
rem web client in Release, and starts the bot again in a window of its own.
rem The quick tunnel is left running, and the bot comes back at its address.
rem
rem   discord-bot-release.cmd                    rebuild and restart
rem   discord-bot-release.cmd -Fast              skip ahead-of-time compiling
rem   discord-bot-release.cmd -PublicUrl URL     the address, when not the tunnel's
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\discord-bot-release.ps1" %*

rem Left open on failure, so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
