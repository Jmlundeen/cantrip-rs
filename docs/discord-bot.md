# The Discord bot

The first slice of [discord-integration-plan.md](discord-integration-plan.md): a member with the right role hosts a
game from Discord, another member joins it, and both play it in their browser. Every seat plays
`decks/rw-aggro.txt`, so there is nothing to choose.

## How a game goes

1. A member with a host role (or Manage Server) runs `/cantrip host` in a channel.
2. The bot opens a table and posts a public message on it with **Join** and **Cancel**. The host gets an ephemeral
   reply, visible only to them, with an **Open in Cantrip** button. Its link carries their seat ticket, and the reply
   never shows the ticket as text.
3. Another member presses **Join** and gets the same kind of reply for seat 2. The message now names both players.
4. Each presses **Open in Cantrip**. The ticket seats them under their Discord name with RW Aggro, already ready; the
   cards are dealt once both are connected. The message says the game is in progress, and is deleted once the game is over -
   won, abandoned or cancelled - so the channel shows only games still to be joined or played.

Once both seats are taken, the button reads **My ticket** and hands a seated player their own reply again. The
ticket also takes back its seat if the client dies mid-game, so pressing **Open in Cantrip** again is how to get back
in.
The host or anyone with Manage Server can **Cancel**. A table that is not dealt within 30 minutes expires. An ended
table stays up for two minutes so both clients can see the result, then its port is freed. A player is in at most one
unfinished game at a time.

## Draft queues

A channel can be a standing draft queue: a "Draft Queues" category with a channel for each kind of draft - `#winston`,
`#grid`, `#booster`. A server manager sets one up in its channel:

```
/cantrip queue cube:LSVCube [format:Winston] [cards:90] [piles:3] [min-deck:40] [bots:0]
```

The bot posts the queue's message there and keeps it up to date: what it drafts, **Players at the table: 1/2**, how
many tables are getting ready and how many drafts are under way, and a **Join** button. Running the command again
changes what the queue drafts (the tables already forming keep what they had); `/cantrip unqueue` closes it.

1. A player presses **Join**. They get a seat at once - at the oldest of the queue's tables with a seat free, or a new
   table - and a private reply with **Open in Cantrip**, which opens the table in their browser. (Discord cannot open
   a browser straight from a button on a shared message, and a link on one cannot be anyone's own seat, so it takes
   this one private reply.) Pressing **Join** again gives a fresh link to the same seat.
2. The page shows the table: who is in each seat, and the seats still waiting for a player. **Leave Table** gives the
   seat up.
3. A player who pressed **Join** has 30 seconds to open their table from the link (pressing **Join** again gives
   another 30), and a player who closes the table while it forms has 30 seconds to come back; otherwise the seat goes
   back to the queue.
4. When every seat is taken and everyone has the table open, everyone gets **Ready!** and has 30 seconds to press it.
   Whoever has not by then loses their seat, which goes to the next player to press **Join**, and everyone left is
   asked again once the table is full and open again.
5. When everyone at a table is ready, the draft begins at that same table, and from then on it is a draft like one
   opened with `/cantrip draft`: kept in the data directory, taken up again after a restart, `/cantrip mydraft` for a
   link back to it.

Players join the oldest table with a free seat first. If two of a queue's tables are each waiting for players -
someone left, or was not ready - the newer table's players are moved to the older one: their page goes there by itself.
So a queue never has two tables each waiting for one player. Only players with their page open are moved; the
others keep their seat, and **Join** gives them a link to it.

### Bots in a queue

`bots:N` on any queue (`/cantrip queue cube:LSVCube format:Grid bots:1`) keeps a table's last N seats for bots, so
fewer players fill it - one player and a bot, for a test queue. The bots are at the table from the start, always ready,
and the message reads **Players at the table: 0/1, and a bot in the other seat**; the ready check waits only for the
players. Once the draft begins the bot runs them inside its own process (`BotSeats`): each a client over the table's
socket with a ticket of its own, as `dotnet\tools\Cantrip.Bot` plays a local draft's empty seat, drafting by the cube's pick
order, building, and playing its games as the tactical bot does (`docs/tactical-bot.md`) - across a booster pod's
game tables too. They sit down again when a restart
hosts the draft again. A table keeps at least one seat for a player.

