# Events: drafts, decks and the data directory

An event is organized play that outlives a single game: a draft and the games played with what was drafted, and in
time leagues of drafts, sealed pools, tournaments. The first kind is a two-player **Winston draft** of a cube, played
end to end at one table: the players take their seats, draft, build decks from their pools, and play. The second is a
**booster draft** of a pod of eight: packs opened and passed, decks built, and three Swiss rounds of best-of-three,
each game at a table of its own. The third is a two-player **grid draft**, played at one table as a Winston draft is.

## The data directory

Events and players are kept as JSON in a directory the server is given - `--data PATH`, or the `CANTRIP_DATA`
environment variable - and never under AppData: this is the record of who played what, and it is the server's to back
up. `tools\draft.cmd` uses `.cantrip-data` at the top of the checkout (git ignores it) when given neither.

```
players/{player}.json             the player's name and every event they are in
events/{event}/event.json         the event (EventRecord)
events/{event}/draft.json         a draft's own state (WinstonDraft, GridDraft, BoosterDraft - whose Log is the draft log)
```

Every write goes to a temporary file and is moved over the old one, so a crash leaves the last good copy. While a draft
runs, `draft.json` is the authority on what everyone has taken - it is saved first on every pick, and the pools in
`event.json` are taken from it again when the event is opened - so a process stopped between the two saves loses
nothing. Ids are checked against a narrow pattern (`EventStore.ValidId`: no separators, no trailing dot, no device
names) before they touch a path. One directory per event keeps a
league of hundreds of drafts as hundreds of small directories, each readable, copyable and archivable on its own.

A **player** is an id - `discord-{user id}` for a Discord account, whatever a host names them otherwise - and their
record lists every event they are in, which is what lets them come back to it from anywhere.

## The event record

`dotnet/src/Cantrip.Server/Events/EventRecord.cs`. Deliberately general, so later kinds need no new shape:

