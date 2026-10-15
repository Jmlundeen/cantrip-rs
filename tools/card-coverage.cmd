@echo off
rem Which lines of a card's Rust file its scenarios never run: tools\card-coverage.cmd "Card Name".
rem Exits 1 if any line never ran. Needs cargo-llvm-cov; see tools\maintenance\card-coverage.py.
python "%~dp0maintenance\card-coverage.py" %*
