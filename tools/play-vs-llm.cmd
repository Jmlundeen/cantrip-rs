@echo off
rem ---------------------------------------------------------------------------
rem Opens the web client with the opposing seat played by a language model, so
rem you can play against it yourself.
rem
rem Asks for both decks, so a double-click walks through it.
rem
rem   play-vs-llm.cmd                                 asks for both decks
rem   play-vs-llm.cmd -Mine rw-aggro -Theirs ub-reanimator
rem   play-vs-llm.cmd -Defaults                       the demo decks, no questions
rem   play-vs-llm.cmd -Model qwen/qwen3.7-flash
rem   play-vs-llm.cmd -Seed 7                         the same deal every time
rem
rem Needs an OpenRouter key: set OPENROUTER_API_KEY in the environment, or run
rem   setx OPENROUTER_API_KEY "sk-or-v1-..."
rem and open a new terminal. Roughly two to five cents a game.
rem
rem Every argument is passed to tools\scripts\play-vs-llm.ps1; see its header for the rest
rem (-Defaults, -NoBuild).
rem
rem The other ways to watch this seat play, when playing it yourself is not the
rem question:
rem   bots-llm-games.cmd   whole games against the random bot, with a transcript
rem   bots-deck-duel.cmd   two decks against each other, random bots, free and fast
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play-vs-llm.ps1" %*

rem Left open only when the game could not start - no key, a mistyped deck, a
rem build that failed - so a double-clicked window does not vanish with the
rem reason in it.
if errorlevel 1 pause
exit /b %errorlevel%
