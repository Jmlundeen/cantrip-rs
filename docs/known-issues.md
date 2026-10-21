# Known issues

Problems being tracked in the client, the tooling and the harness, and engine paths no scenario runs yet, in one place.
A scenario that shows a bug is still committed with a `# Known bug:` line (`scenarios/README.md`, "Known bugs"); this
file is where that bug is described, ranked and followed. When it is fixed, delete the entry in the fixing commit (the
history keeps it).

**This file never accepts a deviation from the game's rules** (`CLAUDE.md`, "No rules deviations"). A rules deviation
found in review blocks the change that has it. One found in existing code is fixed, or reported to a human at once; it
appears here only while its fix is under way, marked as such, and is never left to wait for "a card that needs it". A
path no scenario runs gets a test - a Rust test with test-only cards when no printed card reaches it - rather than an
entry. The only allowed deviations are the human decisions in `docs/rules-exceptions.md`, and limits of scale beyond
any practical game or card (`CLAUDE.md`, "Scale is the one standing exception"), each listed here as "Won't fix:
beyond practical scale" with the limit, what happens past it, and why play cannot reach it.

## Priority

| Priority | Meaning |
| --- | --- |
| Pri-0 | A crash, a hang, a desync between engine and client, or a game that cannot go on. A blocker. |
| Pri-1 | Any rules violation, however rare and whether or not a printed card reaches it today. A blocker. |
| Pri-2 | A choice the client cannot offer or show, or a wrong display of a right game state. |
| Pri-3 | Cosmetic, a harness or tooling gap, or a path no scenario runs yet. |

Pri-0 and pri-1 are fixed before any other work is started or landed (`CLAUDE.md`, "Bug priority"); one is listed here
only while its fix is under way ("Fix queued" or "Fix under way"), and the list of them is kept at zero. A limit of scale
beyond any practical game or card is "Won't fix" (`CLAUDE.md`, "Scale is the one standing exception"). The tactical bot's
own judgement is pri-3: the engine is right, and the bot only plays less well.

## Entry template

```

### KI-<n>: <one-line description of what is wrong>

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open / Suspected / Unexercised | engine / mechanics / card / client / harness | YYYY-MM-DD, how |

**Notes.** What happens, what the rules require (CR numbers, rulings), where in the code, and what a fix needs.

**Scenarios.**
- [`scenarios/<name>.cantrip`](../scenarios/<name>.cantrip) - what it shows (marked `# Known bug`), or "none yet" and
  which implemented card could reach it.
