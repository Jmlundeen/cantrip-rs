@echo off
rem The worktree pool: acquire, token, join, leave, release, status, reconcile. See tools\maintenance\worktrees.py.
python "%~dp0maintenance\worktrees.py" %*
