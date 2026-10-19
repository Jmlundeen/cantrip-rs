@echo off
rem ---------------------------------------------------------------------------
rem Plays a few thousand games between bots and says only which did not finish.
rem
rem   bots-soak.cmd                              2000 games of the every-card deck, the tactical bot in both seats
rem   bots-soak.cmd --games 5000 --parallel 16
rem   bots-soak.cmd --decks artifacts\drafted\final        and a pool of drafted decks, each a mirror
rem   bots-soak.cmd --a random --b tactical                another pair of seats
rem
rem This is fuzzing, on purpose: random shuffles, nobody trying to reach any
rem position, nothing asserted about who wins. A game that throws, stalls or
rem never ends is a bug in the engine, the session or a bot's own loop guards,
rem and each is printed with the command that plays it again exactly:
rem
rem   tools\bot-duel.cmd watch --game 17 --seed 123456 --a tactical --b tactical --deck "demo/card-test"
rem
rem which is where a scenario for it starts (scenarios\).
rem Exits 1 if any game did not finish, 0 if every one did.
rem
rem It is not a measure of how well a bot plays: that is bot-duel.cmd.
rem Every argument is passed through; run cantrip-botlab --help for the rest.
rem ---------------------------------------------------------------------------

setlocal

set "ROOT=%~dp0.."

dotnet build "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" -v q --nologo || exit /b 2
dotnet run --project "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" --no-build -v q -- soak %*

endlocal & exit /b %ERRORLEVEL%