A player has one seat at a time, across all the queues, and none while they are in a draft. The cube is named from the
cube lists (`docs/cube-lists`, `cubes/`), never given as a path. Closing a queue, or stopping the bot, tells the
players at its forming tables before the tables go; a table that has just begun its draft carries on.

### Grid queues

`/cantrip queue cube:LSVCube format:Grid [packs:18] [min-deck:40]` makes a channel a queue for a two-player grid draft:
eighteen packs of nine by default, each laid out in a grid, a row or a column each ([events.md](events.md)). The table
forms and the draft is played exactly as a Winston queue's.

### Booster queues

`/cantrip queue cube:LSVCube format:Booster [packs:3] [pack-size:15] [rounds:3] [min-deck:40]` makes a channel a queue
for a booster draft of a pod of eight. The table forms the same way, eight seats instead of two, and when all eight are
ready the pod drafts at that table: packs passed round with a pick clock, then deck building on a ten-minute clock, then
three Swiss rounds of best-of-three, each player with a 25-minute chess clock for the match - running while the game
waits on them, and losing them the match if it runs out. Each game is played at a table of its own; the page goes there by itself, comes back
to the pairings when the match is over, and shows the final standings at the end ([events.md](events.md)).
`/cantrip mydraft` gives a player a link back into their pod - to their game if one is on, otherwise the pod's table.
Pods are kept in the data directory and taken up again after a restart.

A pod's tables come from the same draft ports as every other draft table: its own, and one for every game being
played, so five of them while a round of a pod of eight is on. Give the bot more ports (`--draft-ports 7790-7829`, and
the tunnel or firewall to match) to run more than one pod at a time; a game waiting for a port opens as soon as one is
free.

Queues are kept in the data directory (`queues/{channel}.json`). Tables still forming are not: a restart empties them,
and their players press **Join** again. `DraftQueues` is the Discord-free half, tested in `DraftQueuesTests` with real
tables; the table itself is `QueueTable` ([events.md](events.md)).

## Drafts

Given a data directory (`--data PATH`, or `CANTRIP_DATA`), the bot also hosts two-player Winston drafts of a cube, kept
as events there ([events.md](events.md)):

1. A member with a host role runs `/cantrip draft`. The bot posts the draft with **Join draft** and **Cancel**.
2. Another member presses **Join draft**. That makes the event in the data directory, opens a table for it, and gives
   them the **Open in Cantrip** reply for seat 2. The button now reads **My ticket**; the first player presses it for
   theirs.
3. The draft starts when both are at the table. They draft, build decks and play, all at that table.

**My ticket** gives a fresh ticket at any point in the event, which works from any computer, and older ones keep
working, so a player who closed their browser, or moved to another machine, presses it and is back where they were.
`/cantrip mydraft` does the same without finding the message: it looks the player up in the data directory.
A draft nobody joins is dropped after 30 minutes. Once both players are in, the draft's own clocks see that it ends
(`DraftTiming`, `docs/events.md`): it is called off if a player has not taken their seat within ten minutes, and from
then on nothing waits on a player who has gone. A draft called off at its table is shown for a while and closed, as a
finished one is. The draft's message is deleted once it is over, finished or called off. Everything is in the data directory, so a restarted bot hosts every live draft again,
wherever it had got to, and finds its message to keep it up to date. A player is in at most one live draft at a time.

`DraftHost` is the Discord-free half, as `Matchmaker` is for games, and is tested in `DraftHostTests` with real tables.

### Players' pictures

Whenever a player joins a draft - `/cantrip draft`, **Join draft**, or **Join** on a queue - the bot takes their
Discord avatar (their avatar on this server, else their own, else Discord's stand-in), 128 pixels square, and keeps it
in the data directory as `portraits/discord-{id}.png`, written over the one before (`PortraitStore`). It is fetched
alongside the join, never ahead of it: a picture Discord will not give leaves the last one kept.

Every table of a draft, a queue or a pod tells the client each seat's picture (`TablePlayerView.Portrait`,
`portraits/{player}.png?v=...`, relative to the page), and the bot's site serves it there. The client draws it
beside the player's name in the queue's seat list, on their plate at a booster draft's table, and in their rail in a
game - and its stand-in for anyone without one. The `?v=` is when the picture was written, so a new avatar is a new
address and browsers may keep each one for good. A client hosted elsewhere (`--client`) is told of no pictures, since
the bot then serves no site to fetch them from. A plain `/cantrip host` game has no player ids, and no pictures.

