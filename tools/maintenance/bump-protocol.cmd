@echo off
rem ---------------------------------------------------------------------------
rem Moves PlayerView.ProtocolVersion on by one and records the wire digest that
rem goes with it.
rem
rem   bump-protocol.cmd           next version, digest recorded for you
rem   bump-protocol.cmd 61        that version instead
rem
rem The digest is never typed by hand: WireShapeTests prints the line to record,
rem and this reads it back. Run this whenever the wire changes shape — a new
rem message, a new field, a new enum member that crosses the wire.
rem
rem Cards added in parallel each want the next version, and only one of them can
rem have it. So an agent runs this in its own worktree to stay green, and the
rem coordinator runs it again after the cherry-pick to land on the number main
rem actually has. Resolve a conflict in PlayerView.cs or WireShapeTests.cs by
rem taking either side and running this; do not merge the two by hand.
rem ---------------------------------------------------------------------------

setlocal

set "TOOLS=%~dp0"
set "ARGS="
if not "%~1"=="" set "ARGS=-To %~1"

powershell -NoProfile -ExecutionPolicy Bypass -File "%TOOLS%bump-protocol.ps1" %ARGS%

endlocal & exit /b %ERRORLEVEL%
