@echo off
rem ---------------------------------------------------------------------------
rem Publishes card art to the server without taking it down, through
rem tools\scripts\publish-art.ps1: brings the pre-sized art cache up to date here,
rem and sends the renditions that are new or changed, and the credits.
rem
rem   publish-art.cmd                 build the cache and send what differs
rem   publish-art.cmd -DryRun         say what would be sent, send nothing
rem   publish-art.cmd -SkipBuild      send the cache as it is
rem
rem Where the server is, and its key, come from tools\deploy.local.json (kept out
rem of git; the format is in tools\scripts\deploy-common.ps1), or -Server and -Key.
rem publish-server.cmd does this too.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\publish-art.ps1" %*

rem Left open on failure, so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
