# Puts a new build behind the Discord bot: stops the bot and anything play-release-build.ps1 left running, publishes the web
# client in Release (play-release-build.ps1 -BuildOnly), and starts the bot again in a window of its own, serving that build.
#
#   discord-bot-release.ps1                 the bot comes back at the tunnel's address
#   discord-bot-release.ps1 -Fast           a Release build without ahead-of-time compiling (see play-release-build.ps1)
#   discord-bot-release.ps1 -PublicUrl URL  the bot's public address, when it is not the quick tunnel's
#
# The Cloudflare quick tunnel is left running, since a new one would get a new address: the bot comes back at the
# address in the tunnel's log. With no tunnel running, one is started and its address read from the log. Any other
# arguments go to the bot as they are. The token and the data directory come from CANTRIP_DISCORD_TOKEN and
# CANTRIP_DATA, as discord-bot.cmd has them.

[CmdletBinding()]
param(
    [string] $PublicUrl = '',
    [switch] $Fast,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $BotArgs = @()
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$cloudflared = Join-Path $env:LOCALAPPDATA 'cloudflared\cloudflared.exe'
$tunnelLog = Join-Path $env:LOCALAPPDATA 'cloudflared\quick-tunnel.log'

if (-not $env:CANTRIP_DISCORD_TOKEN) {
    Write-Host "Set CANTRIP_DISCORD_TOKEN to the bot's token first."
    exit 1
}

function Get-Windows([string] $title) {
    Get-CimInstance Win32_Process -Filter "Name = 'cmd.exe'" |
        Where-Object { $_.CommandLine -like "*title $title &*" }
}

function Stop-Tree([int] $id) {
    taskkill /PID $id /T /F 2>&1 | Out-Null
}

function Stop-Listener([int] $port) {
    Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue |
        ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
}

# --- Down ------------------------------------------------------------------------------------------------------------

Write-Host 'Stopping the Discord bot and the release server...'

foreach ($title in 'Cantrip Discord bot', 'Cantrip server', 'Cantrip web client') {
    Get-Windows $title | ForEach-Object { Stop-Tree $_.ProcessId }
}

# Whatever was started some other way: the bot, and play-release-build.ps1's server, web host and seat-2 bot.
Get-Process cantrip-discord -ErrorAction SilentlyContinue | Stop-Process -Force
Get-CimInstance Win32_Process -Filter "Name = 'cantrip-bot.exe'" |
    Where-Object { $_.CommandLine -like '*ws://localhost:7777/*' } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
foreach ($port in 7779, 7777, 7778, 5100) {
    Stop-Listener $port
}

# --- The build -------------------------------------------------------------------------------------------------------

$buildArgs = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'play-release-build.ps1'), '-BuildOnly')
if ($Fast) { $buildArgs += '-Fast' }
& powershell @buildArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host 'The build failed; the bot was left stopped.'
    exit 1
}

# The bot is run with dotnet run, which would build it in its window; built here, a failure stops this script instead.
dotnet build (Join-Path $root 'dotnet\src\Cantrip.Discord\Cantrip.Discord.csproj') -v q --nologo
if ($LASTEXITCODE -ne 0) {
    Write-Host 'The bot failed to build; it was left stopped.'
    exit 1
}

# --- The address -----------------------------------------------------------------------------------------------------

function Read-TunnelUrl {
    if (-not (Test-Path $tunnelLog)) { return $null }
    $found = Select-String -Path $tunnelLog -Pattern 'https://[a-z0-9-]+\.trycloudflare\.com' -AllMatches |
        Select-Object -Last 1
    if ($found) { return $found.Matches[-1].Value }
    return $null
}

$custom = $BotArgs -contains '--public-url'
if (-not $PublicUrl -and -not $custom -and -not $env:CANTRIP_PUBLIC_URL) {
    if (-not (Get-Windows 'Cantrip tunnel')) {
        if (-not (Test-Path $cloudflared)) {
            Write-Host "No tunnel is running and cloudflared is not at $cloudflared. Pass -PublicUrl."
            exit 1
        }

        Write-Host 'Starting a quick tunnel...'
        if (Test-Path $tunnelLog) { Remove-Item -Force $tunnelLog }
        Start-Process -FilePath 'cmd.exe' -WorkingDirectory $root -ArgumentList (
            "/k title Cantrip tunnel & `"$cloudflared`" tunnel --no-autoupdate --url http://localhost:7779 " +
            "--logfile `"$tunnelLog`"")

        for ($i = 0; $i -lt 120 -and -not (Read-TunnelUrl); $i++) {
            Start-Sleep -Milliseconds 250
        }
    }

    $PublicUrl = Read-TunnelUrl
    if (-not $PublicUrl) {
        Write-Host "No tunnel address in $tunnelLog. Pass -PublicUrl."
        exit 1
    }
}

# --- Up --------------------------------------------------------------------------------------------------------------

$arguments = @($BotArgs)
if ($PublicUrl) { $arguments = @('--public-url', $PublicUrl) + $arguments }

Write-Host ''
Write-Host "Starting the Discord bot$(if ($PublicUrl) { " at $PublicUrl" })..."
Start-Process -FilePath 'cmd.exe' -WorkingDirectory $root `
    -ArgumentList "/k title Cantrip Discord bot & tools\discord-bot.cmd $($arguments -join ' ')"
