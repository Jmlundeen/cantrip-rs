@echo off
rem ---------------------------------------------------------------------------
rem Plays a game in the shipping web client through tools\scripts\play-release-build.ps1:
rem builds it if it is out of date (Release, compiled ahead of time; only what
rem changed is compiled again, though the app is linked again each time),
rem starts a server with the playtest deck (demo/card-test) for both seats,
rem puts a bot in seat 2, and opens seat 1 in your browser.
rem
rem   play-release-build.cmd               build if needed, then play
rem   play-release-build.cmd -NoBuild      play on the last build
rem   play-release-build.cmd -Fast         skip ahead-of-time compiling: quick to
rem                                 build, interpreted to run
rem   play-release-build.cmd -Seed 7       the shuffle and the bot's draws
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play-release-build.ps1" %*

rem Left open on failure, so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
