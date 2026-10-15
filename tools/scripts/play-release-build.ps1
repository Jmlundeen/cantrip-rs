# Plays a game in the shipping web client: builds it when it is out of date (Release, compiled ahead of time), starts a
# game server with the playtest deck (decks\demo\card-test) for both seats, puts a bot in seat 2 (dotnet\tools\Cantrip.Bot),
# serves the client on http://localhost:5100, and opens it in your browser as seat 1.
#
#   play-release-build.ps1              build if anything changed, then play
#   play-release-build.ps1 -NoBuild     play on what was built last
#   play-release-build.ps1 -Fast        a Release build without ahead-of-time compiling: quick to build, interpreted to run
#   play-release-build.ps1 -Seed 7      the shuffle and the bot's draws (default: a new one each game, printed)
#   play-release-build.ps1 -BuildOnly   build, and start nothing (discord-bot-release.ps1 builds the bot's client this way)
#
# The publish is incremental. With nothing changed it takes seconds. After a change, the ahead-of-time compiler
# compiles only the assemblies that changed - it keeps the rest in obj\Release - but the whole app is linked again every
# time, so a change to the client costs about four minutes; the first publish, or one after a package update, longer.
# -Fast is for seeing a change to the UI in a Release build without that wait, not for measuring: it runs interpreted,
# as the dev server does.
#
# Compressed with gzip only. Brotli is a quarter smaller again but took two minutes of every publish to recompress the
# relinked runtime; a real host would pay that once per release, not per change.
#
# The bot plays seat 2 without a window, at random from the seed; its log goes to %TEMP%\cantrip-seat-2.log. The
# server and the web host each get a window of their own; close them to stop.

[CmdletBinding()]
param(
    [uint64] $Seed = 0,
    [int] $Port = 7777,
    [int] $WebPort = 5100,
    [switch] $NoBuild,
    [switch] $Fast,
    [switch] $BuildOnly
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$deck = 'demo/card-test'
$webProject = Join-Path $root 'dotnet\src\Cantrip.Client.Web\Cantrip.Client.Web.csproj'
$hostProject = Join-Path $root 'dotnet\tools\Cantrip.WebHost\Cantrip.WebHost.csproj'
$serverProject = Join-Path $root 'dotnet\src\Cantrip.Server.Host\Cantrip.Server.Host.csproj'
$botProject = Join-Path $root 'dotnet\tools\Cantrip.Bot\Cantrip.Bot.csproj'
$bot = Join-Path $root 'dotnet\tools\Cantrip.Bot\bin\Debug\net8.0\cantrip-bot.exe'

# A fast build goes to a folder of its own, so it does not leave the ahead-of-time build to start over.
$published = Join-Path $root "dotnet\src\Cantrip.Client.Web\bin\Release\net10.0-browserwasm\$(if ($Fast) { 'publish-fast' } else { 'publish' })"
$wwwroot = Join-Path $published 'wwwroot'

if ($Seed -eq 0) {
    $Seed = [uint64](Get-Random -Minimum 1 -Maximum ([int]::MaxValue))
}

function Stop-Listener([int] $port) {
    Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue |
        ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
}

function Wait-Listening([int] $port, [string] $what) {
    for ($i = 0; $i -lt 240; $i++) {
        try {
            $probe = New-Object Net.Sockets.TcpClient
            $probe.Connect('127.0.0.1', $port)
            $probe.Close()
            return
        }
        catch {
            Start-Sleep -Milliseconds 250
        }
    }

    Write-Host "The $what never started listening on port $port. Check its window."
    exit 1
}

# --- The build -------------------------------------------------------------------------------------------------------

# Whatever is serving or playing from a last run holds files the build would replace.
Stop-Listener $WebPort
Stop-Listener $Port
Stop-Listener ($Port + 1)

if (-not $NoBuild) {
    $mode = if ($Fast) { @('-p:RunAOTCompilation=false', '-p:WasmShellMonoRuntimeExecutionMode=Interpreter') } else { @() }
    Write-Host "Publishing the web client (Release, $(if ($Fast) { 'interpreted' } else { 'ahead-of-time compiled' }))..."
    $started = Get-Date

    # Published afresh each time. A publish adds to what is there and never takes the last one's files away, and Uno
    # names the page's loader by looking for dotnet.*.js in this folder: with several there it can pick a stale one,
    # which loads the libraries it was published with and speaks an old protocol. The ahead-of-time work is kept in
    # obj, not here, so starting this folder over costs nothing.
    if (Test-Path $published) {
        Remove-Item -Recurse -Force $published
    }

    dotnet publish $webProject -c Release -f net10.0-browserwasm -o $published -v q --nologo '-p:PublishCompressionFormats=gzip' @mode
    if ($LASTEXITCODE -ne 0) { exit 1 }
    Write-Host ('Published in {0:mm\:ss}.' -f ((Get-Date) - $started))

    foreach ($project in $hostProject, $serverProject, $botProject) {
        dotnet build $project -v q --nologo
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
}

if (-not (Test-Path (Join-Path $wwwroot 'index.html'))) {
    Write-Host "Nothing published at $wwwroot yet. Run without -NoBuild."
    exit 1
}

if ($BuildOnly) {
    exit 0
}

# --- The server and the web client -----------------------------------------------------------------------------------

Write-Host ''
Write-Host "You and a bot, both playing $deck. Seed $Seed, port $Port."
Write-Host ''

$serverArgs = "run --project `"$serverProject`" --no-build -- --port $Port --players 2 --seed $Seed " +
    "--auto-start --deck `"$deck`" --deck `"$deck`""

# In windows of their own that stay open, because a log is where you find out why something ended.
Start-Process -FilePath 'cmd.exe' -ArgumentList "/k title Cantrip server & dotnet $serverArgs" -WorkingDirectory $root
Start-Process -FilePath 'cmd.exe' `
    -ArgumentList "/k title Cantrip web client & dotnet run --project `"$hostProject`" --no-build -- --root `"$wwwroot`" --port $WebPort" `
    -WorkingDirectory $root

Wait-Listening $Port 'game server'
Wait-Listening $WebPort 'web host'

# --- The players -----------------------------------------------------------------------------------------------------

$address = "ws://localhost:$Port/game"

# Seat 2 draws from a seed of its own off the game's, as play-game.ps1 has it.
$log = Join-Path $env:TEMP 'cantrip-seat-2.log'
$botArgs = @('--connect', $address, '--seat', 2, '--name', 'Bot', '--random', '--bot-seed', [int]((($Seed % 100000) * 10) + 2))
Start-Process -FilePath $bot -ArgumentList $botArgs -WindowStyle Hidden -RedirectStandardOutput $log

$page = "http://localhost:$WebPort/?server=$([Uri]::EscapeDataString($address))&seat=1&name=You"
Start-Process $page

Write-Host "Seat 1 is open in your browser: $page"
Write-Host "The bot's log is $log"
