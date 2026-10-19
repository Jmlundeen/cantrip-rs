@echo off
rem ---------------------------------------------------------------------------
rem Publishes main to the public repository, cantrip-rs/cantrip-rs: each commit on
rem origin/main since the last one published is replayed there as its own
rem commit, with the same tree, message, author and date.
rem
rem   push-public.cmd             lists what would go, asks, then pushes
rem   push-public.cmd -DryRun     only lists it
rem   push-public.cmd -Yes        pushes without asking
rem
rem Everything tracked goes public, exactly as it is on origin/main - push there
rem first. The public main must match a commit here; if someone committed to
rem cantrip-rs directly, the script stops instead of overwriting it.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0push-public.ps1" %*
exit /b %ERRORLEVEL%
