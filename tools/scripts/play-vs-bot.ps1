# You against the tactical bot, in the web client: play-game.ps1 with seat 1 yours and seat 2 the bot's, which your
# own tab plays (?bots=2).
#
#   play-vs-bot.ps1                                   asks for both decks
#   play-vs-bot.ps1 -Mine ub-reanimator -Theirs rw-aggro
#   play-vs-bot.ps1 -Defaults                         RW aggro for both, no questions
#
#   -Seed N     the shuffle (default: a new one each game, printed so a game can be repeated)
#   -Defaults   take the default for anything not given rather than asking
#   -NoBuild    skip the build and use the web client already being served, if there is one

[CmdletBinding()]
param(
    [string] $Mine,
    [string] $Theirs,
    [uint64] $Seed = 0,
    [switch] $Defaults,
    [switch] $NoBuild
)

$arguments = @{ Player1 = 'Human'; Player2 = 'Tactical'; Seed = $Seed }
if ($Mine) { $arguments.Deck1 = $Mine }
if ($Theirs) { $arguments.Deck2 = $Theirs }

# The deck the bot's weights were fitted with, for both seats, when nothing is asked: the game it plays best.
if ($Defaults) {
    if (-not $Mine) { $arguments.Deck1 = 'rw-aggro' }
    if (-not $Theirs) { $arguments.Deck2 = 'rw-aggro' }
    $arguments.Defaults = $true
}

if ($NoBuild) { $arguments.NoBuild = $true }

& (Join-Path $PSScriptRoot 'play-game.ps1') @arguments
exit $LASTEXITCODE
