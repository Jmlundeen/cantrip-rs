# Launches a local test game: a server, the web client served beside it, a browser tab for each person, and a bot for
# each other seat.
#
# You choose each player's deck and who plays that seat. Anything not given on the command line is asked for, with the
# default in brackets, so a double-click on tools\play-game.cmd walks through it. -Defaults skips the questions and
# takes the default for everything not given.
#
#   play-game.ps1                                         asks for each choice
#   play-game.ps1 -Defaults                               you (demo/1-black-red) against a bot (demo/2-white-blue)
#   play-game.ps1 -Deck1 legend-rule-test -Defaults       the legend-rule deck against a bot
#   play-game.ps1 -Player1 Human -Player2 Human -Defaults two tabs, you play both seats
#   play-game.ps1 -Player2 Tactical -Defaults             you against the tactical bot (tools\play-vs-bot.cmd)
#   play-game.ps1 -Player2 Llm -Defaults                  you against a language model (tools\play-vs-llm.cmd)
#   play-game.ps1 -Player1 Bot -Player2 Bot -Defaults     two bots, no browser; the result is printed here
#
# A person's seat is a tab of your default browser, at http://localhost:5000 with the seat in its address, served by
# the web client's dev server. A bot seat is dotnet\tools\Cantrip.Bot without a window; its log goes to
# %TEMP%\cantrip-seat-N.log. An Llm seat is the same bot asking a language model over OpenRouter, which needs
# OPENROUTER_API_KEY and costs a few cents a game; its transcript is named in its log.
#
# Bots play at random, drawn from -Seed as the shuffle is, so a game worth seeing again can be played again. A Tactical
# seat is the bot meant to be played against (TacticalPlay). Across from a person it has no process of its own: that
# person's tab plays it, over a second connection (?bots=N), which is where it is meant to run. With nobody at the
# table it is dotnet\tools\Cantrip.Bot --tactical, as any other bot seat is.
#
#   -Seed N     the shuffle and the bots' draws (default: a new one each game, printed so a game can be repeated)
#   -SimpleBot  bot seats play the one deterministic way instead of drawing at random
#   -Model ID   the OpenRouter model an Llm seat asks (default: the client's own default)
#   -Port N     the port the server listens on (default 7777); its card art is served on the next one
#               The server deals through the native Rust engine (cantrip-native-host, built here with cargo from
#               the workspace at the repository root), so the decks' card names must be cards it has ported
#   -Clock N    each player's chess clock, in minutes (default: the table's hour); every game has the clocks every table
#               has (docs/events.md, "Clocks"), whoever's runs out loses
#   -NoBuild    skip the build and use the web client already being served, if there is one

