@echo off
rem Plays one .cantrip scenario with Bot1 a browser page, in a window to watch:
rem   tools\scenario-in-browser.cmd black-lotus-sacrificed-for-mana-is-taken-back-and-can-be-used-again
rem The name is the file's in scenarios\, with or without .cantrip; it need not be marked # Browser.
rem /headless runs it without a window. Builds what it needs (the host, the web client) first.
rem docs\scenarios-in-the-browser.md says what it does; tools\test.cmd /filter "FullyQualifiedName~ScenarioInBrowser"
rem plays every scenario marked # Browser.
setlocal
if "%~1"=="" (
  echo Usage: tools\scenario-in-browser.cmd ^<scenario name^> [/headless]
  exit /b 2
)
set "CANTRIP_SCENARIO=%~n1"
if /i not "%~2"=="/headless" set "CANTRIP_WEB_HEADED=1"
call "%~dp0test.cmd" /filter "FullyQualifiedName~ScenarioInBrowserTests"
exit /b %ERRORLEVEL%
