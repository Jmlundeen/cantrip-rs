@echo off
rem ---------------------------------------------------------------------------
rem Makes the tactical bot's weights: dotnet\src\Cantrip.Client\Tactics\tactical-model.txt,
rem which the client carries with it.
rem
rem   bot-train.cmd                 fit, then tune, with RW aggro: about an hour
rem   bot-train.cmd fit             only the fit
rem   bot-train.cmd tune            only the tuning, from the weights that ship
rem   bot-train.cmd tune --deck rw-aggro --deck ub-reanimator --steps 500
rem
rem Two stages, and each has to earn its keep by beating what it started from
rem over a few thousand games, or nothing is written.
rem
rem   fit    The bot plays itself. Every position is marked with who went on to
rem          win, and the weights are fitted to that: what a creature, a card in
rem          hand and a point of life are each worth, and what each card is worth
rem          beyond its size. It starts from the hand-written prior and is held
rem          towards it where the games say little.
rem
rem   tune   Every weight, the ones the bot plays by included, is nudged up or
rem          down at random and the two nudged bots play each other; the weights
rem          move a little towards the winner. Hundreds of times. This is what
rem          the fit cannot do: it asks which bot wins, not which predicts.
rem
rem Arguments after a stage go to that stage of cantrip-botlab; run it with
rem --help for what each takes (--games, --rounds, --steps, --from, --out).
rem With no stage named they go to both, which is what --deck is for.
rem
rem After a run: bot-duel.cmd says what the new weights are worth, and test.cmd
rem that nothing else moved.
rem ---------------------------------------------------------------------------

setlocal

set "ROOT=%~dp0.."
set "LAB=dotnet run --project "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" --no-build -v q --"

dotnet build "%ROOT%\dotnet\tools\Cantrip.BotLab\Cantrip.BotLab.csproj" -v q --nologo || exit /b 2

if /i "%~1"=="fit" goto fit
if /i "%~1"=="tune" goto tune

%LAB% train %*
if errorlevel 2 exit /b %ERRORLEVEL%

rem The tuning starts from the file the fit just wrote, which is not in the build yet.
%LAB% tune --from "%ROOT%\dotnet\src\Cantrip.Client\Tactics\tactical-model.txt" %*
endlocal & exit /b %ERRORLEVEL%

:fit
shift
%LAB% train %1 %2 %3 %4 %5 %6 %7 %8 %9
endlocal & exit /b %ERRORLEVEL%

:tune
shift
%LAB% tune --from "%ROOT%\dotnet\src\Cantrip.Client\Tactics\tactical-model.txt" %1 %2 %3 %4 %5 %6 %7 %8 %9
endlocal & exit /b %ERRORLEVEL%
