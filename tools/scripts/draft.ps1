# Launches a local Winston or grid draft: a server hosting the draft as an event, the web client served beside it, a browser tab
# for each person, and a bot for the other seat. The draft, the deck building and the game the decks are built for are
# all played at the one table.
#
#   draft.ps1                               you against a bot, 90 cards of LSVCube in 3 piles
#   draft.ps1 -Cube PowerLSV -Cards 60      another cube, fewer cards
#   draft.ps1 -Player2 Human                two tabs, you draft both seats
#   draft.ps1 -Event winston-draft-...      host an event already made, wherever it had got to
#   draft.ps1 -Format Booster               a pod of eight: you and seven bots draft three packs of fifteen, then play
#                                           three Swiss rounds, each game at a table of its own
#   draft.ps1 -Format Booster -Humans 2     two tabs and six bots
#   draft.ps1 -Format Grid                  a grid draft against a bot: eighteen packs of nine, each laid out in a
#                                           grid, a row or a column each
#
# The event is kept in a data directory - -Data, else CANTRIP_DATA, else .cantrip-data at the top of the checkout,
# which git ignores - with every pick and every change to a deck saved as it is made. Stop the server and run this again
# with -Event and the draft goes on from where it was; a tab reloaded, or opened again from its link, finds its seat.
#
# A person's seat is a tab of your default browser at http://localhost:5000, seated by a ticket after the '#' in its
# address; a bot's is dotnet\tools\Cantrip.Bot without a window, its log in %TEMP%\cantrip-draft-seat-N.log. The bot takes any
# pile of two or more cards and builds its two main colours with seventeen lands, and plays its games as the tactical
# bot (TacticalPlay).
#
#   -Cube NAME|PATH  a cube: a list of card names, by path or by name in docs\cube-lists (default LSVCube)
#   -Cards N         how many cards are drafted, drawn at random from the cube (default 90)
#   -Piles N         how many piles (default 3)
#   -MinDeck N       the fewest cards a main deck may have (default 40)
#   -Player2 Bot|Human  (Winston and grid)
#   -Format Winston|Booster|Grid
#   -Packs N, -PackSize N, -Rounds N        a booster draft's packs, their size and its Swiss rounds (3, 15, 3);
#                                           a grid draft's -Packs is how many grids it lays out (18)
#   -Seats N         how many players a booster draft seats (default 8)
#   -Humans N        how many of a booster draft's seats are tabs of your browser; the rest are bots (default 1)
#   -Data PATH       where events and players are kept
#   -Event ID        host this event rather than making a new one
#   -Port N          the port the server listens on (default 7777); its card art is served on the next one
#   -NoBuild         skip the build