## Opening the client from Discord

**Open in Cantrip** opens the web client, which the bot serves itself, in the player's browser and in their seat:

```
http://HOST:7779/?server=ws%3A%2F%2FHOST%3A7782%2Fgame&art=http%3A%2F%2FHOST%3A7779%2F#ticket=...
```

Nothing to install and nothing to register. The client knows nothing of any game: the link names the table in its
query, and carries the ticket after the `#`, which browsers never send to a server, so the ticket is in no request and
no log; the client takes it out of the address bar as the page loads. One site therefore serves every game at once,
and card art beside the client at `/art` (`ClientSite`). The bot serves the built client in `--client-root`, by
default this checkout's Release publish (`tools\play-release-build.cmd` makes it) or else its Debug build, sending each file
compressed to a browser that takes it. `--client` points the replies at the same client hosted anywhere else - a
static HTTPS host or a CDN - with `--art` for where its art is, and the bot then serves neither.
## One-time Discord setup

1. In the [Discord Developer Portal](https://discord.com/developers/applications), create an application. Its **Bot**
   page has the token: reset it, copy it, and keep it secret. No privileged intents are needed.
2. Invite the bot to your server with this URL, putting in the application ID from **General Information**:
   `https://discord.com/oauth2/authorize?client_id=APPLICATION_ID&scope=bot+applications.commands&permissions=3072`.
   3072 is View Channels plus Send Messages, which is all the bot uses: its replies to players are ephemeral, and it
   only edits and deletes its own messages.
3. Turn on Developer Mode in Discord (User Settings, Advanced) so you can right-click to copy the server ID and the ID
   of the role allowed to host.

## Running it

```
set CANTRIP_DISCORD_TOKEN=...
tools\discord-bot.cmd --host-role ROLE_ID
```

The launcher builds the bot into `.cantrip-run\discord` and runs it from there rather than from the project's own `bin`
folder. A bot left running holds the files it was started from; held in `bin`, it stopped the solution from building,
and `tools\test.cmd` with it, for as long as it ran.

| Option | Default | |
| --- | --- | --- |
| `--guild ID` | required; `discord-bot.cmd` fills in the Cantrip server | The server the bot serves. `/cantrip` is registered there, so it appears at once. |
| `--host-role ID` | none | May repeat. Without any, only members with Manage Server may host. |
| `--listen ADDRESS` | `127.0.0.1` | Where tables listen. |
| `--public-host HOST` | `localhost` | The host name in the address players are given. |
| `--ports FIRST-LAST` | `7780-7789` | One table to a port, so this is how many games can run at once. |
| `--web-port N` | `7779` | Where the bot serves the web client and its art. |
| `--client-root PATH` | the Release publish, else the Debug build | The built web client to serve (its `wwwroot`). |
| `--client URL` | the bot's own | The web client hosted elsewhere; the bot then serves none. |
| `--art URL` | the bot's own site | Where card art is, when `--client` is given. Every table tells its clients. |
| `--deck NAME` | `rw-aggro` | A decklist under `decks/`. |
| `--content PATH` | found above the executable | The content root. |
| `--public-url URL` | `CANTRIP_PUBLIC_URL`, else none | Everything behind the web port at this address, for an HTTPS tunnel or proxy: see below. |
| `--data PATH` | `CANTRIP_DATA`, else none | Where events and players are kept, which turns on `/cantrip draft`. Back it up. |
| `--cube NAME\|PATH` | `LSVCube` | The cube drafts draw from: a path, or a name in `docs/cube-lists`. |
| `--draft-cards N` | `90` | How many cards a draft draws from the cube. |
| `--draft-piles N` | `3` | How many piles a Winston draft lays out. |
| `--min-deck N` | `40` | The fewest cards a main deck may have. |
| `--draft-ports FIRST-LAST` | `7790-7799` | One draft table to a port, held from the first player joining to the game's end; shared by queues and `/cantrip draft`. |

The token is read from the environment only, so it never appears in a command line or a shell history.

`tools\discord-bot-release.cmd` puts a new build behind a running bot: it stops the bot and whatever `play-release-build.cmd` left
running, publishes the client in Release, and starts the bot again in a window of its own, at the address of the quick
tunnel it leaves running (or `-PublicUrl URL`). Other arguments go to the bot.

With the defaults, only players on the bot's machine can reach a table. For players elsewhere, listen on `0.0.0.0`,
give this machine's reachable address as `--public-host`, and open the port range and the web port in the firewall
and router. The
game traffic is plaintext WebSocket. That is fine on a LAN or VPN, but over the internet it wants a TLS proxy in front,
and the client accepts a `wss://` address.

## Behind a tunnel: one address, nothing open at home

With `--public-url https://cantrip.example.com` the bot puts everything behind its one web port: the client and its
art at that address, and every table at `wss://cantrip.example.com/table/{port}/game`, passed through to the table on
this machine (`TableProxy`; only the bot's own table ports). The tables keep listening on 127.0.0.1, so the only thing
to publish is the web port - which a Cloudflare Tunnel does from the inside, with no port opened on the router and the
machine's address never shown. The web port pings every player every 30 seconds, since Cloudflare closes a WebSocket
that has been silent for 100.

1. Install `cloudflared`: `winget install --id Cloudflare.cloudflared`.
2. To try it, `cloudflared tunnel --url http://localhost:7779` prints a random `https://....trycloudflare.com`; start
   the bot with that as `--public-url`. It changes every time, so it is for trying out only.
3. To keep it, with a domain on Cloudflare: Zero Trust > Networks > Tunnels > Create a tunnel (Cloudflared), run the
   Windows install command it gives in an administrator PowerShell (it installs a service that starts with Windows),
   and add a public hostname - say `cantrip` on your domain - with service type HTTP and URL `localhost:7779`. Start
   the bot with `--public-url https://cantrip.yourdomain`.

## How it is built

`dotnet/src/Cantrip.Discord` is the bot (`cantrip-discord`). It is two parts:

- `Matchmaker` knows nothing of Discord. It owns the games: the port pool, one `GameServer` per game with
  `RequireTickets` set, the two seat reservations, the one-game-per-player rule, cancellation, expiry and linger (in
  `SweepAsync`, which the host calls on a timer), and the result as the table reports it. `HostPolicy` says who may
  host. Both are tested in `dotnet/tests/Cantrip.Discord.Tests` with real tables on real sockets.
- `DiscordBot` is the adapter. It connects over the gateway with Discord.Net, asking for the Guilds intent only,
  registers `/cantrip host` in the one guild, and turns interactions into matchmaker calls and games into messages
  (`GameMessages`). It decides nothing itself. Handlers run off the gateway thread, and Discord's permissions for the
  command are not trusted: the member's roles and permissions are checked on every call.

`ClientSite` serves the web client and its art, and makes the link that opens a seat in it - and the server's status page
at `/status`, every event, forming table and game being played and the history of those played ([events.md](events.md#the-status-page)).

The seat itself is a `GameServer` ticket, described in [table-lifecycle.md](table-lifecycle.md#seat-tickets).

## Not yet

- **Identity.** A ticket proves only that someone was shown it, and it was shown only to its Discord account. OAuth
  login (stage 1 of the plan) will bind the seat to the account itself.
- **HTTPS.** The bot serves plain HTTP. Beyond a LAN, put a TLS proxy in front of the web port and the table ports, and
  give the players `https://` and `wss://` addresses.
- **Persistence.** Games hosted with `/cantrip host` live in the bot's memory: a restart ends them, and nothing is
  recorded. Drafts are kept in the data directory and survive a restart; a draft's game in progress is dealt again.
- **Choice.** One deck, two players, no sets.
- **Idle games.** A game in play has no time limit. If both clients sit connected and idle, it holds its port until
  someone cancels it. A client cut off without its socket closing still counts as connected until TCP gives up, and
  until then its ticket cannot take the seat back.
