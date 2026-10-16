# The rules engine: design

Status 2026-10-15. The engine is Rust, the Cargo workspace at the repository root (`Cargo.toml`, `crates/`). Every card
in content is ported (841 cards); 1253 of the 1257 scenarios in `scenarios/` pass and none fails (the other four name
cards that are not in content). It answers in about 5 us per decision, against the old C# engine's 261 us. The server
plays every game on it, through `IRulesHost`, and the C# engine is gone. Next is step 6 of the plan: Python bindings
and the AI work.

Card behavior is per-card code. Everything above the rules is C# under `dotnet/`: the server, events, the Discord
bot, the web client and the protocol. They reach the engine through `IRulesHost`, implemented by the
`cantrip-native-host` sidecar.

The reason for the rewrite is total compute. The AI work in [ai-player-design.md](ai-player-design.md) will run
millions of simulated games, most of them inside search. A rewrite is paid once; a slower engine is paid on every run.

## Goals

- **Throughput.** Steps per second, legal-move enumeration, and above all the cost of reaching another position, so
  that search can branch anywhere.
- **Branch anywhere.** Search can move to any decision, including a prompt in the middle of a resolution.
- **Per-card code.** A card's own conditions, values and sequencing stay in that card's code (CLAUDE.md, "bricks,
  and what is built with them"). Nothing about one card goes into a central switch.
- **Two ways in.** One is the server, through `IRulesHost`. The other is Python, in-process, for training: batches of
  games, step, branch, encode.

Not goals: compiling cards at runtime (cards are compiled with the engine); running the rules on a GPU (see the AI
design: rules diverge per card and the state has no fixed size); parsing Oracle text.

## What the C# engine cost

A read of the C# engine (2026-10-14) found five costs that came from its design, not its language, so a port that
kept the design would have kept them:

| Cost | Where it was |
| --- | --- |
| A full deep copy of the game on every batch of zone changes | `ZoneChanger.cs` (`StateGraph.Clone`, reflection-driven) |
| Every card's characteristics recomputed after almost any change, each recompute applying the layers twice to verify | `LayerEngine.cs` (`CharacteristicsCache`), invalidated from ~157 places |
| A paused game could not be copied: a prompt was a suspended `await` held in a delegate, so branching meant replaying the game from the start | `GameTask`, `GameEngine.Ask` |
| Strings at runtime: effect verbs and keywords were strings, `CardFilter.Parse` ran when an effect resolved | Engine, CardLibrary |
| State was a tree of reference objects in `Dictionary` and `List` | `GameState.cs` |

What it did well, and this engine keeps: full determinism (a seeded RNG in the state, shuffles and rolls recorded),
and self-play on every core in one process.

## Architecture

### Crates

| Crate | Holds |
| --- | --- |
| `cantrip-state` | the game state: the arena, its journalled write API, IDs, zones, the decision record, the generated subtypes. Plain data, no behavior. |
| `cantrip-rules` | the bricks: turn structure, priority, stack, targeting, layers, replacement ordering, triggers, state-based actions, combat, mana, costs, the game-log tree. |
| `cantrip-api` | the public surface card code is written against: `Game` and `Query`, card definitions, the event history, the request kinds. The only engine crate card code may depend on. |
| `cantrip-mechanics` | shared mechanics: parameterised recipes printed cards share, and the shared history questions. Depends on `cantrip-api` only. |
| `cantrip-cards` | the cards, one file each in `src/cards/`; its build script registers every file. Depends on `cantrip-api` and `cantrip-mechanics` only. |
| `cantrip-oracle` | characteristics, generated from `content/*/cards/*.json` by its build script (below) |
| `cantrip-engine` | the card table and the engine's entry points, tying the rules to the cards |
| `cantrip-host` | the `cantrip-native-host` sidecar behind `IRulesHost`: JSON lines over stdio ([host-protocol.md](host-protocol.md)) |
| `cantrip-script` | the `.cantrip` scenario runner, and `corpus`, which runs the corpus and says how each went |
| `cantrip-bench` | the benchmark |

The `cards` profile (release code, incremental) is the card loop's: an edit to one card rebuilds in about a second
([adding-cards.md](adding-cards.md)). Python bindings (`cantrip-py`, PyO3) are step 6.

### Layers: core, shared mechanics, cards

1. **Core (`cantrip-rules`): rules authority.** The Comprehensive Rules' procedures and the state they act on:
   turn structure and priority; the stack and the casting, activating and resolving procedures (601, 602, 608); the
   cost-payment framework, targeting and legality; zones, zone changes and new-object identity (400.7) with
   last-known information; the layer system with timestamps and dependencies (613); application of replacement and
   prevention effects (614-616); trigger collection and ordering (603); state-based actions (704); combat; mana;
   game end; hidden information; the game log.
2. **Shared mechanics (`cantrip-mechanics`): parameterised recipes.** Code that is the same across cards and differs
   only in their data: keyword actions (scry, surveil, mill, investigate, explore), keyword abilities that are
   ordinary abilities with parameters (cycling N, ward N), common effect shapes, and history questions more than one
   card asks. Admission: name a second printed card that would use it exactly as written.
3. **Cards (`cantrip-cards`): everything one card does alone.** Grist's repeating +1, Gollum's parity choice. Built
   from the public API and the shared mechanics, nothing else.

**The test for the core is "does it need rules authority", not "is it in the Comprehensive Rules".** Keyword
actions (701) and most keyword abilities (702) are in the rules, but most need nothing beyond the public API, so they
are shared mechanics. A keyword is core only when it changes a core procedure: trample changes damage assignment,
first strike adds a combat damage step, split second changes when players may act. Shared use is not a reason for
core ownership; needing rules authority is.

**The boundary is enforced by the compiler.** `cantrip-mechanics` and `cantrip-cards` depend on `cantrip-api` alone,
and Rust cannot reach another crate's private items, so card code cannot name the rules crate's internals.

### State: one arena, every write journalled

Search needs two things from the state: reaching another position must be cheap, and a change must be small enough to
store as a delta. The layout is chosen for those two and nothing else. It is not an entity-component system: a game
has a few hundred objects at most, and rules code touches a handful of them at a time in specific ways, so ECS's
strength, sweeping many similar entities every frame, buys nothing here.

- The state lives in a few contiguous buffers of plain data: object rows, zone lists, continuous effects, triggers,
  the agenda, the RNG. Everything refers to everything else by ID or offset.
- **Objects are IDs.** An ID indexes a row holding the object's card (an index into the immutable card table),
  counters, damage, status, controller, owner, zone, attachments and timestamps. An object that changes zones becomes
  a new object (CR 400.7) with a new ID; its last-known information is a frozen row.
- **Zones are ordered lists of IDs.** Continuous effects, pending and delayed triggers and replacement effects are
  compact records that name objects by ID and card behavior by function index.
- **Derived data is not state.** A cache of something computed from the state (the list of static-ability sources)
  is keyed on `GameState::epoch`, which moves on every write and every undo and never goes back. It needs no
  journalling and is ignored by equality and hashing.

The card table is shared and immutable, so it is never copied. A whole position copies in about 1 us
(`clone_with_journal` is the deep copy, the journal included).

**Every write goes through one journalled API.** Nothing mutates the state directly. Each write (move this object,
add this counter, tap this, push this frame) records an undo entry. The journal serves three purposes at once:

- **Moving through the game tree.** The journal's entries are what the game log's nodes hold (next section), so
  any position is reached from any other by undoing and redoing them.
- **Checkpoint and rollback** for anything that tries a move and takes it back.
- **Rollback for card code** (below): undoing an effect's mutations before it re-runs uses the same journal.

The RNG lives in the state. Every random event goes through it, so a game replays exactly.

### The game log is the search tree

The game is a tree of nodes, one per answer (`cantrip-rules/src/tree.rs`). The real game is one path from the root;
a search branch is a sibling path that shares everything above it.

- **A node holds the state writes its answer made**, each with redo and undo data (the journal above).
  `Game::goto` moves the working state from one node to another as a checkout does: undo up to the common ancestor,
  redo down to the target. An answer already given at a node is reused rather than played again. A move costs
  4-9 us. **Search moves through the tree, never by copying the game.**
- **A working state is the quick reference.** The live game, and each search thread, holds one fully built state and
  moves it through the tree. A value lookup is a read of the working state, never a rebuild.
- `CANTRIP_VERIFY=1` turns on the engine's own checks while it plays: every answer is round-tripped through the tree
  and the position it lands on is compared.

**The event history is the record the rules ask about the past** (`cantrip_api::history`, read with `q.history()`).
Every rules event since the game began is a `Record` (serial, turn, step, the active player, its cause, and the
`Event`), recorded after its triggers are collected and its replacements applied. Records name objects and copy
nothing out of them: what an object was is read from its row, frozen as it left, or its last-known snapshot. It is
also the last-known-information record for one-off mechanics. A question about the past ("life gained this turn",
"the first card drawn in its controller's draw step") is a query over it: written in the card's file when one card
asks it, moved to `cantrip_mechanics::history` when a second printed card asks it in the same words. The core only
records events; no per-turn counter answers a history question. One static ability (Shardmage's Rescue) keeps a
`controlled_since` field because layers cannot read the history.

- **Causes are recorded.** Each event names what produced it, which serves explanations, debugging and the AI's
  features.
- **Visibility is derived, not stored.** Who knows what follows from the state and the log, provided every action
  that gives a player knowledge is an event: a reveal, a look at cards (`seen_by` on a face-down card), a search.
- **Narration, replay and training data are views of it.** "State before a decision, the options, the choice, the
  outcome" comes out of the tree with no extra instrumentation.

**Hidden information is the bot's problem, not the engine's.** The real game's path holds the true hidden state,
and a search must not read what its seat cannot know. Search expansion ignores what is hidden from the AI and
branches on categories with probabilities, carrying each probability into the node's cost; what that needs
(visibility, and the events that give a player knowledge) comes out of the log. See
[ai-player-design.md](ai-player-design.md).

### Decisions are data

The engine's interface is a step function, not a coroutine:

```text
start(setup)               -> Step
step(state, seat, answer)  -> Step
Step = Pending { requests } | Over { outcome }
```

Everything the engine is in the middle of lives in the state, as data: the agenda is a stack of frames (resolving
this stack entry, applying this zone-change batch, putting these triggers on the stack), and a pending prompt is a
frame waiting for an answer. There is no suspended Rust future, so a game waiting on a prompt is an ordinary value.

#### Card code that asks questions

Card code is ordinary straight-line code calling `g.ask(...)`. When it reaches a question with no answer yet, it
stops and the frame records the question. When the answer arrives, the effect runs again from its start, with the
answers so far replayed, and continues past the question. The mutations it made before the question are journalled
and rolled back before the re-run, and card code is deterministic, so this is exact. It costs about 0.2 us per
question over writing each card as explicit stages, which the spike measured, and keeps every card readable.

### Characteristics

- **Computed on query** (`chars()`), through the layers (613), from the object's card, its counters and the
  continuous effects that apply.
- **Text-changing effects (612, layer 3)** are `EffectKind::ChangeText { from, to }`, a `Word` (a color word or a
  subtype) replaced by another. The layers change the type line; card code reads every word of its rules text through
  `Query::words` (`Words`, the changes in timestamp order), so the card does what its changed text says. A token's
  colors and creature types are those of the text that made it (`Copiable::words`, 612.4), a permanent spell's effects
  go on to the permanent (400.7a), and an object that has left keeps the words it last had (`LastKnown::words`).
- **Static abilities granted by effects (613.1f, 613.7a)** are `granted_static` entries (`AbilityDef::Static`) of a card's
  `delayed` list, given with `EffectKind::AddAbility`. Once layer 6 has settled an object's abilities, those it still has
  (so "loses all abilities" later in timestamp order takes them) apply to it alone, with it as their source, in their own
  layer (7a onward, refused at table construction for an earlier one) at the timestamp of the granting effect, merged in
  timestamp order with the effects and other statics of that layer.
- **A card's printed characteristics are made once** (`CardTable::printed`), as a pure function of the card, both
  before and after the ability layer, so an object nothing acts on costs a lookup.
- **The static-ability sources are found once per position** (the epoch cache above), not on every query.
- **No strings at runtime:** keywords, types, subtypes and colors are enums or bitsets, card behavior is reached by
  function index, and nothing about a card is read from JSON while a game runs.

### Card data: three sources

1. **Characteristics, generated.** Name, mana cost, types, colors, power and toughness, loyalty, core keywords and
   rules text are `content/<dataset>/cards/<card>.json`, which the client also draws from, so the engine and the
   client cannot disagree. `crates/cantrip-oracle/build.rs` turns them into Rust constants at build time (into `OUT_DIR`; nothing
   generated is checked in, so adding a card adds files and edits none). The subtype enum in `cantrip-state` is
   rules data, every subtype of CR 205.3, edited only when the rules add one; the build fails naming the card
   and the subtype if a card prints one that is not there. A card with nothing but characteristics (a vanilla creature, a basic
   land) needs no file. An MTGJSON import may replace the hand-kept JSON later.
2. **Behavior, in Rust.** Every ability: a card's file adds behavior with struct update syntax over its generated
   constant (`CardDef { spell: ..., ..oracle::LIGHTNING_BOLT }`). The compiler checks all of it, so there is no
   load-time validator, nothing is parsed at runtime, and a typo is a build error rather than a surprise in a game.
3. **Presentation, in JSON, editable without a rebuild.** Art and frames, never read by the rules. Planned: a
   variant list per card (`content/<dataset>/presentation/<card>.json`, keyed by the card's name) with an art asset,
   artist, frame template and a default; decklists may name a variant; `Cantrip.Art` serves art by card and variant.
   The tag rides outside the rules state and is shown under the same visibility as the card's identity.

### Events and triggers

Triggers are indexed by the kinds of event they listen for (`EventMask`), so an event checks only the triggers that
could match it. "Leaves the battlefield" look-back (CR 603.10) reads the frozen last-known rows.

A static ability that stops abilities from triggering (CR 603.2: Torpor Orb's "Creatures entering don't cause abilities
to trigger", Doorkeeper Thrull, Hushbringer's "entering or dying") is a `CardDef::trigger_rules` entry, asked by
`collect_batch` of each event and each triggered ability the event matched (`Triggering`: its source, controller and
definition); true drops that triggering before it is put on the stack or counted against "triggers only N times each turn". Which permanents' rules
are asked follows CR 603.10: those on the battlefield after the event for an ability that triggers then (so a rule
entering with the permanent applies), those before it for a look-back ability (so one dying with the creature applies).
The rule judges the event through `Query` - an entering permanent as it is on the battlefield, copy and other effects
applied; a dead one by its last-known characteristics. Replacement effects and "as [this] enters" are not triggers and
are never asked about, and neither is the end of an "until" exile (CR 610.3, a one-shot effect). A trigger whose event is a sacrifice rather than the zone change it makes is
`TriggeredDef::on_sacrifice` (Hushbringer's ruling). The sources are found from the board, once per batch and only once
something has matched.

A static ability that makes triggered abilities trigger an additional time (CR 603.2d: Elesh Norn, Mother of Machines,
Panharmonicon, Teysa Karlov, Harmonic Prodigy) is a `CardDef::additional_triggers` entry, the same shape asked of the same
permanents (CR 603.10) after the suppressing rules: a triggering they stop is never doubled. Each entry that says yes adds
one triggering (two Elesh Norns make three), so a "one or more" ability triggered once for a batch triggers once more for
each such ability that applies to any of the batch's events. The triggerings are separate (`collect_batch` pushes the
pending trigger that many times): each is ordered, gets its modes and targets as it goes on the stack, checks its
intervening "if" again as it resolves (CR 603.4), and counts against "triggers only N times each turn" (no numbered rule covers that wording), so a
once-each-turn ability under a doubler still triggers once. `Triggering::permanent` says the ability is a permanent's: a
delayed triggered ability is no ability of the object (CR 603.2d, 603.7) and is never asked about, and an ability of a
card in another zone or of an emblem (CR 114.4) is not "of a permanent". Triggered mana abilities (CR 605.1b) are asked too, each
triggering resolving at once.

A trigger event is asked of the ability's source and controller (`TriggeredDef::fires`, an `Asker`): a matcher that says
"you" is written `when_asked` and reads the controller it is given. For an ability of a permanent that is the permanent's
controller; for a delayed triggered ability it is the delayed ability's own controller, the player who controlled the spell
or ability that created it (CR 603.7d-e), whoever controls its source afterwards and wherever the source has gone. Its
intervening "if" is asked with the same controller as it triggers.

A delayed triggered ability waits as a `Delayed` record in the state, on no object (CR 603.7): once at the next such step
(`DelayedWhen::Step`), at each such step for the rest of the game (`EachStep`: paradigm, CR 702.192a, which goes on whatever
happens to the card it came from), or on events this turn, until an event or until a player's next turn.

A crime (CR 700.13) is `Event::CrimeCommitted { player, by }`, emitted by `ops::commit_crime` from the three procedures that
choose targets for a new object - casting a spell (CR 601.2i), activating an ability (CR 602.2) and putting a triggered
ability on the stack (CR 603.3d), the trigger's controller the criminal - after their `BecameTarget` events, once for the
object however many of its targets are an opponent, a permanent, spell or ability an opponent controls, or a card in an
opponent's graveyard. A copy (CR 707.10) and a changed target (CR 115.7) choose no targets that way and commit none. It costs
nothing where nothing targets an opponent's things. The scenario dump lists a player's crimes as `BotN.Crimes`.

### Life: gaining it, and damage that can't be prevented

Every gain and loss of life goes through `ops::change_life` (CR 119.3): an effect's, a cost's (CR 119.4), damage (CR
120.3a), lifelink (CR 702.15b), and a life total set to a number (`Game::set_life_total`, CR 119.5: the player gains or loses
the difference). A gain does not happen while the player can't gain life (CR 119.7, `static_rules::cant_gain_life`): a
resolved effect about the player says so (`EffectKind::CantGainLife`, Screaming Nemesis's "for the rest of the game"), or a
static ability of a permanent on the battlefield does (`CardDef::cant_gain_life`, a `LifeGainRule` asked of the player:
`cantrip_mechanics::statics::players_cant_gain_life` for Sunspine Lynx, Giant Cindermaw, Rampaging Ferocidon and Leyline of
Punishment, `opponents_cant_gain_life` for Knight of Dusk's Shadow). The battlefield is searched only when a player would gain
life, and a permanent that has lost all its abilities has none of these. No life gain event is recorded for a gain that did
not happen, so "whenever you gain life" does not trigger (CR 119.9); paying life is a loss and is still allowed. The host shows
"You can't gain life." / "They can't gain life." beside a player's total while it applies (`Game::cant_gain_life`). Not built,
because no card in content has one: exchanges of life totals, costs that gain life and replacements of life gain (the rest of
CR 119.7), and "can't lose life" (CR 119.8).

"Damage can't be prevented" (CR 615.12) is asked by `ops::damage_can_be_prevented` before any prevention: a turn an effect said
so (Bonecrusher Giant's Stomp) or a `CardDef::unpreventable_damage` static (Questing Beast's combat damage,
`statics::damage_cant_be_prevented` for Sunspine Lynx and Leyline of Punishment). Protection (CR 702.16e, of a permanent or a
player's protection from everything) is the only prevention effect the engine has today; a prevention shield or other
prevention effect added later must ask the same function.

### Ending the turn and the cleanup step

"End the turn" (CR 724.1) is `Game::end_the_turn`, called by a resolving spell or ability (`turn::end_the_turn`): the
waiting triggered abilities cease to exist (724.1a), every object on the stack is exiled at once, the resolving one
included, whose later instructions still run (724.1b), and state-based actions are checked alone (724.1c,
`settle::state_based_actions_alone`). It records `Event::TurnEnded` and sets `Turn::ending`; the priority round below the
resolution then ends without asking anyone, and `next_step` removes every creature from combat if the turn was in
combat and goes straight to a cleanup step of the same turn - a new one if it was already in one (724.1d). The end step
is never begun, so nothing "at the beginning of the end step" triggers (724.1e). Abilities that trigger during the
process wait for that cleanup step (724.1f).

The cleanup step (CR 514) gives priority only when state-based actions were performed or a triggered ability is waiting
once its turn-based actions are done (514.3a), and then marks `Turn::cleanup_again`, so another cleanup step follows when
everyone has passed. Ultima, Time Stop, Sundial of the Infinite, Glorious End, Day's Undoing, Hurkyl's Final
Meditation, Discontinuity and Obeka, Brute Chronologist end the turn.

### Casting: when a cost's choices are made

A spell's costs are chosen and paid in the order of CR 601.2b-h, as the C# engine asked them: the permanents a cost
sacrifices or returns, its discards and exiles are chosen before the mana is paid, an additional cost's choices after
it. A cost of tapped permanents (`PermanentCost::tapped`, web-slinging's "return a tapped creature you control", CR
702.188a) is chosen once the mana is paid, since mana abilities come first (CR 601.2g) and one of them may have tapped
the creature (Spider-Sense's ruling): it is offered while a creature is tapped or a mana ability its controller could
activate would tap one, its own {T} or Springleaf Drum's, and a payment that leaves none tapped can't be completed, so
the cast is taken back (CR 601.2h). Only a spell's cost has one yet; an activated ability with one is refused by a
card-table test (`every_activated_ability_has_a_cost_activation_can_pay`) and an assertion in `activate`. The web
client's payment caption for a web-slinging cast says to tap a creature for mana while none is tapped; the cast that
leaves none is still taken back after it is paid.

### Restricted mana

Mana a restriction limits (CR 106.6) is kept in the pool apart from ordinary mana, one `ManaPool` per restriction
(`Player::restricted`, by `ManaRestriction::index`). A restriction is either a limit on what may be cast or activated or
an effect the mana has on what it pays for: `Spells(types)` pays only for casting a spell with one of the card types
(Mishra's Workshop, Great Hall of the Biblioplex, Gideon's Memorial), `Abilities(types)` only for activating an ability
whose source has one of them, or any source (Sunken Citadel, Steelswarm Operator, Omen Hawker), and
`NotNonartifactSpells`, `GainsHaste` and `LegendaryUncounterable` are the three single cards' own. What a payment is for
is `ManaPayment`, judged as the cost is paid (`actions::payment_for`): casting a spell on the stack (CR 601.2g), activating
an ability of a source in any zone - a mana ability's own mana cost too (CR 605.3a) - or anything else, which no
"abilities" mana pays: a special action (CR 116.2, turning a face-down permanent up, unlocking a door), "unless ... pays"
(CR 118.12). `ManaRestriction::may_pay` is the one rule, read by the payment (`restricted_for`), by the advice whether a
cost can be paid (`Available::covers_for`), and by the options offered while paying: a mana ability whose mana could pay
nothing of this cost is still offered (CR 601.2g) and is marked `pays_nothing`, which the bots read. The restriction's
mana goes where the player sends it: two mana of one restriction may pay one ability or two. The pool empties at the end
of each step and phase as any mana does (CR 106.4).

### Spells in an inset frame: Adventures and Omens

An adventurer card (CR 715) and an omen card (CR 720) print a second set of characteristics in an inset frame:
`CardDef::adventure` and `CardDef::omen`, from the content keys `adventure` and `omen`, and `CardDef::inset_faces` names
either with the way it is cast by (`WAY_ADVENTURE`, `WAY_OMEN`) and the face a granted way names it by. The card is cast as
that face wherever the card itself may be cast - from the hand, or from elsewhere by a permission, by its own say-so or by an
effect's (Yawgmoth's Will), for what the permission says to pay (CR 601.3, 715.3, 720.3) - but not as an Adventure by the
permission an Adventure gives the card it exiled (CR 715.3d). Only the face is judged to see whether it can be cast, at its own
speed (CR 715.3a, 720.3a), and on the stack it is that face alone (`Obj::copy`, CR 715.3b, 720.3b), which a copy of it and a
choice of new targets for it read too (CR 715.3c, 720.3c). As it resolves, an Adventure is exiled on its adventure (CR 715.3d)
and an Omen is shuffled into its owner's library by its controller (CR 720.3d); a copy of an Omen still shuffles its owner's
library and ceases to exist. Countered, or with every target gone, either goes to the graveyard. The host sends the face as
`ObjectView::inset_spell`, printed beside the card. Not built: casting the face from the top of a library
(`PlayRule::LibraryTopSpells`, which offers only the card itself).

### Legal moves as a first-class output

Every pending request comes with its legal answers enumerated, cheaply and in a stable order. The client's prompt
and the AI's move list are the same data.

## Interfaces

**The server.** `cantrip-host` implements the `IRulesHost` protocol as a sidecar (JSON lines over stdio,
[host-protocol.md](host-protocol.md)), so `GameSession`, events, the Discord bot and the web client do not depend on
the engine's language. The server finds the binary under `target/` when it runs from the repository, or by
`CANTRIP_NATIVE_HOST`. Calling the engine in-process through FFI is possible later if the sidecar shows up in
profiles; for human play it will not.

**Python (step 6).** `cantrip-py` exposes batches of games to a training loop: create N games, step them with a batch
of answers, branch, and encode each game's state for a seat into tensors. It encodes a redacted view, with hidden
cards masked or filled in from a belief model, never the true hidden state.

**Scenarios.** `cantrip-script` runs the `.cantrip` corpus in `scenarios/`; `cargo test` makes one test per file in
`scenarios/` (its build script lists the folder; nothing is registered). A scenario whose decks name a card the
engine does not have yet is skipped; one that can run must pass.

## Correctness

- **The scenarios are the reference.** A card is done when it is exact and its scenarios pass; a scenario that is
  wrong by the rules is fixed, with the rule.
- **`CANTRIP_VERIFY=1`** runs the engine's own consistency checks while it plays; `tools\test.cmd` sets it.
- **Seeds are not swept.** Positions come from scenarios (CLAUDE.md); random self-play is fuzzing, kept separate.

## Performance

The C# engine measured 261 us and 380 KB allocated per decision on one thread, scaling to only 4.5x across 16 cores,
0.8 ms for a deep copy and 25-93 ms to branch at a prompt by replay; 58% of its time was the deep copy per
zone-change batch and 28% the layer recompute. The targets set then: 10 us per decision, no allocation per step,
14x on 16 cores, a 10 us copy.

`cargo run --release -p cantrip-bench -- --games 128` measures the engine on the same games every run ("engine
answering ... us per decision", one thread). The spike
(nineteen cards, no features) answered in 1.95 us. With every card and every rules feature it is **5.06 us** per
decision (2026-10-15), 5.90 us after ending the turn (CR 724) and the exact cleanup step (2026-10-16, 5.99 us before); 6.03-6.45 us after mana
restricted to abilities of [type] sources (CR 106.6), against 6.08-6.14 us before on the same, noisy, machine. The enduring story's look at the board as each event happens (CR 702.195c, 2026-10-16) measured 6.15-6.55 us against 6.01-6.37 us before, alternating the two builds. The last six feature landings had taken it from 6.93 to 7.5-7.8 us; a sampling profile found
Stack items record each target's slot (`StackItem::spec_of`, 16 bytes more per stack item): 5.92-6.42 us (best 5.92) against 5.87-5.99 us (best 5.87) before (2026-10-16).
the cost in the layers, not in the new features' own code:

- `chars()` rebuilt a card's printed values for every object on every question, and scanned the battlefield for
  static-ability sources on every call. Printed characteristics are now made once per card, and the source list once
  per position (the epoch cache). 7.76 -> 5.34 us.
- The journal is never dropped (the game-log tree keeps it) and an `Undo` was as large as an `Obj` (296 bytes); the
  big rows are boxed in it (192 bytes), 5.30 -> 5.06 us. The frame entries stay inline: boxing them too measured
  slower.

Left on the profile: `chars()` is still about a fifth of the time, mostly the 170-byte `Chars` copied out per call
(callers that read one field could borrow the printed values instead); `mana_ability_options`, `castable` and
`available` call it per permanent per priority pass; the journal's `Vec<Undo>` regrowth (a chunked journal) and
`Frame` clones (184 bytes with heap parts) are the next costs; `Obj` is 288 bytes, 112 of them a `Copiable` that
nearly every object leaves empty. A game allocates about 13 MB, because the path's journal is kept; per-field undo
would shrink it.

## Plan

1. **Measure the C# engine.** Done (the numbers above).
2. **Spike.** Done: nineteen cards passed all 49 of their scenarios at 1.95 us per decision, 143 times faster than
   the C# engine, and the spike chose re-run-with-recorded-answers for card code.
3. **The rest of the rules bricks** and
4. **the cards,** together. Done (October 2026): feature agents built one rules area each and ported the cards
   waiting on it; card agents ported cards with the core closed to them and reported what it lacked. Every card in
   content is ported.
5. **Switch the server** to the engine behind `IRulesHost`. Done; the C# engine is deleted.
6. **Python bindings and the AI work.** Next.

Known gaps:

- Cost reductions by colored mana (`CostDelta::less_mana`, CR 118.7) take off the cost's plain symbols of that color and
  then generic mana. A reduction by an amount that includes hybrid mana, which the payer splits as it is applied (CR
  118.7e), is not built and panics if reached: no card in content reduces a cost by hybrid mana. The power-up cards whose
  own mana cost has a hybrid symbol (She-Hulk, Attorney-at-Law; Brawn, Amadeus Cho; Abomination, Terrifying Titan) would,
  and wait on it. Nor is a colored reduction meeting a hybrid or Phyrexian symbol of its color in the cost built.
- Power-up abilities are not yet a class other cards can name ("power-up abilities of other creatures you control cost
  {3} less", "whenever you activate a power-up ability", "can be activated an additional time"): Hulk, Gamma Goliath,
  Marvel Boy, Wonder Man, Kang, Advancing the Spirit and Quinjet Technician wait on it.
- The web client cannot yet send three answers the engine asks for: naming colorless mana (Squandered Resources),
  choosing which kind of counter to remove (`Label::Counter`, Glen Elendra Guardian), and turning a face-down
  permanent face up (`TurnFaceUp`).
- Serving a scenario's handed-over position to the web client, to look at or play on from, went with the C# engine
  and is not rebuilt.
- Web-slinging cards that wait: Spider-UK ("if two or more creatures entered the battlefield under your control this
  turn") needs the controller a creature entered under, which the history cannot answer once control has changed (no
  event records a change of control); Arachne, Psionic Weaver needs a card type chosen as it enters (`Entering` has a
  color, a subtype, a player, a name, odd or even, no card type) and a prompt for it; Peter Parker // Amazing
  Spider-Man's back face grants web-slinging to the legendary spells its controller casts, a way a permanent gives to
  cards in hand, which only exists for cards in graveyards (`granted_ways_of`).
- `web_slinging::way` is a way from the hand (`ZoneKind::Hand`), like every printed alternative cost. Cast from
  elsewhere at priority by a permission or the card's own say-so, a hand way is offered too (`castable`'s `elsewhere`;
  Spider-Sense from the graveyard under Yawgmoth's Will is a scenario, and sneak's instant-speed way likewise), unless its
  text names the hand (`WayDef::only_from`: warp, CR 702.185a, so a warped card is cast from exile for its mana cost), but a cast made while an effect resolves
  (`cast_ways_during_resolution`) is offered for the card's mana cost and faces only: an effect that lets a spell be cast
  paying its costs ("cast it from exile this turn") as it resolves can't use web-slinging or any other alternative cost.
- Costs to attack (CR 508.1g-h: "creatures can't attack you unless their controller pays {1} for each") are not built:
  Dáin, Lord of the Iron Hills, the one storied card not in content, waits on it.
- No scenario reaches Questing Beast's unpreventable damage, Emrakul's extra turn on cast, or Tamiyo's -7 emblem.
- Suspend does not give a creature cast that way haste (no card in content needs it), and several creatures put onto
  the battlefield one after another by an effect (Ghost Vacuum) enter one at a time rather than together.
- Rooms (CR 709.5) are built: `Marks::doors` holds the unlocked designations, `layers::doors` gives a Room on the
  battlefield the name, mana cost and abilities of its unlocked doors, the unlock special action is an intrinsic ability
  per locked door (`intrinsic::UNLOCK_LEFT`/`UNLOCK_RIGHT`, cost read off the door), and `Event::DoorUnlocked` is what
  "when you unlock this door" listens for. Not yet: a door with static abilities, replacement effects or an enters
  rule (the card table refuses one), instructions to unlock or lock a door (`Game::unlock_door` exists for the first,
  nothing locks), "fully unlocks", and a copy of a Room spell (`resolve_copy_as_token` gives the token its door, but
  no card in content can copy a Room spell to prove it).
  Known deviations from the rules: the unlock is an intrinsic ability entry, so an effect that removes abilities
  (Humility) would remove it, though CR 709.5e says the unlock is not an ability; a copy effect's "except its name is X"
  is overwritten by `layers::doors`, which names a Room by its unlocked doors; `resolve_copy_as_token` drops the copy
  exceptions of a Room spell's copy; a granted cast from a graveyard (Serra Paragon) is judged on the whole Room card, mana
  value 8, never on a door (Known bug scenario `serra-paragon-casts-either-door-of-a-room-in-the-graveyard`).

## Open questions

- **Checkpoints:** how often the live game keeps a full state checkpoint, so that resuming or jumping far through the
  tree does not replay from the root.
- **What a step is for search:** whether the engine exposes plan-level moves (a whole combat) or only primitive
  answers, with plans built above it.
- **Multiplayer:** the rules are written for two players today, and multiplayer (CR 800) must not be designed out.
- **Determinism across machines:** the same results on every platform the server and training run on, which rules
  out depending on hash map iteration order or unordered floating point.