[CmdletBinding()]
param(
    [string] $Cube = 'LSVCube',
    [int] $Cards = 90,
    [int] $Piles = 3,
    [int] $MinDeck = 40,
    [ValidateSet('Human', 'Bot')][string] $Player2 = 'Bot',
    [ValidateSet('Winston', 'Booster', 'Grid')][string] $Format = 'Winston',
    [int] $Packs = 0,
    [int] $PackSize = 15,
    [int] $Rounds = 3,
    [int] $Seats = 8,
    [int] $Humans = 1,
    [string] $Data,
    [string] $Event,
    [int] $Port = 7777,
    [switch] $NoBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$webProject = Join-Path $root 'dotnet\src\Cantrip.Client.Web'
$serverProject = Join-Path $root 'dotnet\src\Cantrip.Server.Host\Cantrip.Server.Host.csproj'
$botProject = Join-Path $root 'dotnet\tools\Cantrip.Bot\Cantrip.Bot.csproj'
$bot = Join-Path $root 'dotnet\tools\Cantrip.Bot\bin\Debug\net8.0\cantrip-bot.exe'
$webPort = 5000
$webHostPort = 5286

if (-not $Data) { $Data = if ($env:CANTRIP_DATA) { $env:CANTRIP_DATA } else { Join-Path $root '.cantrip-data' } }
$Data = [IO.Path]::GetFullPath($Data)

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

    Stop-Listener $webPort
    Stop-Listener $webHostPort
    Start-Sleep -Seconds 1
    dotnet build (Join-Path $webProject 'Cantrip.Client.Web.csproj') -f net10.0-browserwasm -v q --nologo
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

# --- The server ------------------------------------------------------------------------------------------------------

$tickets = Join-Path $env:TEMP "cantrip-draft-tickets-$Port.json"
Remove-Item -LiteralPath $tickets -ErrorAction SilentlyContinue

$serverArgs = "run --project `"$serverProject`" --no-build -- --port $Port --data `"$Data`" --tickets `"$tickets`""
if ($Event) {
    $serverArgs += " --event $Event"
}
elseif ($Format -eq 'Booster') {
    if ($Packs -le 0) { $Packs = 3 }
    $serverArgs += " --booster --cube `"$Cube`" --packs $Packs --pack-size $PackSize --rounds $Rounds --min-deck $MinDeck --player you=You"
    for ($seat = 2; $seat -le $Seats; $seat++) {
        $player = if ($seat -le $Humans) { " --player player$seat=`"Player $seat`"" } else { " --player bot$seat=`"Bot $seat`"" }
        $serverArgs += $player
    }
}
elseif ($Format -eq 'Grid') {
    if ($Packs -le 0) { $Packs = 18 }
    $serverArgs += " --grid --cube `"$Cube`" --packs $Packs --min-deck $MinDeck --player you=You --player opponent=`"$(if ($Player2 -eq 'Bot') { 'Bot' } else { 'Player 2' })`""
}
else {
    $serverArgs += " --winston --cube `"$Cube`" --cards $Cards --piles $Piles --min-deck $MinDeck --player you=You --player opponent=`"$(if ($Player2 -eq 'Bot') { 'Bot' } else { 'Player 2' })`""
}

Write-Host "Events are kept in $Data"
Start-Process -FilePath 'cmd.exe' -ArgumentList "/k title Cantrip draft server & dotnet $serverArgs" -WorkingDirectory $root

Write-Host 'Waiting for the server to make its tickets...'
for ($i = 0; $i -lt 240 -and -not (Test-Path -LiteralPath $tickets); $i++) { Start-Sleep -Milliseconds 250 }
for ($i = 0; $i -lt 120 -and -not (Test-Listening $Port); $i++) { Start-Sleep -Milliseconds 250 }
if (-not (Test-Path -LiteralPath $tickets) -or -not (Test-Listening $Port)) {
    Write-Host "The server never came up on port $Port. Check its window."
    exit 1
}

$made = Get-Content -LiteralPath $tickets -Raw | ConvertFrom-Json
Remove-Item -LiteralPath $tickets
Write-Host "Event $($made.eventId). Host it again with: tools\draft.cmd -Event $($made.eventId)"

# --- The web client --------------------------------------------------------------------------------------------------

if (-not (Test-Listening $webPort)) {
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

$address = [Uri]::EscapeDataString("ws://localhost:$Port/game")
$players = @($made.tickets.PSObject.Properties | ForEach-Object { $_.Value } | Sort-Object seat)
$booster = $Format -eq 'Booster' -or ($Event -and $Event -like 'booster-draft*')
foreach ($player in $players) {
    $isBot = if ($booster) { $player.name -like 'Bot *' } else { $player.seat -eq 2 -and $Player2 -eq 'Bot' }
    if ($isBot) {
        $log = Join-Path $env:TEMP "cantrip-draft-seat-$($player.seat).log"
        # It drafts by the cube's pick order and then plays its games as the tactical bot does: an opponent, not a seat-filler.
        $botArgs = @('--connect', "ws://localhost:$Port/game", '--ticket', $player.ticket, '--name', "`"$($player.name)`"", '--tactical')
        if ($booster) { $botArgs += '--event' }
        Start-Process -FilePath $bot -ArgumentList $botArgs -WindowStyle Hidden -RedirectStandardOutput $log | Out-Null
        Write-Host "Seat $($player.seat) ($($player.name)) is a bot; its log is $log"
    }
    else {
        Start-Process "http://localhost:$webPort/?server=$address#ticket=$($player.ticket)"
        Write-Host "Seat $($player.seat) ($($player.name)) is open in your browser."
    }
}
