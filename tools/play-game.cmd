@echo off
rem ---------------------------------------------------------------------------
rem Launches a local test game through tools\scripts\play-game.ps1: a server, the web
rem client in your browser for each person, and a bot for each other seat.
rem
rem   play-game.cmd                     asks for each player's deck and whether
rem                                     a human, a random bot, the tactical bot
rem                                     or a language model plays it
rem   play-game.cmd -Defaults           you against one bot, no questions
rem   play-game.cmd -Deck1 legend-rule-test -Defaults
rem
rem Bot seats play at random - a different game every run, from -Seed, so one
rem worth seeing again can be. -SimpleBot plays the one deterministic way. A
rem Tactical seat is the bot meant to be played against (play-vs-bot.cmd).
rem
rem Every argument is passed to the script; see its header for the rest
rem (-Deck2, -Player1, -Player2, -Seed, -SimpleBot, -Model, -Port, -NoBuild).
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play-game.ps1" %*

rem Left open on failure, so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
