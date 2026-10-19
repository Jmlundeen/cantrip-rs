@echo off
rem ---------------------------------------------------------------------------
rem Publishes Cantrip to the server, through tools\scripts\publish-server.ps1:
rem builds the Discord bot (linux-arm64) and the web client here, sends the
rem source and data and builds the engine host on the server, publishes the art
rem (publish-art.cmd), and swaps the new build in with one restart of the bot.
rem If the bot does not come back, the old one is put back.
rem
rem   publish-server.cmd                   everything
rem   publish-server.cmd -Fast             a web client without ahead-of-time compiling
rem   publish-server.cmd -SkipClient       reuse the web client built last time
rem   publish-server.cmd -SkipArt          leave the art alone
rem
rem Where the server is, and its key, come from tools\deploy.local.json (kept out
rem of git; the format is in tools\scripts\deploy-common.ps1), or -Server and -Key.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\publish-server.ps1" %*

rem Left open on failure, so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