[CmdletBinding()]
param(
    [string] $Deck1,
    [string] $Deck2,
    [ValidateSet('Human', 'Bot', 'Tactical', 'Llm')][string] $Player1,
    [ValidateSet('Human', 'Bot', 'Tactical', 'Llm')][string] $Player2,
    [uint64] $Seed = 0,
    [int] $Port = 7777,
    [string] $Model,
    [int] $Clock = 0,
    # The format the decks are played in, which the server checks them against: freeform (no deck-construction restriction),
    # limited, or a constructed format's key - standard, pioneer, modern, legacy, vintage, commander, penny.
    [string] $Format = 'freeform',
    [switch] $Defaults,
    [switch] $SimpleBot,
    [switch] $NoBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$decksRoot = Join-Path $root 'decks'
$webProject = Join-Path $root 'dotnet\src\Cantrip.Client.Web'
$serverProject = Join-Path $root 'dotnet\src\Cantrip.Server.Host\Cantrip.Server.Host.csproj'
$botProject = Join-Path $root 'dotnet\tools\Cantrip.Bot\Cantrip.Bot.csproj'
$bot = Join-Path $root 'dotnet\tools\Cantrip.Bot\bin\Debug\net8.0\cantrip-bot.exe'

# The web client's dev server: the page on 5000, which it proxies to the app host on 5286.
$webPort = 5000
$webHostPort = 5286

# --- Decks -----------------------------------------------------------------------------------------------------------

$available = @(Get-ChildItem -Path $decksRoot -Recurse -Filter *.txt |
    ForEach-Object { $_.FullName.Substring($decksRoot.Length + 1).Replace('\', '/') -replace '\.txt$', '' } |
    Sort-Object)

if ($available.Count -eq 0) {
    Write-Host "No decklists under $decksRoot."
    exit 1
}

function Resolve-Deck([string] $deck) {
    if (Test-Path -LiteralPath $deck -PathType Leaf) { return (Resolve-Path -LiteralPath $deck).Path }
    if ($available -contains $deck) { return $deck }
    throw "No deck '$deck'. Decks: $($available -join ', ')"
}

function Ask-Deck([int] $seat, [string] $default) {
    while ($true) {
        $answer = "$(Read-Host "Deck for player $seat, by number or name [$default]")".Trim()
        if ($answer -eq '') { return $default }

        $number = 0
        if ([int]::TryParse($answer, [ref] $number) -and $number -ge 1 -and $number -le $available.Count) {
            return $available[$number - 1]
        }

        try { return (Resolve-Deck $answer) } catch { Write-Host $_.Exception.Message }
    }
}

function Ask-Player([int] $seat, [string] $default) {
    while ($true) {
        $answer = "$(Read-Host "Player $seat is a (h)uman, a random (b)ot, the (t)actical bot or a (l)anguage model [$default]")".Trim().ToLowerInvariant()
        switch -Regex ($answer) {
            '^$' { return $default }
            '^h(uman)?$' { return 'Human' }
            '^b(ot)?$' { return 'Bot' }
            '^t(actical)?$' { return 'Tactical' }
            '^l(lm)?$' { return 'Llm' }
            default { Write-Host 'Answer h, b, t or l.' }
        }
    }
}

$asking = -not $Defaults -and -not ($Deck1 -and $Deck2 -and $Player1 -and $Player2)

if ($asking) {
    Write-Host 'Decks:'
    for ($i = 0; $i -lt $available.Count; $i++) {
        Write-Host ('  {0,2}. {1}' -f ($i + 1), $available[$i])
    }
    Write-Host ''
}

$decks = @{}
$players = @{}

foreach ($seat in 1, 2) {
    $deckGiven = Get-Variable -Name "Deck$seat" -ValueOnly
    $playerGiven = Get-Variable -Name "Player$seat" -ValueOnly
    $defaultDeck = @('demo/1-black-red', 'demo/2-white-blue')[$seat - 1]
    $defaultPlayer = @('Human', 'Bot')[$seat - 1]

    if ($deckGiven) { $decks[$seat] = Resolve-Deck $deckGiven }
    elseif ($asking) { $decks[$seat] = Ask-Deck $seat $defaultDeck }
    else { $decks[$seat] = $defaultDeck }

    if ($playerGiven) { $players[$seat] = $playerGiven }
    elseif ($asking) { $players[$seat] = Ask-Player $seat $defaultPlayer }
    else { $players[$seat] = $defaultPlayer }
}

if ($Seed -eq 0) {
    $Seed = [uint64](Get-Random -Minimum 1 -Maximum ([int]::MaxValue))
}

$humans = @(1, 2 | Where-Object { $players[$_] -eq 'Human' })
$machines = @(1, 2 | Where-Object { $players[$_] -ne 'Human' })

if (($machines | Where-Object { $players[$_] -eq 'Llm' }) -and -not $env:OPENROUTER_API_KEY) {
    Write-Host 'A language model seat needs OPENROUTER_API_KEY in this shell:'
    Write-Host ''
    Write-Host '  setx OPENROUTER_API_KEY "sk-or-v1-..."     then open a new terminal'
    Write-Host ''
    Write-Host 'A key is made at https://openrouter.ai/keys.'
    exit 1
}

# --- The build -------------------------------------------------------------------------------------------------------

function Stop-Listener([int] $port) {
    Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue |
        ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
}

function Test-Listening([int] $port) {
    try {
        $probe = New-Object Net.Sockets.TcpClient
        $probe.Connect('127.0.0.1', $port)
        $probe.Close()
        return $true
    }
    catch {
        return $false
    }
}

if (-not $NoBuild) {
    Write-Host 'Building...'
    foreach ($project in $serverProject, $botProject) {
        dotnet build $project -v q --nologo
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }

    # The sidecar the server runs the game on; the server finds it in the workspace's target\cards.
    Push-Location $root
    try {
        cargo build --profile cards -p cantrip-host --bin cantrip-native-host
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    finally { Pop-Location }

    if ($humans.Count -gt 0) {
        # A web client still being served holds the output it would be built into, and a build around it leaves a
        # mix of the two that never boots, so the old one goes first.
        Stop-Listener $webPort
        Stop-Listener $webHostPort
        Start-Sleep -Seconds 1

        dotnet build (Join-Path $webProject 'Cantrip.Client.Web.csproj') -f net10.0-browserwasm -v q --nologo
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
}

# --- The server ------------------------------------------------------------------------------------------------------

Write-Host ''
Write-Host "Player 1: $($players[1]) with $($decks[1])"
Write-Host "Player 2: $($players[2]) with $($decks[2])"
Write-Host "Seed $Seed, port $Port (art on $($Port + 1))$(if ($SimpleBot) { ', deterministic bots' } else { ', bots drawing at random' })"
Write-Host 'Rules: native (the cantrip-native-host sidecar; its log is in the server window)'
Write-Host ''

$serverArgs = "run --project `"$serverProject`" --no-build -- --format $Format --port $Port --players 2 --seed $Seed " +
    "--auto-start --deck `"$($decks[1])`" --deck `"$($decks[2])`"" + $(if ($Clock -gt 0) { " --clock $Clock" } else { '' })

# In a window of its own that stays open after the game, because the server log is where you find out why it ended.
Start-Process -FilePath 'cmd.exe' -ArgumentList "/k title Cantrip server & dotnet $serverArgs" -WorkingDirectory $root

Write-Host 'Waiting for the server to accept connections...'
$listening = $false
for ($i = 0; $i -lt 120 -and -not $listening; $i++) {
    $listening = Test-Listening $Port
    if (-not $listening) { Start-Sleep -Milliseconds 250 }
}

if (-not $listening) {
    Write-Host "The server never started listening on port $Port. Check its window."
    exit 1
}

# --- The web client --------------------------------------------------------------------------------------------------

if ($humans.Count -gt 0 -and -not (Test-Listening $webPort)) {
    # In a window of its own, like the server's: closing it stops serving the client.
    Start-Process -FilePath 'cmd.exe' `
        -ArgumentList "/k title Cantrip web client & dotnet run -f net10.0-browserwasm --no-build --urls http://localhost:$webHostPort" `
        -WorkingDirectory $webProject

    Write-Host 'Waiting for the web client to be served...'
    $served = $false
    for ($i = 0; $i -lt 240 -and -not $served; $i++) {
        try {
            Invoke-WebRequest -Uri "http://localhost:$webPort/" -UseBasicParsing -TimeoutSec 2 | Out-Null
            $served = $true
        }
        catch {
            Start-Sleep -Milliseconds 500
        }
    }

    if (-not $served) {
        Write-Host "The web client was never served on port $webPort. Check its window."
        exit 1
    }
}

# --- The players -----------------------------------------------------------------------------------------------------

# Each client asks for its seat, so the order they connect in cannot give a person the bot's deck.
$address = "ws://localhost:$Port/game"
$running = @()

# A tactical seat across from a person is played by that person's tab: the first person's, when there are two.
$hosted = @()
if ($humans.Count -gt 0) { $hosted = @(1, 2 | Where-Object { $players[$_] -eq 'Tactical' }) }

foreach ($seat in 1, 2) {
    $name = switch ($players[$seat]) { 'Human' { "Player $seat" } 'Llm' { "Model $seat" } default { "Bot $seat" } }

    if ($players[$seat] -eq 'Human') {
        $page = "http://localhost:$webPort/?server=$([Uri]::EscapeDataString($address))&seat=$seat&name=$([Uri]::EscapeDataString($name))"
        if ($seat -eq $humans[0] -and $hosted.Count -gt 0) { $page += "&bots=$($hosted -join ',')" }
        Start-Process $page
        Write-Host "Seat $seat is open in your browser: $page"
        continue
    }

    if ($hosted -contains $seat) {
        Write-Host "Seat $seat is the tactical bot, played by seat $($humans[0])'s tab."
        continue
    }

    $arguments = @('--connect', $address, '--seat', $seat, '--name', "`"$name`"")
    if ($players[$seat] -eq 'Llm') {
        $arguments += @('--llm', '--deck', "`"$($decks[$seat])`"")
        if ($Model) { $arguments += @('--model', $Model) }
    }
    elseif ($players[$seat] -eq 'Tactical') {
        $arguments += @('--tactical')
    }
    elseif (-not $SimpleBot) {
        # Each seat draws from a seed of its own off the game's.
        $arguments += @('--random', '--bot-seed', [int]((($Seed % 100000) * 10) + $seat))
    }

    $log = Join-Path $env:TEMP "cantrip-seat-$seat.log"
    $running += Start-Process -FilePath $bot -ArgumentList $arguments -WindowStyle Hidden -RedirectStandardOutput $log -PassThru
    $what = switch ($players[$seat]) { 'Llm' { 'a language model' } 'Tactical' { 'the tactical bot' } default { 'a bot' } }
    Write-Host "Seat $seat is $what without a window; its log is $log"
}

if ($humans.Count -gt 0) {
    Write-Host 'Game launched. Close a player tab to leave the game.'
    exit 0
}

# Nobody to show a browser to: the bots play it out here, and the result is what is printed.
Write-Host 'Playing...'
$running | ForEach-Object { $_.WaitForExit() }
foreach ($seat in 1, 2) { Get-Content (Join-Path $env:TEMP "cantrip-seat-$seat.log") }
