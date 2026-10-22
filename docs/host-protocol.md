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
| `{"Cmd":"NewGame","Seats":[{"Name":"Alice","Deck":["Island", ...],"Sideboard":["Negate", ...]}, ...],"Seed":123,"Shuffle":true,"FirstPlayer":1,"Chooser":null,"BasicLandsOutside":[]}` | `{"GameId":1,"Decklists":{"1":[ObjectView, ...], ...}}` - decks are card names, top first when unshuffled; `Sideboard` is the cards the seat owns outside the game (CR 100.4, 400.11a), which only that seat sees (`ZoneKindView.Sideboard`, a zone per seat in each view) and which a card that brings one into the game chooses among (North Wind Avatar); `BasicLandsOutside` makes it a Limited game, every sideboard holding any number of those basic lands; `FirstPlayer` fixes who goes first, `Chooser` asks that seat instead (CR 103.1); a printed `ObjectView` per deck card, in deck order, for the welcome. A card the engine has not ported, in a deck or a sideboard, is `Rejected` with `InvalidSetup`, naming it, as is a deal of fewer than two seats or more than eight (`MAX_SEATS`) |
| `{"Cmd":"NewGame","Seats":[{"Name":"You","Deck":[]}, ...],"Seed":1,"Scenario":{"Name":"...","Text":"...","Driven":[1]}}` | the same: a table playing a `.cantrip` scenario, its decks, order and dice the scenario's; the `Driven` seats are answered by the table and every other by the script, and each step carries a `Scenario` object (`docs/scenarios-in-the-browser.md`) |
| `{"Cmd":"Start","GameId":1}` | a step |
| `{"Cmd":"Answer","GameId":1,"RequestId":7,"Seat":1,"Answer":{AnswerPayload}}` | a step, or `Rejected` with `IllegalAnswer`, `StaleReply`, `UnknownHandle` or `NotYourTurnToAnswer`, after which the request is still outstanding |
| `{"Cmd":"Check","GameId":1,"RequestId":7,"Seat":1,"Selected":[12, 15]}` | `{"Complete":true,"Addable":[18, 21]}` - what the rule of the outstanding choice of cards says of a partial selection (below); nothing advances. `Rejected` with `StaleReply`, `NotYourTurnToAnswer`, `UnknownHandle`, or `IllegalAnswer` for a question that picks no cards |
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

Each option of a priority, pay-costs or unless-pays prompt carries an `OptionId`, its place among the question's options
from 1, and a `TakeActionAnswer` echoes it. Two options may be alike in `Kind`, `Source` and `AbilityIndex` - a card two
permanents' permissions would each let be cast, or one cast for its mana cost and for an alternative cost another permanent
gives - and the id says which is taken; what makes each possible (the engine's `granted_by` and `paid_by`) stays engine-side,
so nothing about hidden objects goes to the client. An id the question does not have, or one of an option with another kind,
source or index, is refused with `IllegalAnswer`; an answer without one (0) is taken where only one option has its kind,
source and index, and refused where several do. The rows themselves say what tells them apart (`Description`: "Pay {E} rather
than its mana cost (Lurrus of the Dream-Den)", "Mystic Forge"). The C# protocol is version 194 for this.

A pay-costs prompt that `AnnouncesXOrKicks` (CR 601.2b) is answered with an `AnnounceAnswer` whose `Value` is the X, or the
times kicked, from 0 to the prompt's `MostAnnounced` - one answer however high it goes, so a page steps the value itself and
sends nothing until it is chosen. A value past `MostAnnounced` is refused with `IllegalAnswer`. Its `IncreaseX` and
`DecreaseX` options still step it one answer at a time, as the bots do. Version 202.

**Checking a selection.** A choice of cards (`ChooseCardsPromptMessage`) and a library search (`SearchLibraryPromptMessage`)
are judged by the question's selection rule (docs/engine-design.md, "Selection rules"), which the client cannot run: the
prompt says how many it takes for its caption (`Count`, `CountKind`: `Exactly`, `UpTo`, `AnyNumber`, `AsManyAsCan`), its
noun and its parts' words (`Kind`, `Words`: "with different names"), a running total a part keeps for the player
(`MaximumTotalManaValue` with each card's `ManaValues`, for "total mana value 6 or less": `Tally`, display alone), and the
rule's `Verdict` on none chosen yet - `Complete`, whether that is an answer, and `Addable`, the cards that may be picked first, and for a
question that takes one card at most `CompleteAlone`, the addable cards that are an answer picked alone (a one-card pick of
those is sent without asking; every other selection waits for the host's verdict as it is made). There is no eligible list beside
it. Every other selection is asked of the host as it changes: the client sends `CheckSelectionMessage { RequestId, Selected }`,
the server asks the host's `Check` and answers `SelectionVerdictMessage { RequestId, Selected, Verdict }`, to that seat
alone. The answer is still checked when it is given. A cost's tapping for power, collecting evidence and crafting, which
the same prompt carries, are checked the same way. `GameClient.Answer` settles any answer to such a question with the host
before sending it - checked, and if refused mended a card at a time from what may be added - so every client, the bots
included, answers what the rules take. A check the host refuses is answered with an `ErrorMessage` whose `RequestId` names the
request, so a client reacts only to the error of its own check. Version 203.

A mana ability can be taken back while nothing hidden has been shown and nothing has followed from it. While the last
answer a seat gave activated one, and it and everything since only tapped, sacrificed from the battlefield, removed
counters, added mana or cost life (a land, Black Lotus, Lotus Petal, Pentad Prism, a Treasure), that seat's next priority
or pay-costs prompt offers an `UndoManaAbility` option on the permanent; taking it moves the game back to before the
activation through the game-log tree (`Game::goto`), each seat's handles back with it, and sends an
`EventsRetractedEvent` naming the sequences told of it. Never taken back: an ability whose cost discards (Lion's Eye
Diamond shows the hand), and one that triggered anything (Chromatic Star draws). Any other answer - casting a spell
included - a concession, or anything else happening clears what can be taken back. An undo is an answer like any other,
so a resumed game replays it.

A draw the players agree to (`docs/rules-exceptions.md`, RX-4) is not a request of the host's: the offer and the answers to it
are the server's (`GameSession.OfferDraw`, `AnswerDraw`, over the C# protocol's `OfferDrawMessage`, `AnswerDrawMessage` and
`DrawOfferMessage`, version 201). Offering passes the offerer's priority, which the host is sent as any `PassAnswer`; once
every other player still in the game has accepted, the session ends the game as a draw from outside the host, as a match
clock running out does (`GameSession.Forfeit`), and nothing more is asked of it.

The cards a host knows are the ones the native engine has ported (`docs/adding-cards.md`); a deck naming any other
is refused at the deal. The decks under `decks/demo` and `decks/spike` are kept to ported cards, and
`NativeHostGameTests` deals them through the real binary.
