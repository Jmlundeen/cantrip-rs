@echo off
rem ---------------------------------------------------------------------------
rem Plays games with a language model in one seat and prints every decision it
rem made: the question it was put, the choices it was offered, and what it
rem picked — with whatever reason it gave.
rem
rem   bots-llm-games.cmd                              one game, model vs rando-bot
rem   bots-llm-games.cmd --games 5                    five of them
rem   bots-llm-games.cmd --model qwen/qwen3.7-flash   a different model
rem   bots-llm-games.cmd --deck rw-aggro --deck ub-reanimator
rem   bots-llm-games.cmd --both                       the model plays both seats
rem   bots-llm-games.cmd --verbose                    also print the whole position
rem                                                   the model was shown
rem
rem Needs an OpenRouter key: set OPENROUTER_API_KEY in the environment, or pass
rem --key. One is made at https://openrouter.ai/keys.
rem
rem The measure here is the transcript, not the tally. A game costs money and
rem minutes, so a hundred of them is not the first thing to reach for; one game
rem read end to end says whether the model is playing Magic or flailing. The
rem tally is printed anyway, because beating the random seat is the bar.
rem
rem Every argument is passed through; run with --help for the rest (--seed,
rem --bot-seed, --players, --temperature-percent, --quiet).
rem
rem The other layers, when this one is not the question:
rem   bots-random-games.cmd   thousands of random games a minute, and free
rem   test.cmd                every test project, plus one smoke game in the client
rem ---------------------------------------------------------------------------

setlocal

set "ROOT=%~dp0.."

dotnet build "%ROOT%\tools\Cantrip.LlmGames\Cantrip.LlmGames.csproj" -v q --nologo || exit /b 1
dotnet run --project "%ROOT%\tools\Cantrip.LlmGames\Cantrip.LlmGames.csproj" --no-build -v q -- %*

endlocal & exit /b %ERRORLEVEL%
