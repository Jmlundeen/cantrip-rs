# The native host protocol

`cantrip-native-host` (`crates/cantrip-host`) is one process per server, holding any number of games.
`Cantrip.Server` talks to it over stdin and stdout, one JSON object per line, one reply per request, in order; stderr is
the host's log. Every payload inside is `protocol.rs`'s JSON (PascalCase, `$type` tags), so the C# side deserialises it
into `Cantrip.Protocol` types and forwards. The C# half is `dotnet/src/Cantrip.Server/Rules/NativeRulesHost.cs` over
`NativeHostProcess.cs` (which finds the binary: `CANTRIP_NATIVE_HOST`, else the workspace's `target/cards`), behind
`IRulesHost`, which is the whole of what `GameSession` asks of a rules engine. The rules themselves are the native
engine's: `docs/engine-design.md`.

Requests and replies (a reply is `{"Ok": {...}}` or `{"Rejected": {"Code": "...", "Detail": "..."}}`; codes are
`Cantrip.Protocol.ErrorCode` member names):

| Request | Reply |
| --- | --- |
| `{"Cmd":"NewGame","Seats":[{"Name":"Alice","Deck":["Island", ...],"Sideboard":["Negate", ...]}, ...],"Seed":123,"Shuffle":true,"FirstPlayer":1,"Chooser":null,"BasicLandsOutside":[]}` | `{"GameId":1,"Decklists":{"1":[ObjectView, ...], ...}}` - decks are card names, top first when unshuffled; `Sideboard` is the cards the seat owns outside the game (CR 100.4, 400.11a), which only that seat sees (`ZoneKindView.Sideboard`, a zone per seat in each view) and which a card that brings one into the game chooses among (North Wind Avatar); `BasicLandsOutside` makes it a Limited game, every sideboard holding any number of those basic lands; `FirstPlayer` fixes who goes first, `Chooser` asks that seat instead (CR 103.1); a printed `ObjectView` per deck card, in deck order, for the welcome. A card the engine has not ported in a deck is `Rejected` with `InvalidSetup`, naming it; one in a sideboard is left out of the game (KI-37) |
| `{"Cmd":"NewGame","Seats":[{"Name":"You","Deck":[]}, ...],"Seed":1,"Scenario":{"Name":"...","Text":"...","Driven":[1]}}` | the same: a table playing a `.cantrip` scenario, its decks, order and dice the scenario's; the `Driven` seats are answered by the table and every other by the script, and each step carries a `Scenario` object (`docs/scenarios-in-the-browser.md`) |
| `{"Cmd":"Start","GameId":1}` | a step |
| `{"Cmd":"Answer","GameId":1,"RequestId":7,"Seat":1,"Answer":{AnswerPayload}}` | a step, or `Rejected` with `IllegalAnswer`, `StaleReply`, `UnknownHandle` or `NotYourTurnToAnswer`, after which the request is still outstanding |
| `{"Cmd":"Concede","GameId":1,"Seat":1}` | a step |
| `{"Cmd":"View","GameId":1,"Seat":1}` | `{"View":PlayerView}` - the seat's current view, for resynchronising a client |
| `{"Cmd":"Oracle","Name":"Lightning Bolt"}` | `{"Card":ObjectView}` or `{"Card":null}` - a card as printed |
| `{"Cmd":"Close","GameId":1}` | `{}` |

A **step** is what one advance of the game produced:

```
{"Requests":[{"RequestId":7,"Seat":1,"Prompt":{PromptPayload},"MayCancel":false}, ...],
 "Views":{"1":{PlayerView},"2":{PlayerView}},
 "Events":{"1":[GameEventView, ...],"2":[...]},
 "Finished":false,"Outcome":"InProgress","WinningTeam":0}
```

`Requests` are every question outstanding, not only new ones; request ids are the host's and never reused within a game.
A run in progress - a spell or ability being announced and paid for (CR 601.2a-h), a resolution waiting on a question -
is told to nobody until it completes, since it may yet be taken back: its events wait, and every seat but the one
answering is sent the view it was last sent. An opponent sees no card leave the hand, no land tapped and no spell on the
stack until the spell is cast (CR 601.2i), and nothing at all of a cast that is given up.
Each event object carries a `"Sequence"` beside its `$type`, the host's game-wide event number, the same for every seat
told of it: it is what `EventsMessage.Sequences`, `AbilityResolvedEvent.First` and the log's line sharing run on. A
request may carry a `"Presentation"` (`PromptPresentationContext`); without one the C# side derives what it can from the
prompt and the seat's view. Seats are numbered from one. A game is resumed the way it always was: a fresh `NewGame` with
the same seed and decks, `Start`, and the recorded answers sent again in order, so the host is deterministic for a seed
and nothing in it is a snapshot.

Casting options include `CastPlanId`, an opaque positive identity within the current question's options. A
`TakeActionAnswer` echoes it with the option's kind, source and ability index. It selects the exact zone permission
and cost route whose terms the server displayed. The identity carries no engine object IDs and has no meaning in a
later question; the enclosing request ID still identifies the question. Zero retains the older answer form only when
its kind/source/index identifies one unambiguous route. Unknown IDs and IDs belonging to another option are rejected.
The C# MessagePack protocol is version 185; existing keys keep their numbers and these fields are appended.

A mana ability can be taken back while nothing hidden has been shown and nothing has followed from it. While the last
answer a seat gave activated one, and it and everything since only tapped, sacrificed from the battlefield, removed
counters, added mana or cost life (a land, Black Lotus, Lotus Petal, Pentad Prism, a Treasure), that seat's next priority
or pay-costs prompt offers an `UndoManaAbility` option on the permanent; taking it moves the game back to before the
activation through the game-log tree (`Game::goto`), each seat's handles back with it, and sends an
`EventsRetractedEvent` naming the sequences told of it. Never taken back: an ability whose cost discards (Lion's Eye
Diamond shows the hand), and one that triggered anything (Chromatic Star draws). Any other answer - casting a spell
included - a concession, or anything else happening clears what can be taken back. An undo is an answer like any other,
so a resumed game replays it.

The cards a host knows are the ones the native engine has ported (`docs/adding-cards.md`); a deck naming any other
is refused at the deal. The decks under `decks/demo` and `decks/spike` are kept to ported cards, and
`NativeHostGameTests` deals them through the real binary.
