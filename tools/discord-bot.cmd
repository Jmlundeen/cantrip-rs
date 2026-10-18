@echo off
rem ---------------------------------------------------------------------------
rem Runs the Discord bot: /cantrip host opens a two-player table, Join takes
rem the other seat, and each player gets a private Open in Cantrip button.
rem
rem   set CANTRIP_DISCORD_TOKEN=...      the bot token; never pass it as an argument
rem   discord-bot.cmd [--guild ID] [--host-role ID]... [--listen ADDRESS]
rem                   [--public-host HOST] [--ports FIRST-LAST] [--deck NAME]
rem
rem With no --host-role, only members with Manage Server may host. Tables
rem listen on 127.0.0.1 unless --listen says otherwise; for players on other
rem machines, pass --listen 0.0.0.0 --public-host <this machine's address> and
rem open the port range (7780-7789 by default). The traffic is plaintext
rem WebSocket. docs\discord-bot.md has the one-time Discord setup.
rem
rem The bot is built into .cantrip-run\discord and run from there, not from the
rem project's own bin folder: a bot left running holds the files it was started
rem from, and held there it would stop the solution from building, and with it
rem tools\test.cmd, for as long as it ran.
rem ---------------------------------------------------------------------------

setlocal
set "ROOT=%~dp0.."

if not defined CANTRIP_DISCORD_TOKEN (
  echo Set CANTRIP_DISCORD_TOKEN to the bot's token first.
  exit /b 1
)

rem Our own server, unless --guild names another.
set "GUILD=--guild 1161743985849663528"
echo.%* | findstr /c:"--guild" >nul && set "GUILD="

set "RUN=%ROOT%\.cantrip-run\discord"
dotnet build "%ROOT%\dotnet\src\Cantrip.Discord\Cantrip.Discord.csproj" -v q --nologo -o "%RUN%" || exit /b 1

"%RUN%\cantrip-discord.exe" %GUILD% %*
endlocal
