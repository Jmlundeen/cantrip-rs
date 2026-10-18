@echo off
rem ---------------------------------------------------------------------------
rem Drafts a cube with nobody at the table, to say whether one way of drafting
rem and building a deck is better than another: the measure of the draft bot.
rem
rem   bot-draft.cmd                                50 pods of eight, the cube drafter in alternate seats with the
rem                                                old pick-order one, and what their decks are
rem   bot-draft.cmd --games 2                      and every deck of one plays every deck of the other in its pod,
rem                                                the tactical bot playing both: who wins
rem   bot-draft.cmd --a new:Signals=0 --b new      one number of the drafter's against the one it ships with
rem   bot-draft.cmd --a new-picks --b plain        only its picks (the old deck building on both sides)
rem   bot-draft.cmd --pods 10 --out decks\drafted  writes every deck as a decklist, for bot-duel.cmd or a table
rem   bot-draft.cmd watch --seat 3                 one pod, pick by pick as one seat thought it, and its deck
rem   bot-draft.cmd watch --format grid            the same of a grid (or winston) table of two
rem   bot-draft.cmd watch --event ID --seat 3      a stored booster draft's seat, replayed from the event's log
rem   bot-draft.cmd calibrate                      what a pack usually still holds, for the drafter's tables
rem   bot-draft.cmd fit                            what a card is worth by its text, fitted to the pick orders
rem
rem A pod is the server's own booster draft with each seat shown what a client
rem is shown and nothing else, so it drafts and builds in milliseconds; the
rem games are at the session layer, every core at once, a few a second.
rem
rem Every argument is passed through; run with --help for the rest.
rem docs\draft-bot.md says what the drafter does and what these measured.
rem ---------------------------------------------------------------------------

setlocal

set "ROOT=%~dp0.."

dotnet build "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" -v q --nologo || exit /b 2
dotnet run --project "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" --no-build -v q -- draft %*

endlocal & exit /b %ERRORLEVEL%
