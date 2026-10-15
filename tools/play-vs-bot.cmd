@echo off
rem ---------------------------------------------------------------------------
rem Opens the web client with the opposing seat played by the tactical bot, so
rem you can play against it yourself.
rem
rem Asks for both decks, so a double-click walks through it.
rem
rem   play-vs-bot.cmd                                 asks for both decks
rem   play-vs-bot.cmd -Mine ub-reanimator -Theirs rw-aggro
rem   play-vs-bot.cmd -Defaults                       RW aggro for both, no questions
rem   play-vs-bot.cmd -Seed 7                         the same deal every time
rem
rem The bot runs in your own browser tab: the page that is your seat also plays
rem the seat across from you, over a second connection of its own (?bots=2 in
rem its address). Nothing is asked of the server but to seat it, and nothing
rem leaves your machine. It reads the board the way a player does, works out
rem combat before it happens, and scores what it sees with weights fitted to
rem games it played against itself (bot-train.cmd).
rem
rem Every argument is passed to tools\scripts\play-vs-bot.ps1; see its header
rem for the rest (-Defaults, -NoBuild).
rem
rem The other ways to see this seat play, when playing it yourself is not the
rem question:
rem   bot-duel.cmd    it against the random bot, or any seat against any other
rem   bot-train.cmd   how its weights are made
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play-vs-bot.ps1" %*

rem Left open only when the game could not start - a mistyped deck, a build
rem that failed - so a double-clicked window does not vanish with the reason in
rem it.
if errorlevel 1 pause
exit /b %errorlevel%
