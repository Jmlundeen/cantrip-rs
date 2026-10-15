# Opens the web client with the opposing seat played by a language model, so you can play against it yourself.
#
# Asks for both decks, with the default in brackets, so a double-click on tools\play-vs-llm.cmd walks through it.
# Anything given on the command line is not asked for.
#
#   play-vs-llm.ps1                                     asks for both decks
#   play-vs-llm.ps1 -Mine rw-aggro -Theirs ub-reanimator
#   play-vs-llm.ps1 -Defaults                           the demo decks, no questions
#
# A deck is a decklist under decks\, named without .txt ("rw-aggro", "demo/2-white-blue"), or a path.
#
# Needs an OpenRouter key: set OPENROUTER_API_KEY in the environment, or run
#   setx OPENROUTER_API_KEY "sk-or-v1-..."
# and open a new terminal. Roughly two to five cents a game.
#
# The model's seat is dotnet\tools\Cantrip.Bot asking the model, a second or two over each decision. If the model cannot be
# reached the seat falls back to the game's own answer for that prompt and play goes on, so a dropped connection costs
# a bad move rather than the game. Its questions and answers are written to a transcript its log names
# (%TEMP%\cantrip-seat-2.log).
#
# This is tools\scripts\play-game.ps1 with you in seat 1 and the model in seat 2:
#
#   -Model ID      the OpenRouter model (default: the client's own default)
#   -Seed N        the shuffle, so the same deal can be played again (default: a new one each run)
#   -Defaults      take the default for anything not given rather than asking
#   -NoBuild       skip the build; what is already built is what runs

[CmdletBinding()]
param(
    [string] $Mine,
    [string] $Theirs,
    [string] $Model,
    [uint64] $Seed = 0,
    [switch] $Defaults,
    [switch] $NoBuild
)

$arguments = @{ Player1 = 'Human'; Player2 = 'Llm'; Seed = $Seed }
if ($Mine) { $arguments.Deck1 = $Mine }
if ($Theirs) { $arguments.Deck2 = $Theirs }
if ($Model) { $arguments.Model = $Model }
if ($Defaults) { $arguments.Defaults = $true }
if ($NoBuild) { $arguments.NoBuild = $true }

& (Join-Path $PSScriptRoot 'play-game.ps1') @arguments
exit $LASTEXITCODE
