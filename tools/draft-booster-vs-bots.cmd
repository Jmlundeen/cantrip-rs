@echo off
rem ---------------------------------------------------------------------------
rem A booster draft against bots, a pod of eight: you in your browser, and seven bots
rem in the other seats. Three packs of fifteen from LSVCube, deck building,
rem then three Swiss rounds of best-of-three - the bots play every game, yours
rem included, each at a table of its own. Through tools\scripts\draft.ps1.
rem
rem   draft-booster-vs-bots.cmd                         you and seven bots, LSVCube
rem   draft-booster-vs-bots.cmd -Cube PowerLSV          another cube
rem   draft-booster-vs-bots.cmd -PackSize 5 -Rounds 1   a quick one
rem   draft-booster-vs-bots.cmd -Humans 2               two tabs and six bots
rem   draft-booster-vs-bots.cmd -Event ID               host a pod already made again
rem
rem The pod's table and the server's progress are in the window it opens; the
rem bots' logs are %TEMP%\cantrip-draft-seat-N.log. The event, with every pick,
rem is kept in .cantrip-data (or -Data, or CANTRIP_DATA).
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\draft.ps1" -Format Booster %*

if errorlevel 1 pause
exit /b %errorlevel%
