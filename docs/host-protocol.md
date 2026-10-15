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
| `{"Cmd":"NewGame","Seats":[{"Name":"Alice","Deck":["Island", ...]}, ...],"Seed":123,"Shuffle":true,"FirstPlayer":1,"Chooser":null}` | `{"GameId":1,"Decklists":{"1":[ObjectView, ...], ...}}` - decks are card names, top first when unshuffled; `FirstPlayer` fixes who goes first, `Chooser` asks that seat instead (CR 103.1); a printed `ObjectView` per deck card, in deck order, for the welcome. A card the engine has not ported is `Rejected` with `InvalidSetup`, naming it |
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
Each event object carries a `"Sequence"` beside its `$type`, the host's game-wide event number, the same for every seat
told of it: it is what `EventsMessage.Sequences`, `AbilityResolvedEvent.First` and the log's line sharing run on. A
request may carry a `"Presentation"` (`PromptPresentationContext`); without one the C# side derives what it can from the
prompt and the seat's view. Seats are numbered from one. A game is resumed the way it always was: a fresh `NewGame` with
the same seed and decks, `Start`, and the recorded answers sent again in order, so the host is deterministic for a seed
and nothing in it is a snapshot.

A mana ability can be taken back. While the last answer a seat gave activated one, and it and everything since did only
what mana abilities do (tap, add mana, cost life), that seat's next priority or pay-costs prompt offers an
`UndoManaAbility` option on the permanent; taking it moves the game back to before the activation through the game-log
tree (`Game::goto`) and sends an `EventsRetractedEvent` naming the sequences told of it. Any other answer, a
concession, or anything else happening (a trigger, a card seen, a spell finished casting) clears what can be taken
back. An undo is an answer like any other, so a resumed game replays it.

The cards a host knows are the ones the native engine has ported (`docs/adding-cards.md`); a deck naming any other
is refused at the deal. The decks under `decks/demo` and `decks/spike` are kept to ported cards, and
`NativeHostGameTests` deals them through the real binary.
