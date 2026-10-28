@echo off
rem Wait for pull requests on cantrip-rs/cantrip-rs and clear the queue with a fresh Claude Code session each time:
rem tools\pr-watch.cmd [--interval 60] [--once] [--mark-seen] [--include-drafts] [--dry-run]. See tools\maintenance\pr-watch.py.
python "%~dp0maintenance\pr-watch.py" %*