- `Kind` (`winston-draft`, `grid-draft`, `booster-draft`), `ParentId` and `Children` - a league is an event whose children are its drafts.
- `Stage`: NotActive, Seating, Drafting, DeckBuilding, Playing (with `Round`), Complete, Cancelled.
- `Participants`, a list, each with their seat, their `Pool` (card ids, in the order they came), their `Deck` (the
  pool's cards in the main deck by their place in the pool, the basic lands added, submitted or not) and the hashes of
  their tickets.
- `Matches`, a list, each with its round, players, games (each game's seed, start, end and result) and winners.
- `Seed`, from which the shuffle of the cards, who picks first and every deal come, so anything can be followed
  exactly afterwards.
- `Draft`: the format, the cube, the number of piles and of cards.
- `HostData`: whatever the host that runs the event keeps with it, such as where the Discord message about it is.

## A Winston draft

`WinstonDraft`: the cards are shuffled face down into a stack and each pile (three, classically) starts with one card.
On a turn a player looks at the first pile with cards in it, and takes it - the pile starts again with a card from the
stack, and the turn passes - or passes it, which adds a card from the stack to it and moves on to the next pile. Passing
the last pile takes the top card of the stack instead, unseen. Once the stack is empty a passed pile gets nothing, and
a pile can only be passed while something is left after it, so every turn takes a card and the draft ends when every
card has been taken. Every take and pass names the decision it answers, so a click sent twice is refused rather than
applied to the next pile. Passing the last pile with one card left in the stack takes that card rather than adding it
to the pile.

Every seat sees the piles' sizes, the stack's, who is looking at which pile and what everyone did; only the seat
looking at a pile sees what is in it, and only its own pool. The draft keeps the cube's pick order as far as its cards
go, and the seat looking at a pile is told where its cards stand (`DraftMessage.PileRanks`) - that seat only, and that
pile only: nothing is ranked for a card a seat may not see.

Each turn is on a clock (`DraftTiming.WinstonTurn`, a minute by default): every pile looked at, to the one taken, on
the one clock. When it runs out the rest of the turn is played for its player (`WinstonDraft.TakeForClock`) - a pile of
two or more cards taken, a pile of one passed for the next, the last taken when it cannot be passed - and each such
decision is marked in the history and said "(time ran out)". Every seat is shown the active player's clock
(`DraftMessage.PickLeftMs`).

The cube is a list of card names in the decklist format (`docs/cube-lists/LSVCube.txt`; `# comments`, `2 Name`). A
name no loaded card has is left out, and the host says how many were. The number of cards drafted is drawn at random
from the rest.

## A grid draft

`GridDraft`: two players, and nine cards of the cube a pack - eighteen packs by default, 162 cards drawn at random.
Each pack is laid out face up in a three-by-three grid. The player who picks first takes a row or a column of it,
three cards; the other takes a row or a column of what is left, which may be three cards or - where it crosses the
first - two; the three or four cards left are discarded. Then the next grid is laid out, and the other player picks
first from it. Who picks first from the first grid is drawn at random.

Both seats see the whole grid and what each player took (`DraftMessage`, `Format` "grid": `Pile` is the grid's nine
places row by row, a place already taken an unknown card); only a seat's own pool is shown to it. A pick names its
line - rows 0 to 2 from the top, columns 3 to 5 from the left (`DraftPickMessage.Card`) - and the decision it answers,
so a click sent twice is refused. The draft keeps the cube's pick order as far as its cards go and tells both seats
where the grid's cards stand (`PileRanks`), which the bots' drafter takes the line worth most by
(`DraftPlay.ChooseLine`), after looking at the grid a moment so a person sees the pick made (`Cantrip.Bot
--draft-think MS`, 1200 by default; a Winston pile too). Every seat is told the line the last pick took
(`DraftMessage.LastLine`). `draft.json` keeps the grid, the cards still to be laid out, every pick (`History`) and what
was discarded.

Each pick is on a clock (`DraftTiming.GridPick`, 45 seconds by default), which every seat is shown
(`DraftMessage.PickLeftMs`). When it runs out the line is taken for its player (`GridDraft.ClockLine`): each card's
pick-order rank, from 1, squared, and the line whose average is lowest - so a poor card costs a line more than a good one
earns it. A place whose card is already gone counts as the worst card left on the grid, so a short line is not taken for
being short. The first among equals; with no pick order, the first line with a card in it.

## Nothing waits on a player who has gone

A Winston or grid draft is covered as a booster draft is, every stage on a clock (`DraftTiming`, the host's to set in
`DraftPod.Open`):

- **Seating**: ten minutes for every player to take their seat by their ticket, or the event is called off ("Bob did
  not take their seat in time."). A table formed from a queue seats everyone at once, so only a draft opened by hand
  waits on this.
- **Drafting**: the pick clocks above.
- **Deck building**: ten minutes, then every deck not submitted is submitted as it stands, made up to the minimum with
  basic lands, and the game is dealt. A deal that fails sends everyone back to building with a fresh ten minutes.
- **The game**: each player has a 25-minute match clock, running whenever the game waits on them (`ClockAtDeal`,
  kept in `MatchRecord.ClockMs`); a clock that runs out loses. A player not at the table when the game is dealt has
  three minutes to come to it, and one who leaves has five to come back, or forfeits (`ITableEvent.TakeForfeits`):
  the table ends the game with the other player winning, and the match is marked `Forfeit`.

The deadlines for seating and deck building are kept in the record (`Deadline`) and get a minute's grace after a
restart; the pick clocks and the absences start again from a restart.

## Pick orders

Each cube has a pick order beside it, `docs/cube-lists/<Cube>.picks.json`: every card of the cube, best first, by
consensus card quality. The booster draft's clock follows it and looks at nothing else. Every seat is told where each
card of its pack stands (`DraftMessage.PileRanks`), which is where a bot's drafter starts: what it then makes of its
pool, its colours and what it is passed is in [draft-bot.md](draft-bot.md). `python tools/maintenance/pick-order.py` writes the pick orders: 17lands' public pick data for Arena's Powered Cube
(a Bradley-Terry fit over the first eight picks of each pack), with CubeCobra's draft Elo, mapped onto the same scale,
for cards Arena's cube lacks. Every rating it fetches is kept in `docs/cube-lists/card-ratings.json` - gitignored,
scraped data - so a changed cube is ranked again from it and only cards never rated are looked up; `--refresh` fetches
everything again. A test fails when a cube list and its pick order disagree.

## A booster draft

`BoosterDraft`: the cube's loaded cards are shuffled and dealt into packs - three of fifteen for each of eight players,
by default - and everyone opens their first. A player takes a card from the pack in front of them and the rest goes on
to the next seat: up the seats (left) in packs 1 and 3, down (right) in pack 2. Nobody waits for the whole table: a pack
passed to a player still picking from another waits behind it, in the order it came, so a slow player has packs piling
up in front of them, and every seat sees how many (`DraftSeatView.Queued`). Only the next pack waits for everyone: it
is opened when every pack of the last one is empty. A pack's last card is nobody's choice: it
sits in front of whoever it reaches on a two-second clock, long enough to see, and the clock takes it without asking.

Every pick is on a clock started when the pack reaches the front of its player's queue: five seconds a card left in the
pack, so 75 seconds for a fresh pack of fifteen down to 10 for two cards. A clock that runs out first asks a player
who has the table open for the card they had chosen (`DraftMessage.PickDue`): the page answers with it, or with a pick
that takes nothing (`Take` false) when none was chosen, and the pack waits for the answer - two seconds at most
(`BoosterTiming.PickAnswer`). Without a card, an answer or a connection, the clock takes the pack's best card by the
cube's pick order for its player - at random only among cards the order ranks alike, or for a cube without one - so a
player who has dropped does not pass their neighbours what anyone at the table would have taken. A pick names how many picks its seat has made (`DraftPickMessage.Decision`) and the card by its place in
the pack, so a click sent twice is refused rather than taken from the next pack.

**Banlists.** A format leaves out of its cube the cards that do nothing in it (`Banlist`, kept in
`dotnet/src/Cantrip.Server/Events/banlist.json` by format name): Cogwork Librarian is banned in Winston and grid drafts, where
no pack is passed for it to go into. A banned card is dropped when the draft is dealt, and not counted among the cube's
playable cards; it is not reported missing.

**Draft abilities** (CR 905.2) are the draft's to carry out, and a card names the ones it prints in its JSON
(`draftAbilities`, `DraftAbility`). The first is Cogwork Librarian's (`additionalPickPutThisIntoPack`): a seat with one
in its pool may take an additional card with a pick, and the Librarian goes from its pool into that pack. The draft
learns which of the cube's cards have it when it is dealt (`BoosterDraft.AdditionalPicks`); a seat is told which cards
of its pool it may spend on the pack in front of it (`DraftMessage.AdditionalPicks`) and sends the extra cards with its
pick (`DraftPickMessage.Also`). The page offers a box beside Pick to use one, after which a second card can be chosen;
a bot spends one on any pick while the pack is still half full. "Draft this face up" exists to keep paper tables honest, and has
nothing to do where the server sees every pool. The other draft-matters cards of Conspiracy would join `DraftAbility`
the same way.

Every pick is kept, in order, in `draft.json`'s `Log`: the seat, the pack and pick number, who opened the pack, the
card, everything that was in the pack with it, whether nobody chose it (the clock, or the last card), what was put into
the pack for it (`Spent`, for an additional card), and when. That is
the draft log, for a viewer to come.

When the draft is over the players build (below), with ten minutes on the clock; a deck not submitted by then is
submitted as it stands, made up to the minimum with basic lands of its colours. Then the **Swiss rounds**
(`Swiss`): round one pairs across the table - seat 1 with seat 5 - and each later round pairs players of equal match
points, never two who have met while anyone else will do; an odd player out gets a bye, a 2-0 win. Three points a match
won and one a draw; ties are broken by opponents' match-win, game-win and opponents' game-win percentages, none below a
third (MTR appendix C). A round's pairings are shown for 15 seconds before the first round, and a minute after the last
match of each round ends; then every match's first game begins.

A match is best of three, and each of its games is played at **a table of its own** (`MatchGame`): its players are sent
there from the pod's table (`TableMoveMessage`), see the result for a moment when the game ends, and are sent on - to the
next game's table, where they sideboard first (three minutes, the same deck view, `DeckBuildingMessage.Title` saying
where the match stands), or back to the pod's table when the match is decided. Every page is a fresh one, so nothing of
one game is carried into the next. The pod's table shows the pairings and how each match stands (`PairingsMessage`)
whenever a player is there, and the final standings at the end.

Every match has a **match clock**, a chess clock for each player across all its games: 25 minutes each by default
(`BoosterTiming.MatchClock`). A player's clock runs while the game is waiting on them - priority, the mulligans, every
choice - and both run while the game waits on both. The table (`GameServer`) keeps it: it asks the event for each
seat's time as a game is dealt (`ITableEvent.ClockAtDeal`), charges the time to whoever the session is waiting on
(`GameSession.WaitingOn`), tells the seats whenever whose clock runs changes (`ClockMessage`), and gives what is left
back to the event to keep (`ClockUsed`, `MatchRecord.ClockMs`), so it carries into the next game and through a restart.
A player whose clock reaches zero loses the **whole match**, whatever the score: the game ends with their opponent the
winner (`GameSession.Forfeit`), and the match is theirs (`MatchRecord.TimedOut`). Sideboarding and the time between games
are on no one's clock.

Nothing waits for someone who is not there. The pick clock drafts for them, deck building's submits for them, and a
player who does not come to their match's table within three minutes, or is gone from it for five, forfeits the match
2-0 and is dropped from the rounds after - and if neither came, both lose. A player can also drop on purpose, from the
standings page (`DropMessage`, `BoosterPod.Drop`): before a round begins it is paired again without them, so nobody is
handed a match against someone who has gone; once it has begun, their match has to be over first - played out, or its
games conceded (`ConcedeMessage`, which loses the game being played, never the match). A player's pod ticket always takes them to
where they should be: to their game's table while their match is on, otherwise to the pod's. The match records keep
each game, its seed, its result, and each player's deck for the match (`MatchRecord.Decks`), so a game cut off by a
restart is dealt again from its seed, and every clock starts again from the restart.

`BoosterPod` is the pod's event and `PodHost` (`Cantrip.Server.Host`) hosts it: the pod's table, and a table for each
game it asks for (`TablesToOpen`), closed once its players have gone on (`TablesDone`). Where tables are is the host's
opener - loopback ports locally, the draft ports behind the public address for the Discord bot. The clocks are the
host's (`BoosterTiming`): a test gives them milliseconds.

## Deck building

When the last card is taken every player's whole pool is put in their main deck, less any sideboard the player made
in the client during the draft. Cards are dragged between columns and to or from the sideboard, or double-clicked
across ([web-client.md](web-client.md)); which column a card is in is the client's alone and not saved; basic lands (Plains, Island, Swamp, Mountain, Forest) are added as many as wanted and
are never drafted. Each change goes to the server as it is made and is saved, so a reload or another computer picks the
deck up where it was. Each carries a number the client counts up, which the deck view carries back once the server has
dealt with it, so a view from before the page's latest clicks (an echo, or the other player submitting) does not undo
them on screen. A deck is submitted once it has at least the event's minimum (forty by default) and can be taken
back to change until everyone has submitted; then the game is dealt with the decks built, the rest of each pool as the
sideboard.

## The table

`DraftPod` plays an event at one table, through `ITableEvent`: a `GameServer` given `Event` seats only the event's
players, by the event's tickets; shows them the draft and then the deck being built (`DraftMessage`,
`DeckBuildingMessage`, protocol 110); takes their picks and decks (`DraftPickMessage`, `BuildDeckMessage`); and deals
the game when everyone has submitted, recording its result in the event. A game that cannot be dealt - a pool naming a
card the content no longer has - is refused with the reason, and the event goes back to deck building with every deck
taken back, rather than being dealt the same failure again. `TableMessage` says the phase - Setup while
players take their seats, then Drafting, DeckBuilding and Playing - and names the event.

A player is never released from an event. Their seat waits for them however long they are gone, a browser's saved
membership never expires, and their ticket takes them back to it from any computer - taking the seat over from a
connection that still holds it, which is most likely the same player on the computer they just left; the old window is
told and closed, and its reconnect proof replaced so it cannot resume its way back. Tickets are kept only as SHA-256
hashes, the newest eight per player; a host makes a fresh one whenever a player asks. The command-line host makes one
for every player each time it hosts an event, and prints them or writes them to `--tickets` - fine on your own machine,
but they are credentials.

Stopping the table loses nothing that was saved: hosted again, the event goes on from the same decision of the draft,
the same deck, and the same tickets work. A game cut off by a restart is dealt again from its start, from the same seed:
the game itself is not saved.

## Tables formed from a queue

`QueueTable` is a table whose players are not known in advance. Its host - a queue - gives each player who joins a
seat and a ticket; the page shows the seats, taken or empty, and **Leave Table** (`LeaveMessage`). A player has 30
seconds from being given their seat to open the table, and 30 to come back to it having closed it, or the seat goes
back. When every seat is taken and every player has the table open, a ready check starts (`TableMessage.ReadyLeftMs`, `ReadyMessage`): whoever has not said ready within 30 seconds
loses their seat (`SeatReleasedMessage`, after which the page forgets the seat and says why). When everyone is ready
the event is made with the players in their seats, and the table becomes its table: the draft starts in the same page.
A host can also move a player to another table (`TableMoveMessage`), which the page follows by loading that table with
the ticket given. The forming part is only in memory; the event, once made, is kept like any other.

## Hosting one

- `tools\draft-winston-vs-bot.cmd`, `tools\draft-grid-vs-bot.cmd`, `tools\draft-booster-vs-bots.cmd` - one format each,
  you against bots on this machine (the booster pod is you and seven).
- `tools\draft.cmd` - any of them with every option: two tabs (`-Player2 Human`); `-Event ID` hosts one again.
  `-Format Booster` is a pod of eight: you and seven bots (`-Humans N` for more tabs; `-Packs`, `-PackSize`, `-Rounds`).
- `cantrip-server --data PATH --booster --cube LSVCube --player you=You --player b2=Bot ...` makes a booster draft and
  hosts it, the pod's table on `--port` and its games' tables on ports of their own; `Cantrip.Bot --event --ticket T`
  plays a seat of it from the draft to the final standings.
- `cantrip-server --data PATH --winston --cube LSVCube --cards 90 --piles 3 --player alice=Alice --player bob=Bob`
  makes an event and hosts it; `--event ID` hosts one already made. `--grid [--packs 18]` in place of `--winston`
  makes a grid draft (`tools\draft-grid-vs-bot.cmd`). It prints each player's ticket, or writes them to
  `--tickets FILE` as JSON. `Cantrip.Bot --ticket T` drafts, builds and plays a seat.
- The Discord bot, given a data directory: a queue channel's **Join**, or `/cantrip draft` ([discord-bot.md](discord-bot.md)).

## The status page

`/status` on the Discord bot's site (`StatusSite`, in `Cantrip.Server.Host`) shows anyone everything being played and
everything played: the events in the data directory, the queues' tables still forming and the games hosted on their own.
Active events are grouped by kind with their cube, players and where each stands; the completed ones are a history,
the last 24 hours by default, filtered by when they finished, kind, cube, player and outcome. Opening an event (its
address is `#event=ID`, so it can be linked) shows its draft - the pack or grid being drafted, which way, how many cards
each seat has taken and, in a booster draft, the packs waiting for each and their pick clocks - who has submitted a deck,
each round's pairings with the score, games and match clocks, and the Swiss standings with their tiebreakers.

`EventStatusBoard` (`Cantrip.Server`) is what it is allowed to say: only what every player at the table is shown, and
nothing any one of them is not - no pool, deck, card, ticket, seed, player id or host data, which a test holds it to. It
reads `event.json` and `draft.json`, which every draft and pod saves as it changes, keeping what it read and reading
an event again only when its files have changed. An event is being played only while the host runs it: the bot tells the board which events it hosts, and one the
directory still has as live that it does not - a draft played locally in the same directory (`CANTRIP_DATA` names it
for every tool), or one stopped for good - is listed with the events over, as not finished. A Winston or grid draft's pick clock is not saved, so the page shows
whose pick it is but not its clock. The JSON behind the page is at `/status/api/active`, `/status/api/history` and
`/status/api/events/{id}`.

## Not yet

- Leagues and sealed. A Winston or grid draft still goes to Complete after its first game. A booster draft plays its rounds;
  a league is an event whose children are its drafts (`Children`, `ParentId`).
- A draft-log viewer: the log is kept (`BoosterDraft.Log`) and nothing shows it yet.
- One table per port, so the Discord bot's ten draft ports are ten drafts at once - and a booster pod wants five of them
  while a round is played. Hundreds want many tables behind one listener, routed by table.
- The store reads every `event.json` to find the live ones, writes under one lock, and a player's record lists every
  event they were ever in. A league wants an index of live events, a lock per event and an archive.
- The Discord bot looks at each draft's stage every two seconds rather than being told.
- The game in progress is not saved, only re-dealt.
- No screen in the client lists a player's events. The record is there (`players/{id}.json`); the Discord bot uses it
  for `/cantrip mydraft` and to keep a player to one live draft.
- A ticket is a bearer credential; accounts (Discord OAuth, [discord-integration-plan.md](discord-integration-plan.md))
  will bind a seat to the player themselves.
