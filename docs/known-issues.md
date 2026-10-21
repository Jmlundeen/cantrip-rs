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
only while its fix is under way, and the list of them is kept at zero. The older entries' Critical / High / Medium /
Low read as pri-0 / pri-1 / pri-2 / pri-3, except that every wrong game result is pri-1 whatever it says.

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

| ID | Issue | Severity | Status |
| --- | --- | --- | --- |
| [KI-3](#ki-3-engine-paths-no-scenario-runs) | Engine paths no scenario runs | Low | Unexercised |
| [KI-35](#ki-35-the-tactical-bot-counts-no-summoning-sick-creature-as-a-mana-source-under-shang-chi) | The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi | Low | Open |
| [KI-10](#ki-10-a-lone-action-whose-cost-depends-on-its-target-shows-only-the-printed-cost-until-the-payment) | A lone action whose cost depends on its target shows only the printed cost until the payment | Low | Open |
| [KI-49](#ki-49-the-tactical-bot-does-not-know-infect-or-poison) | The tactical bot does not know infect or poison | Low | Open |
| [KI-51](#ki-51-a-land-whose-subtype-is-set-keeps-the-copiable-values-other-than-abilities-a-copy-effect-gave-it) | A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it | Low | Exercised; reading unconfirmed |
| [KI-54](#ki-54-where-the-loop-watchs-judgement-is-not-the-rules) | Where the loop watch's judgement is not the rules' | Pri-1 | Fix queued |
| [KI-56](#ki-56-a-counter-ban-judges-an-entering-permanent-as-the-object-it-was-before-it-moved) | A counter ban judges an entering permanent as the object it was before it moved | Low | Unexercised |
| [KI-63](#ki-63-control-layer-2-is-not-ordered-by-the-layer-pass) | Control (layer 2) is not ordered by the layer pass | Pri-1 | Fix queued |
| [KI-64](#ki-64-control-changing-effects-that-decide-one-another-are-settled-by-a-capped-fixpoint) | Control-changing effects that decide one another are settled by a capped fixpoint | Pri-1 | Fix queued |
| [KI-71](#ki-71-the-tactical-bots-combat-planning-ignores-bands) | The tactical bot's combat planning ignores bands | Low | Open |
| [KI-78](#ki-78-the-tactical-bot-attacks-a-player-whom-protection-keeps-from-being-dealt-damage) | The tactical bot attacks a player whom protection keeps from being dealt damage | Low | Open |
| [KI-98](#ki-98-a-station-permanent-lists-its-reached-striations-keyword-again-as-granted) | A station permanent lists its reached striation's keyword again, as granted | Pri-2 | Open |
| [KI-99](#ki-99-a-spell-or-ability-with-more-than-232-targets-or-a-question-of-more-than-231-candidates) | A spell or ability with more than 2^32 targets, or a question of more than 2^31 candidates | Won't fix | Won't fix: beyond practical scale |
| [KI-100](#ki-100-a-choice-search-or-sacrifice-of-more-than-232-cards) | A choice, search or sacrifice of more than 2^32 cards | Won't fix | Won't fix: beyond practical scale |
| [KI-101](#ki-101-a-player-out-of-a-game-that-goes-on-sees-it-a-step-late) | A player out of a game that goes on sees it a step late | Pri-2 | Open |

## Issues

### KI-3: Engine paths no scenario runs

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-15/16, fixes in the U-B-R-G Prowess run and the known-bug fixes |

**Notes.** Code written for every card the rule describes, where no implemented card reaches it yet. Each is a
candidate bug until something runs it; when a card that reaches one is implemented, its scenarios cover the path and
the line goes. So does a line whose path a Rust test drives through the real engine with test cards (the tests under
`crates/cantrip-script/tests`, `real_cards` among them with every card the engine has, and the rules crate's own), where
no printed card can reach it.

| Path | Covered instead by | Why no scenario |
| --- | --- | --- |
| A fused split spell no longer allowed once its proposal is complete (CR 601.6: `actions::cast` asks `CardDef::spell_rules` again of the spell on the stack), its mana value the sum of its halves' | the check before the cast (every Gaddock Teeg scenario); a back face cast transformed, whose spell has its front face's mana value (CR 712.8c), in a Rust test (`transformed_cast_tests.rs`) | No scenario casts a fused spell (Turn // Burn, mana value 5) that a rule forbids by its whole mana value, such as Gaddock Teeg's. |

**Scenarios.** None, by definition.

### KI-35: The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-16, review of Shang-Chi, Master of Kung Fu |

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
| Low | Open | client | 2026-10-17, review of Inkmoth Nexus |

**Notes.** The engine is right: damage from a source with infect is -1/-1 counters to a creature and poison counters to a
player (CR 702.90b-c, 120.3b, 120.3d), and ten poison counters lose the game (CR 704.5c). The tactical bot
(`dotnet/src/Cantrip.Client/Tactics`) has no notion of either: `CardReading.cs` passes "infect" over as a keyword it does
not know, and nothing in the bot reads a player's poison counters, so it scores an infect creature's damage (Inkmoth
Nexus, Blightsteel Colossus) as life lost, races and blocks as if it were, and does not see a player near ten poison. A fix
reads infect into `Keywords` and gives the bot's damage and race arithmetic a poison clock beside the life one. The web
client shows poison on the player rail and the language-model client in its text, so a person and the model are not
affected.

**Scenarios.** none: a bot's judgement is not a rules outcome; a bot-lab duel with infect decks would show it.

### KI-51: A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Exercised; the reading is the CR's grammar, no ruling | engine | 2026-10-17, Magus of the Moon |

**Notes.** CR 305.7: a land whose subtype an effect sets "loses all abilities generated from its rules text, its old land types,
and any copiable effects affecting that land". The engine reads the sentence as naming the three sources of the abilities the land
loses - its rules text, its old land types (their mana abilities, CR 305.6) and any copiable effects (a copied card's abilities
and keywords, a copy's "except it has ...") - so `Chars::set_land_subtypes` takes, in layer 4, every ability the land has by
then, and what else a copy effect made of it (the name, supertypes, card types, colors, power and toughness of what it copies,
CR 707.2) stays, as layer 1 applied it. The sentence's first clause already says the land "no longer has its old land type", so
"loses ... its old land types" read as a list of things lost would repeat it; read as sources of abilities it does not. The
other reading - the copy effect itself is lost - has the land stop being what made it a land at all in the case of a Clone
copying a Dryad Arbor, and no ruling of Blood Moon, Magus of the Moon, Vesuva or Thespian's Stage says so; the judges' articles
consulted (Blood Moon, Progenitor of Dependency; a 2013 Judge Forum thread on Thespian's Stage under Blood Moon, which predates
the "copiable effects" wording and kept the Stage's ability) settle nothing either way. A ruling that the copy effect goes
entirely would start the land's characteristics from its own card when `layers::land_text_lost` holds it.

**Scenarios.** vesuva-copying-dark-depths-under-blood-moon-is-a-legendary-mountain-named-dark-depths-with-no-abilities,
thespians-stage-copying-urborg-under-blood-moon-keeps-the-name-and-loses-the-ability-it-kept,
clone-copying-dryad-arbor-under-blood-moon-is-a-green-dryad-creature-mountain: they assert this reading and pass.

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
  of rounds in - rather than at its second round. Past `KEPT` questions in one step the watch starts over, and finds the
  loop again within two rounds.
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
| Low | Open | client | 2026-10-17, adding Lord of Atlantis |

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
| Low | Open | client | 2026-10-17, review of banding |

**Notes.** The engine is right (CR 702.22h-k). The tactical bot's blocks and attacks (`TacticalPlay.Combat.cs`, `Combat`) are
judged pair by pair: it does not know that blocking one member of a band blocks all of it, nor that the opponent divides its
blocker's damage among the band (702.22k) or its attacker's damage among banding blockers (702.22j). Its band choice and the
divisions it makes are on purpose (`TacticalPlay.Choices.cs`, the engine's sparing suggestion).

**Scenarios.** none: a bot's judgement is not a rules outcome.

### KI-63: Control (layer 2) is not ordered by the layer pass

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-17, review of the control-changing static abilities; restated 2026-10-21 with the layer pass |

**Notes.** When control is settled is right: at every event and added effect and at the end of each instruction
(`ops::settle_at_once`, 0e36d26bc). How it is settled is not CR 613.8's: `ops::settle_controller` gives each permanent to the
latest control-changing effect that applies to it, judging a control effect's reach on the object as layer 1 left it, and
`ops::settle_control` repeats that until nothing changes. The rules order layer 2's effects like any layer's: one control
effect depending on another (applying the other changes what it applies to, or whom it gives control to) waits for it, a
dependency loop applies in timestamp order (613.8b; RX-3), and a control effect's reach is judged on the characteristics
as they stand at layer 2. The fix: layer 2 in the layer pass, its order writing the stored controller.

**Scenarios.** None that reach the difference.

### KI-64: Control-changing effects that decide one another are settled by a capped fixpoint

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix queued | engine | 2026-10-17, review of the control-changing static abilities |

**Notes.** `ops::settle_control` works the controllers out again while any changes, at most once per permanent, since an
Aura's controller is whom it gives control to (gain control of a Mind Control, and the creature it enchants follows). That is
a fixpoint, not CR 613.8's order: where two control-changing effects each decide the other (a dependency loop, 613.8b), the
rounds can alternate until the bound stops them on an arbitrary answer, where the rules apply the loop in timestamp order.
No card in content can take control of an Aura that gives control. Fixed with KI-63, by layer 2 in the layer pass.

**Scenarios.** None.

### KI-78: The tactical bot attacks a player whom protection keeps from being dealt damage

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-18, Absolute Virtue |

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

### KI-99: A spell or ability with more than 2^32 targets, or a question of more than 2^31 candidates

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Won't fix | Won't fix: beyond practical scale | engine / host | 2026-10-21, widening the target counts past 255 |

**Notes.** A stack item counts its targets in a `u32` (`StackItem::target_count`), and a target question counts its
candidates in a `u32` (`RequestKind::ChooseTarget::count`), which the host sends as the protocol's `int`. Each stops at its
bound rather than wrap (`cantrip_state::count_u32`, the host's `wire`): a list of more than 2^32 targets keeps the first 2^32,
a question of more candidates asks for at most 2^32, and the host sends a count past 2^31 - 1 as 2^31 - 1, so the client may
offer fewer than the engine would take. Both are tested at their bounds. Play cannot reach them: every target and every
candidate is a different object or player (CR 115.3), so each needs an object of its own in memory - 2^31 objects is
hundreds of gigabytes - and a player chooses each target by hand. Tokens that double each round reach 2^16 in sixteen rounds,
well within these limits; 2^31 would take thirty-one rounds of doubling in a timed game, and the memory above. No card needs
it. The target arena where an item's targets past the first 16 are kept is indexed by a `usize`, with no bound of its own; the
share arena where the shares of a divided total are kept (`GameState::share_arena`) is named by a `u32`, and past 2^32 shares
recorded in one game an item keeps none, so its targets are dealt or given nothing - 2^32 shares is 16 gigabytes of them,
each chosen by hand.

### KI-100: A choice, search or sacrifice of more than 2^32 cards

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Won't fix | Won't fix: beyond practical scale | engine / mechanics / cards / host | 2026-10-21, widening the card counts past 255 |

**Notes.** A question to choose cards counts them in a `u32` (`RequestKind::ChooseCards::count`, with "any number" its own
flag, `any_number`), as do a library search's and a find's counts (`SearchLibrary`, `Find`), craft's materials, and the
counts cards work out from lists of cards or permanents (Balance, Settle the Wreckage, Winds of Abandon, Liliana of the
Veil). A count made from a list's length stops at `u32::MAX` rather than wrap (`u32::try_from(..).unwrap_or(u32::MAX)`, as
`cantrip_state::count_u32` does), so a list of more than 2^32 cards asks for 2^32; the host sends a count past 2^31 - 1 as
2^31 - 1 (`wire`, KI-99). "Sacrifice any number" past 255 is tested (`any_number_sacrificed_is_any_number_past_255`), and an
answer of more cards than asked is refused (`an_answer_of_257_cards_to_choose_one_is_refused`). Play cannot reach the
bound: each card counted is an object in memory, 2^31 of them hundreds of gigabytes, chosen by hand; tokens doubling each
round pass 255 in eight rounds - why the counts are no longer `u8` - but 2^31 takes thirty-one, in a timed game. No card
needs more.

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
