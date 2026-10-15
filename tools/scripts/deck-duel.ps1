# Plays two decks against each other many times and says which won more.
#
# Asks for both decks and how many games, with the default in brackets, so a double-click on tools\bots-deck-duel.cmd walks
# through it. Anything given on the command line is not asked for.
#
#   deck-duel.ps1                                        asks for everything
#   deck-duel.ps1 -Deck1 demo/1-black-red -Deck2 demo/2-white-blue -Games 500
#   deck-duel.ps1 -Games 2000 -Defaults                  the demo decks, no questions
#
# A deck is a decklist under decks\, named without .txt ("card-test", "demo/2-white-blue"), or a path.
#
# The games are played by the randomized bot at the session layer: no client and no socket, every core at once, which
# is the fastest way this repository can play whole games. Neither bot plays well — the point is that both play the
# same way, so what differs between the two columns is the decks.
#
# The seats are swapped every other game, so each deck plays first in half of them and the tally is not counting who
# drew fewer cards. Each run deals a different set of games, because a tally of the same two hundred games run again
# is the same tally. The pair they came from is printed at the top, so a run is repeated exactly by giving it back as
# -Seed and -BotSeed, and any one game of it can be replayed on its own with the command the report prints for it.
#
#   -Seed N        the first shuffle (default: a new one each run)
#   -BotSeed N     the first seat's draws (default: a new one each run)
#   -Parallel N    games at once (default: one per core, at most 32)
#   -Defaults      take the default for anything not given rather than asking
#   -NoBuild       skip the build; what is already built is what runs

[CmdletBinding()]
param(
    [string] $Deck1,
    [string] $Deck2,
    [int] $Games,
    [int] $Seed,
    [int] $BotSeed,
    [int] $Parallel,
    [switch] $Defaults,
    [switch] $NoBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$decksRoot = Join-Path $root 'decks'
$project = Join-Path $root 'dotnet\tools\Cantrip.Playtest\Cantrip.Playtest.csproj'

# Every decklist, named as the runner takes it: relative to decks\, forward slashes, no extension.
$available = @(Get-ChildItem -Path $decksRoot -Recurse -Filter *.txt |
    ForEach-Object { $_.FullName.Substring($decksRoot.Length + 1).Replace('\', '/') -replace '\.txt$', '' } |
    Sort-Object)

if ($available.Count -eq 0) {
    Write-Host "No decklists under $decksRoot."
    exit 2
}

# Exit 2 rather than throwing, so a mistyped deck is the same "could not start" code the runner uses and the
# wrapper can tell it apart from a run that played and found something.
function Resolve-Deck([string] $deck) {
    if (Test-Path -LiteralPath $deck -PathType Leaf) { return (Resolve-Path -LiteralPath $deck).Path }
    if ($available -contains $deck) { return $deck }
    Write-Host "No deck '$deck'. Decks: $($available -join ', ')"
    exit 2
}

function Ask-Deck([string] $which, [string] $default) {
    while ($true) {
        $answer = "$(Read-Host "$which deck, by number or name [$default]")".Trim()
        if ($answer -eq '') { return $default }

        $number = 0
        if ([int]::TryParse($answer, [ref] $number) -and $number -ge 1 -and $number -le $available.Count) {
            return $available[$number - 1]
        }

        if (Test-Path -LiteralPath $answer -PathType Leaf) { return (Resolve-Path -LiteralPath $answer).Path }
        if ($available -contains $answer) { return $answer }
        Write-Host "No deck '$answer'. Pick a number from the list, or type a name or a path."
    }
}

function Ask-Games([int] $default) {
    while ($true) {
        $answer = "$(Read-Host "How many games [$default]")".Trim()
        if ($answer -eq '') { return $default }

        $number = 0
        if ([int]::TryParse($answer, [ref] $number) -and $number -ge 1) { return $number }
        Write-Host 'Answer a whole number of games, or nothing for the default.'
    }
}

$asking = -not $Defaults -and -not ($Deck1 -and $Deck2 -and $Games)

if ($asking) {
    Write-Host 'Decks:'
    for ($i = 0; $i -lt $available.Count; $i++) {
        Write-Host ('  {0,2}. {1}' -f ($i + 1), $available[$i])
    }
    Write-Host ''
}

if ($Deck1) { $Deck1 = Resolve-Deck $Deck1 }
elseif ($asking) { $Deck1 = Ask-Deck 'First' 'demo/1-black-red' }
else { $Deck1 = 'demo/1-black-red' }

if ($Deck2) { $Deck2 = Resolve-Deck $Deck2 }
elseif ($asking) { $Deck2 = Ask-Deck 'Second' 'demo/2-white-blue' }
else { $Deck2 = 'demo/2-white-blue' }

if (-not $Games) {
    $Games = if ($asking) { Ask-Games 200 } else { 200 }
}

if (-not $NoBuild) {
    & dotnet build $project -v q --nologo
    if ($LASTEXITCODE -ne 0) { exit 2 }
}

$arguments = @(
    '--games', $Games,
    '--deck', $Deck1,
    '--deck', $Deck2)

# Only when given: unpassed, the runner draws a new pair and says which, so a plain duel is a fresh set of games.
if ($PSBoundParameters.ContainsKey('Seed')) { $arguments += @('--seed', $Seed) }
if ($PSBoundParameters.ContainsKey('BotSeed')) { $arguments += @('--bot-seed', $BotSeed) }

if ($Parallel) { $arguments += @('--parallel', $Parallel) }

Write-Host ''
& dotnet run --project $project --no-build -v q -- @arguments
exit $LASTEXITCODE