```

## Summary

| ID | Issue | Priority | Status |
| --- | --- | --- | --- |
| [KI-113](#ki-113-the-engine-panics-on-costs-it-has-not-built) | The engine panics on costs it has not built | Pri-0 | Fix queued |
| [KI-54](#ki-54-where-the-loop-watchs-judgement-is-not-the-rules) | Where the loop watch's judgement is not the rules' | Pri-1 | Fix queued |
| [KI-103](#ki-103-mana-costs-x-kicker-and-the-mana-a-cast-records-are-narrower-than-play-reaches) | Mana costs, X, kicker and the mana a cast records are narrower than play reaches | Pri-1 | Fix queued |
| [KI-104](#ki-104-counters-power-toughness-and-damage-are-narrower-than-play-reaches) | Counters, power, toughness and damage are narrower than play reaches | Pri-1 | Fix queued |
| [KI-105](#ki-105-an-objects-abilities-text-changes-and-chosen-modes-are-held-in-fixed-lists) | An object's abilities, text changes and chosen modes are held in fixed lists | Pri-1 | Fix queued |
| [KI-107](#ki-107-a-layer-1-copy-static-ability-is-not-counted-among-what-could-give-a-look-back-ability) | A layer-1 copy static ability is not counted among what could give a look-back ability | Pri-1 | Fix queued |
| [KI-111](#ki-111-a-rooms-unlock-is-an-ability-and-its-doors-name-overwrite-a-copys-exceptions) | A Room's unlock is an ability, and its doors' name overwrite a copy's exceptions | Pri-1 | Fix queued |
| [KI-112](#ki-112-a-creature-cast-with-suspend-does-not-gain-haste) | A creature cast with suspend does not gain haste | Pri-1 | Fix queued |
| [KI-115](#ki-115-a-fuse-card-cannot-be-cast-fused-while-a-spell-or-ability-resolves) | A fuse card cannot be cast fused while a spell or ability resolves | Pri-1 | Fix queued |
| [KI-98](#ki-98-a-station-permanent-lists-its-reached-striations-keyword-again-as-granted) | A station permanent lists its reached striation's keyword again, as granted | Pri-2 | Open |
| [KI-101](#ki-101-a-player-out-of-a-game-that-goes-on-sees-it-a-step-late) | A player out of a game that goes on sees it a step late | Pri-2 | Open |
| [KI-108](#ki-108-what-can-be-cast-is-over-offered-where-a-mana-abilitys-yield-is-unknown-or-its-cost-is-shared) | What can be cast is over-offered where a mana ability's yield is unknown or its cost is shared | Pri-2 | Open |
| [KI-114](#ki-114-the-web-client-cannot-name-colorless-mana-for-a-sacrificed-land-that-could-also-produce-a-color) | The web client cannot name colorless mana for a sacrificed land that could also produce a color | Pri-2 | Open |
| [KI-3](#ki-3-engine-paths-no-scenario-runs) | Engine paths no scenario runs | Pri-3 | Fix queued |
| [KI-35](#ki-35-the-tactical-bot-counts-no-summoning-sick-creature-as-a-mana-source-under-shang-chi) | The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi | Pri-3 | Open |
| [KI-49](#ki-49-the-tactical-bot-does-not-know-infect-or-poison) | The tactical bot does not know infect or poison | Pri-3 | Open |
| [KI-57](#ki-57-the-tactical-bot-does-not-know-islandwalk) | The tactical bot does not know islandwalk | Pri-3 | Open |
| [KI-71](#ki-71-the-tactical-bots-combat-planning-ignores-bands) | The tactical bot's combat planning ignores bands | Pri-3 | Open |
| [KI-78](#ki-78-the-tactical-bot-attacks-a-player-whom-protection-keeps-from-being-dealt-damage) | The tactical bot attacks a player whom protection keeps from being dealt damage | Pri-3 | Open |

## Issues

### KI-114: The web client cannot name colorless mana for a sacrificed land that could also produce a color

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-2 | Open | client | 2026-10-21, moved from `docs/engine-design.md`'s known gaps |

**Notes.** Squandered Resources adds a type the land it sacrifices could produce (CR 106.7). The page offers {C} only where
colorless is the land's only type (`CardActions.ChoosesColorless`); a land that could produce {C} and a color is offered
its colors alone. No one is stuck: the ability is offered, as a mana ability is, and its other types answer it. Choosing
which kind of counter to remove (`Label::Counter`, Glen Elendra Guardian) is a modes question the page's modes dock answers,
and turning a face-down permanent face up is a card action it offers (`MainPage.Prompts.cs`, `TurnFaceUp`).

**Scenarios.** None.

### KI-3: Engine paths no scenario runs

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Fix queued | engine | 2026-10-15/16, fixes in the U-B-R-G Prowess run and the known-bug fixes |

**Notes.** Code written for every card the rule describes, where no implemented card reaches it yet. Each is a
candidate bug until something runs it; when a card that reaches one is implemented, its scenarios cover the path and
the line goes. So does a line whose path a Rust test drives through the real engine with test cards (the tests under
`crates/cantrip-script/tests`, `real_cards` among them with every card the engine has, and the rules crate's own), where
no printed card can reach it.

| Path | Covered instead by | Why no scenario |
| --- | --- | --- |
| A fused split spell no longer allowed once its proposal is complete (CR 601.6: `actions::cast` asks `CardDef::spell_rules` again of the spell on the stack), its mana value the sum of its halves' | the check before the cast (every Gaddock Teeg scenario); a back face cast transformed, whose spell has its front face's mana value (CR 712.8c), in a Rust test (`transformed_cast_tests.rs`) | Fuse is built (Turn // Burn): a scenario casting it fused under Gaddock Teeg-style rule now covers this, queued. |

**Scenarios.** None, by definition.

### KI-35: The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open | client | 2026-10-16, review of Shang-Chi, Master of Kung Fu |

**Notes.** Under Shang-Chi, Master of Kung Fu ("You may activate abilities of creatures you control as though those creatures
had haste", CR 302.6; `CardDef::activate_as_though_haste`) the engine offers the {T} mana abilities of creatures that came
this turn, Shang-Chi's own among them, but the tactical bot's mana planning
(`dotnet/src/Cantrip.Client/Tactics/TacticalPlay.Priority.cs`, the source counts that skip `SummoningSick` creatures) leaves
them out, so it plans with less mana than it has. The view has no word for the permission; a fix counts a sick creature as
a source when the actions offered for it include a mana ability, or the view says its {T} abilities may be activated.

**Scenarios.** none: a bot's plan is not scenario'd. Shang-Chi's scenarios show the engine side.

### KI-49: The tactical bot does not know infect or poison

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open | client | 2026-10-17, review of Inkmoth Nexus |

**Notes.** The engine is right: damage from a source with infect is -1/-1 counters to a creature and poison counters to a
player (CR 702.90b-c, 120.3b, 120.3d), and ten poison counters lose the game (CR 704.5c). The tactical bot
(`dotnet/src/Cantrip.Client/Tactics`) has no notion of either: `CardReading.cs` passes "infect" over as a keyword it does
not know, and nothing in the bot reads a player's poison counters, so it scores an infect creature's damage (Inkmoth
Nexus, Blightsteel Colossus) as life lost, races and blocks as if it were, and does not see a player near ten poison. A fix
reads infect into `Keywords` and gives the bot's damage and race arithmetic a poison clock beside the life one. The web
client shows poison on the player rail and the language-model client in its text, so a person and the model are not
affected.

**Scenarios.** none: a bot's judgement is not a rules outcome; a bot-lab duel with infect decks would show it.

### KI-54: Where the loop watch's judgement is not the rules'

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-17, CR 104.4b loop detection; narrowed by the loop hardening and CR 732.3 |

**Notes.** `loops.rs` finds the loops the rules make a draw and applies CR 732.3 (its module comment is the design). A
loop of mandatory actions is a draw when a position comes round with nothing asking the history in between - through
forced priorities, or, in a busy step, with every player only passing at priority since (CR 732.4-5: an instant no loop
object calls for can't be required). A priority asked again in the same exact position in a busy step, with two or more
players having chosen to act since, is a fragmented loop: the active player (or the first involved) must choose otherwise,
and chooses where - only their last action of the round, once every answer of theirs before it was the same as last
round, is refused. Every other loop goes on for as long as its players choose (no shortcuts, CR 732.2; RX-2), up to
`MAX_GAME_EVENTS` (RX-1). What is still not the rules' procedure, each a pri-1 fix queued after the layer redesign:
- (a) CR 732.3 is found only by the exact position: a fragmented loop whose rounds differ only in what the rules' "game
  state" leaves out is not found - continuous effects that pile up or are superseded round after round (the rule's own
  example: "gains flying" and "loses flying" until end of turn each make an effect, so the effects list grows), or a
  history read between the rounds whose answer is the same each time. Its players can go on with it.
- (b) A player made to choose otherwise is refused only an action at priority, a single target, card, mode or player, and
  a "may" taken; a choice of several targets or cards at once, an order of triggers or a number repeated at their last
  action is still accepted.
- (c) A mana ability whose whole cost is {T} is treated as no choice when nothing more mana could pay for is castable, even
  when its effect does more than add mana (a painland's damage).
- (d) The watch begins only in a step with `BUSY_STEP` events, so a loop is found at its first round after that - some tens
  of rounds in - rather than at its second round. From then on it keeps every question of the step.
- (e) State triggers (CR 603.8): a card's text is one instruction unless it marks a sentence's end with `end_sentence`,
  which only some cards do (every Oracle sentence needs one), and a staged frame (`Game::ask_then`) drops a pending
  `state_check_due` when it suspends.

**Scenarios.**
- [`scenarios/oblivion-ring-loop-a-player-can-break-with-an-instant-is-not-a-draw.cantrip`](../scenarios/oblivion-ring-loop-a-player-can-break-with-an-instant-is-not-a-draw.cantrip),
  [`scenarios/worldgorger-dragon-loop-ends-when-animate-dead-chooses-another-creature-card.cantrip`](../scenarios/worldgorger-dragon-loop-ends-when-animate-dead-chooses-another-creature-card.cantrip)
  and [`scenarios/worldgorger-dragon-loop-chosen-forty-times-goes-on-until-its-player-chooses-otherwise.cantrip`](../scenarios/worldgorger-dragon-loop-chosen-forty-times-goes-on-until-its-player-chooses-otherwise.cantrip):
  loops with a choice, played round by hand until the script breaks them.
- [`scenarios/oblivion-ring-loop-a-player-never-breaks-with-the-instant-they-hold-is-a-draw.cantrip`](../scenarios/oblivion-ring-loop-a-player-never-breaks-with-the-instant-they-hold-is-a-draw.cantrip)
  and [`scenarios/worldgorger-dragon-loop-whose-player-always-declines-fecundity-is-a-draw.cantrip`](../scenarios/worldgorger-dragon-loop-whose-player-always-declines-fecundity-is-a-draw.cantrip):
  a loop of passes, a draw as a loop of mandatory actions, and one of a "may" declined, drawn at a low event limit (RX-1, RX-2).
- `crates/cantrip-script/tests/fragmented_loop.rs`: CR 732.3 with test-only cards (tap and untap an artifact in turn).

### KI-57: The tactical bot does not know islandwalk

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open | client | 2026-10-17, adding Lord of Atlantis |

**Notes.** The engine is right: a creature with islandwalk can't be blocked while the defending player controls an Island
(CR 702.14c, `turn::can_block`), and the blockers question offers only the pairs the rules allow, which the bots follow. The
tactical bot's own combat arithmetic (`dotnet/src/Cantrip.Client/Tactics`, `Unit.CouldBlock` and `Exposure`) reads no
landwalk: `CardReading.cs` passes "Islandwalk" over, and a unit knows nothing of the defending player's lands, so the bot
plans its attacks as if an islandwalker could be blocked by an opponent with an Island, and counts its own blockers against
one. A fix reads islandwalk into `Keywords` and makes a landwalker unblockable in the bot's planning when the defending
seat controls a land of its type. The web client shows the keyword on the card, and the language-model client reads it.

**Scenarios.** none: a bot's judgement is not a rules outcome.

### KI-71: The tactical bot's combat planning ignores bands

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open | client | 2026-10-17, review of banding |

**Notes.** The engine is right (CR 702.22h-k). The tactical bot's blocks and attacks (`TacticalPlay.Combat.cs`, `Combat`) are
judged pair by pair: it does not know that blocking one member of a band blocks all of it, nor that the opponent divides its
blocker's damage among the band (702.22k) or its attacker's damage among banding blockers (702.22j). Its band choice and the
divisions it makes are on purpose (`TacticalPlay.Choices.cs`, the engine's sparing suggestion).

**Scenarios.** none: a bot's judgement is not a rules outcome.

### KI-78: The tactical bot attacks a player whom protection keeps from being dealt damage

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-3 | Open | client | 2026-10-18, Absolute Virtue |

**Notes.** The engine is right: damage from an opponent's sources to a player with protection from them, or from everything,
is prevented (CR 702.16e, 702.16j, 702.16k: Absolute Virtue, Teferi's Protection, The One Ring), and the host tells both seats
("They have protection from Bot1."). Targeting is the engine's, so the bot never aims a spell at such a player. But its attack
planning (`TacticalPlay.Combat.cs`, `Combat`) scores the damage an unblocked attacker would deal that player as life lost, so it
may attack into a protected player for nothing and leave itself open. A fix tells the bot, in the view, whom each player is
protected from (a field on `PlayerSummaryView` rather than the text), and has the attack plan count no damage to them.

**Scenarios.** none yet.

### KI-98: A station permanent lists its reached striation's keyword again, as granted

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-2 | Open | client / host | 2026-10-20, Wurmwall Sweeper's browser scenario (KI-25) |

**Notes.** A wrong display of a right game state. A station card's striation abilities (CR 721.2a) are its own static
abilities, which give the permanent the keyword while it has the charge counters (`station::charged`,
`statics::have_flying`). The host's `ability_changes` counts every keyword the permanent has and its card definition does
not as granted, so Wurmwall Sweeper at four or more charge counters reads "... / 4+ | Flying | 2/2 / Flying": the
striation's flying said a second time, as though another effect gave it. A fix leaves out of `granted_abilities` what the
permanent's own reached striation prints.

**Scenarios.** `wurmwall-sweeper-shows-its-striation-power-and-toughness-in-the-browser` (browser) checks the face in hand and
below the threshold only, for this reason.

### KI-101: A player out of a game that goes on sees it a step late

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-2 | Open | host / client | 2026-10-21, a player out of time in a game of four (`TableClockTests`) |

**Notes.** A wrong display of a right game state. When a player of a game of more than two leaves it - their clock ran out
(`GameSession.TimeOut`), or they lost - the game goes on for the rest (CR 800.4a), but that player's own page is told only
once the run being played is published: the host shows a run in progress to its answering seat alone, and every other seat
keeps the position from before it (`session.rs`, `view_for`). Until then the page shows the player still in, and its last
question; `GameClient` drops the question once its own view shows it out. The server judges who is still in from every
seat's view and its own leaves, so the rules are unaffected (`GameSession.InGame`, `TimeOutTests`). A fix sends a seat that
has left its own view at once, as the answering seat's is.

**Scenarios.** None; `TableClockTests.In_a_game_of_four_a_player_out_of_time_leaves_and_the_others_play_on` answers the others'
questions until the departed seat is told.

### KI-103: Mana costs, X, kicker and the mana a cast records are narrower than play reaches

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine / mechanics / cards | 2026-10-21, the engine-wide audit (workstream W1, parked on `pool5-w1`) |

**Notes.** Mana doubling reaches these within a timed game, so each is a rules violation, not a limit of scale:
- A mana cost's counts are `u8` (`ManaCost`, `crates/cantrip-state/src/vocab.rs:281`): a total past 255 stops at 255, so
  Fireball with X = 300 costs {255}{R}.
- Multikicker's count is a `u8` and its cost is added in `u8`: Everflowing Chalice kicked 128 times costs {0}.
- An activated ability's X is multiplied into the generic cost in `u8` (`actions.rs`, where an activation announces X:
  `(n * x).min(255) as u8`): Kessig Wolf Run with X = 300 costs {255}{R}{G}. It saturates since the event-cap sweep, so it
  no longer wraps or panics, but it still stops at 255.
- X is a `u16` (`StackItem::x`, `Obj::x`), and X paid in life stops at 1000 (`actions.rs`,
  `payable_life(..).clamp(0, 1000)`).
- A reflexive trigger's "that many" becomes its X, so it is a `u16` too (`PendingTrigger::count`,
  `Game::reflexive_trigger_counted`): the counts Loamcrafter Faun, Miasma Demon, Swashbuckler Extraordinaire, Seasoned
  Cryomancer and Ill-Timed Explosion give it saturate at 65,535 rather than wrap, and widen with X.
- Splice onto an Arcane spell takes at most two cards (`eligible.len().min(2)`, `actions.rs:4850`; CR 702.47 allows any
  number), and twobrid symbols at most eight.
- The mana a cast records per kind is a `u8` (`ManaTally`, `vocab.rs:250`): Dyadrine counts at most 255 spent.
- Cards narrow their own reductions with `.min(255) as u8` (Eddymurk Crab, Embercleave, Affinity, Ghalta and others).
A fix widens these to `u32` with saturating arithmetic, and a cost that saturates is judged unpayable, never cheap.

**Scenarios.** None yet; each needs a test that fails on today's code (Everflowing Chalice kicked 128 times, Kessig Wolf
Run at X = 256, Fireball at X = 300).

### KI-104: Counters, power, toughness and damage are narrower than play reaches

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine / mechanics / cards | 2026-10-21, the engine-wide audit (workstream W2) |

**Notes.** Counter and damage doublers reach these within a timed game:
- An object's counters are `i16` (`Obj::counters`), and a doubling replacement (`*n *= 2`, Doubling Season) wraps
  negative, after which the counters are dropped.
- Power and toughness are `i16` (`Chars`, `Pump`, `Copiable::pt`), and the layers add them with plain `+`, which wraps.
  Some thirty cards and mechanics make a power or toughness from a count of objects with `count() as i16` or
  `len() as i16` (Cranial Plating, Nethergoyf, Ms. Marvel, Yuriko, `goyf`, `melee`, `lands.rs` and others), which wraps
  negative past 32,767 objects.
- Damage is a `u16` (`Obj::damage`, `ops::damage`, `Event::Damage`); `damage as i16 >= toughness` (`settle.rs`,
  `turn.rs`) lets a creature dealt more than 32,767 damage survive it.
- A player's poison, energy and experience counters are `u8` (`object.rs:1137-1139`, `pay_energy`).
- A divided share of damage or counters is cut to the damage and counter widths (`damage_divided`, `counters_divided`).
A fix widens these to `i32`/`u32` with saturating arithmetic.

**Scenarios.** None yet: Doubling Season past 32,767 counters; a creature past 32,767 power; more than 65,535 damage to
a creature.

### KI-105: An object's abilities, text changes and chosen modes are held in fixed lists

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-21, the engine-wide audit (workstream W3, part on `pool-4-w3-wip`) |

**Notes.**
- An object holds at most 32 abilities (`MAX_ABILITIES`, `object.rs:1083`); a grant past it is not given, which stacked
  grants or a chain of Thespian's Stage copies reach.
- A copy holds at most six distinct abilities added by exceptions (`Copiable::EXTRA_ABILITIES`), and a seventh panics
  (`with_ability`'s `expect`): a crash, pri-0 if reached.
- An object keeps at most four text-changing effects (`Words::MAX`, `vocab.rs:709`); a fifth is ignored.
- The modes chosen for a spell are a `u8` mask (`StackItem::modes`, `object.rs:575`) with two bits a mode where modes
  repeat, so a fourth pick of a mode carries into the next (the Seasons choose up to five).
- The bands of `AbilityRef::index` are guarded only by `debug_assert`.
A fix gives each a capacity past practical play, handled gracefully at its bound; a check of every card's lists at table
construction for the bands; and the modes kept as the chosen indices in order.

**Scenarios.** None yet.

### KI-111: A Room's unlock is an ability, and its doors' name overwrite a copy's exceptions

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-21, moved from `docs/engine-design.md`'s known gaps |

**Notes.**
- The unlock special action is an intrinsic ability entry per locked door (`intrinsic::UNLOCK_LEFT`/`UNLOCK_RIGHT`, given
  by `layers::doors`), so an effect that removes abilities (Humility) removes it, though CR 709.5e says unlocking a door
  is a special action, not an ability.
- A copy effect's "except its name is X" is overwritten by `layers::doors`, which names a Room by its unlocked doors.
- `resolve_copy_as_token` drops the copy exceptions of a Room spell's copy.

**Scenarios.** None yet.

### KI-112: A creature cast with suspend does not gain haste

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | mechanics | 2026-10-21, moved from `docs/engine-design.md`'s known gaps |

**Notes.** CR 702.62a: a creature spell cast by suspend's last-counter trigger gains haste until its controller loses
control of the spell or the permanent it becomes. `cantrip_mechanics::suspend` casts the card and gives it nothing. No
creature with suspend is in content yet, but the rule is the keyword's.

**Scenarios.** None yet.

### KI-115: A fuse card cannot be cast fused while a spell or ability resolves

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-21, a comment that pointed here with no entry |

**Notes.** A split card with fuse in its owner's hand may be cast fused (CR 702.102a), and a resolving effect that lets its
controller cast a card from their hand offers the ways it could be cast (`Game::cast_ways_during_resolution`, CR 608.2g).
The fused cast is not among them, so it is not offered there.

**Scenarios.** None yet.

### KI-113: The engine panics on costs it has not built

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-0 | Fix queued | engine | 2026-10-21, the sweep for known gaps |

**Notes.** Asserts that stop the engine where a cost part is not built, each kept from a game only by what cards there are:
- an activated ability's behold (`actions.rs`, `activate`: "an activated ability's behold is not built"), and a mana
  ability whose cost names tapped permanents - refused for every card in the table by
  `every_activated_ability_has_a_cost_activation_can_pay` (cantrip-engine);
- a cost paid during a resolution with parts other than mana and life (`actions::may_pay` and `may_pay_x`: "a cost paid
  during resolution is mana and life; its other parts are not built").
A fix builds each, or refuses the cost gracefully (the cast or payment taken back) where it is reached.

**Scenarios.** None.

### KI-107: A layer-1 copy static ability is not counted among what could give a look-back ability

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-21, review of the look-back record |

**Notes.** Before a move that can trigger a look-back ability (CR 603.10a), the engine records the characteristics of each
permanent that could have one (`settle::Then::now`). The set is built from each card's own abilities, copy exceptions,
abilities that static abilities give, and effects. A static ability of layer 1 that makes a permanent a copy of another
card (Volrath's Shapeshifter's kind) is not counted, so a look-back ability it gives would not be recorded. No card in
content has such a static. A fix counts layer-1 copy statics in the set.

**Scenarios.** None yet: needs a test card.

### KI-108: What can be cast is over-offered where a mana ability's yield is unknown or its cost is shared

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-2 | Open | engine | 2026-10-21, review of the exact payability check |

**Notes.** A choice the client offers that cannot be made; the payment takes the cast back (CR 733.1), so no game state is
wrong. A mana ability whose yield is not stated (Sunken Ruins, X costs, Phyrexian or {2/W} symbols in its cost) makes
every cost count as payable. Life, energy and sacrifices that several mana abilities would share are judged for each
alone: three Mana Confluences at 1 or 2 life count as three mana, so Raffine is offered. And the search's time budget
falls back to offering the cast on rare heavy boards. A permanent Urza, Lord High Artificer could tap that also has a mana ability
without {T} counts Urza's tap apart from its own abilities, as if both could be used (`available_without`, `Repeat::Tap`).

**Scenarios.** None yet.
