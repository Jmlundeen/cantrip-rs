@echo off
rem ---------------------------------------------------------------------------
rem Plays random games at the session layer and reports what happened: the real
rem session and protocol, in one process, no client and no socket. This is the
rem fast layer — around 20 games a second on a many-core machine — so it is
rem what to reach for when the question needs volume rather than a screen.
rem
rem   bots-random-games.cmd                          100 games against the demo decks
rem   bots-random-games.cmd --games 2000             a bulk run
rem   bots-random-games.cmd --games 50 --verbose     one line per game
rem   bots-random-games.cmd --deck card-test --deck demo/2-white-blue
rem   bots-random-games.cmd --games 1 --seed 7 --bot-seed 113   one game again
rem
rem Each run deals a different set of games. The pair it drew is printed at the
rem top, so giving it back plays exactly the same run a second time.
rem
rem Every argument is passed through; run with --help for the rest (--parallel,
rem --players, --seed, --bot-seed, --deck). Exits non-zero if any game was left
rem stuck or had an answer refused, and prints, for each thing it found, the whole
rem command that plays that one game again — seeds, decks and the seating.
rem
rem The other layers, when this one is not the question:
rem   play-game.cmd   a real server, a socket and the web client
rem   test.cmd        every test project, plus one smoke game in the client
rem ---------------------------------------------------------------------------

setlocal

set "ROOT=%~dp0.."

dotnet build "%ROOT%\tools\Cantrip.Playtest\Cantrip.Playtest.csproj" -v q --nologo || exit /b 1
dotnet run --project "%ROOT%\tools\Cantrip.Playtest\Cantrip.Playtest.csproj" --no-build -v q -- %*

endlocal & exit /b %ERRORLEVEL%
