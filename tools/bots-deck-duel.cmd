@echo off
rem ---------------------------------------------------------------------------
rem Plays two decks against each other many times and says which won more.
rem
rem Asks for both decks and how many games, so a double-click walks through it.
rem
rem   bots-deck-duel.cmd                                  asks for everything
rem   bots-deck-duel.cmd -Games 2000 -Defaults            the demo decks, no questions
rem   bots-deck-duel.cmd -Deck1 card-test -Deck2 demo/2-white-blue -Games 500
rem
rem The games are played by the randomized bot at the session layer — no client
rem and no socket, every core at once — which is the fastest whole games this
rem repository can play. Neither bot plays well; both play the same way, so what
rem differs between the two columns is the decks.
rem
rem Seats are swapped every other game, so each deck plays first in half of them
rem and the tally is not counting who drew fewer cards.
rem
rem Every argument is passed to tools\scripts\deck-duel.ps1; see its header for the rest
rem (-Seed, -BotSeed, -Parallel, -Defaults, -NoBuild).
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\deck-duel.ps1" %*

rem Left open only when the run could not start — a mistyped deck, a build that
rem failed — so a double-clicked window does not vanish with the reason in it. A
rem run that played and reported is not held open, even though it exits non-zero
rem when some game did not finish: the tally is the output, and it has printed.
if errorlevel 2 pause
exit /b %errorlevel%
