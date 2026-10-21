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
| [KI-26](#ki-26-an-exile-that-a-replacement-sends-elsewhere-is-not-exercised-by-hex-magic-fireglass-mentor-or-esper-origins) | An exile that a replacement sends elsewhere is not exercised by Hex Magic, Fireglass Mentor or Esper Origins | Low | Unexercised |
| [KI-35](#ki-35-the-tactical-bot-counts-no-summoning-sick-creature-as-a-mana-source-under-shang-chi) | The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi | Low | Open |
| [KI-39](#ki-39-a-state-that-is-true-only-in-the-middle-of-a-resolution-with-no-event-misses-its-state-trigger) | A state that is true only in the middle of a resolution, with no event, misses its state trigger | Low | Unexercised |
| [KI-41](#ki-41-a-dependency-through-another-object-is-judged-on-the-object-being-worked-out) | A dependency through another object is judged on the object being worked out | Low | Open |
| [KI-49](#ki-49-the-tactical-bot-does-not-know-infect-or-poison) | The tactical bot does not know infect or poison | Low | Open |
| [KI-51](#ki-51-a-land-whose-subtype-is-set-keeps-the-copiable-values-other-than-abilities-a-copy-effect-gave-it) | A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it | Low | Exercised; reading unconfirmed |
| [KI-54](#ki-54-a-loop-a-player-could-stop-is-shortcut-after-a-fixed-number-of-rounds-not-one-the-players-name) | A loop a player could stop is shortcut after a fixed number of rounds, not one the players name | Low | Open |
| [KI-71](#ki-71-the-tactical-bots-combat-planning-ignores-bands) | The tactical bot's combat planning ignores bands | Low | Open |
| [KI-61](#ki-61-a-land-whose-own-layer-4-ability-and-a-subtype-setting-ability-depend-on-each-other-is-judged-by-a-capped-fixpoint) | A land whose own layer-4 ability and a subtype-setting ability depend on each other is judged by a capped fixpoint | Pri-1 | Fix under way |
| [KI-76](#ki-76-a-fused-cast-is-not-among-the-ways-an-effect-can-cast-a-split-card-from-a-hand) | A fused cast is not among the ways an effect can cast a split card from a hand | Low | Unexercised |
| [KI-78](#ki-78-the-tactical-bot-attacks-a-player-whom-protection-keeps-from-being-dealt-damage) | The tactical bot attacks a player whom protection keeps from being dealt damage | Low | Open |
| [KI-98](#ki-98-a-station-permanent-lists-its-reached-striations-keyword-again-as-granted) | A station permanent lists its reached striation's keyword again, as granted | Pri-2 | Open |
| [KI-99](#ki-99-a-spell-or-ability-with-more-than-232-targets-or-a-question-of-more-than-231-candidates) | A spell or ability with more than 2^32 targets, or a question of more than 2^31 candidates | Won't fix | Won't fix: beyond practical scale |

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
| A fused split spell no longer allowed once its proposal is complete (CR 601.6: `actions::cast` asks `CardDef::spell_rules` again of the spell on the stack), its mana value the sum of its halves' | the check before the cast (every Gaddock Teeg scenario); a back face cast transformed, whose spell has its front face's mana value (CR 712.8c), in a Rust test (`transformed_cast_tests.rs`) | Fuse (CR 702.102) is not built: `CardDef` has no fuse, so no card can be cast fused (KI-76 is its effect-cast half). |

**Scenarios.** None, by definition.

### KI-26: An exile that a replacement sends elsewhere is not exercised by Hex Magic, Fireglass Mentor or Esper Origins

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | card | 2026-10-16, review of Hex Magic |

**Notes.** Hex Magic exiles the whole hand, then draws "that many" cards and lets the player play "cards exiled this way"; Fireglass
Mentor exiles two cards and lets the player choose one "of them". Both keep only the cards that really reached exile
(`g.obj(card).zone.is(ZoneKind::Exile)`), as a replacement effect could send one elsewhere (CR 614.6, 406.2), and draw or grant
play for those alone. No card in the engine replaces an exile from a hand or a library today, so the false arm of that guard
(hex_magic.rs, fireglass_mentor.rs) runs in no scenario. Esper Origins likewise exiles itself as it resolves and puts onto the
battlefield transformed only a card that reached exile ("exile it, then put it onto the battlefield transformed", CR 608.2c,
712.14a); the false arm of that guard (esper_origins.rs, `effect`) is the one line of the card no scenario runs, and no card
replaces an exile from the stack. The day such a card exists each card needs a scenario, and this entry is deleted.

**Scenarios.** None yet.

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

### KI-39: A state that is true only in the middle of a resolution, with no event, misses its state trigger

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, review of the state triggers |

**Notes.** CR 603.8: a state trigger "triggers as soon as the game state matches the condition". The engine asks a state
trigger's condition after every event (`settle::consider`, inside `collect_batch`) and each time a player would receive
priority (`settle::state_triggers`). A state that becomes true and false again within one resolution through changes that
record no event - a continuous effect starting and ending, a characteristic changing in the layers - is never seen, so the
ability doesn't trigger where the rules say it has. No card in content reaches it: Mazemind Tome's state changes only by
counters, which are events. A fix asks the conditions after every change to the game state, or makes every such change
an event.

**Scenarios.** None can be written with the cards in content.

### KI-41: A dependency through another object is judged on the object being worked out

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | core (layers) | 2026-10-16, the dependency system (`layers::by_dependency`, CR 613.8) |

**Notes.** CR 613.8a asks whether applying one effect would change what another applies to, which is a set of objects.
The layers work out one object at a time, so `layers::ready` asks it of that object: whether applying the other changes
whether the first applies to it, now or once the other's own prerequisites have applied (which covers a chain such as
"creatures are artifacts" after "lands are creatures"). An effect that depends on another only through some other object
- the other changes the first's reach on that object and not on this one - is applied to this one in timestamp order. The
result differs only where the two do not commute on this object (one sets what the other adds), and no two cards the
engine has meet that way: every multi-object static ability it has adds types, abilities or power, and the resolved
effects that set types (Oko's Elk) name their object. A fix asks the question over every object the first could affect,
at the cost of their characteristics part-way through the layer.

**Scenarios.** None: no pair of implemented cards reaches it.

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

### KI-54: A loop a player could stop is shortcut after a fixed number of rounds, not one the players name

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | engine / client | 2026-10-17, CR 104.4b loop detection; narrowed by the loop hardening |

**Notes.** `loops.rs` makes every loop end (its module comment is the design): a loop of mandatory actions is a draw at
once when a position comes round with nothing asking the history in between, and otherwise after `ROUNDS` rounds of its
shape with no amount heading for an end; a loop its players keep going by choice goes round `ROUNDS_TO_CHOOSE` times and
then the player CR 732.3 names must choose otherwise; one whose players only pass or decline is a draw after `ROUNDS`
(CR 732.5, 732.4). What is still not the rules' procedure:
- No shortcut is proposed (CR 732.2a-c): no player names a number of iterations, and none can shorten one. The engine takes
  `ROUNDS_TO_CHOOSE` as the number every player chose; a human who wants more rounds (mana past 32 rounds of an infinite
  mana loop) can't have them, and one who wants fewer stops by choosing otherwise, as the rules allow. A fix is a shortcut
  proposal the client offers (a number of iterations) and the bots answer.
- The player made to choose otherwise is chosen as CR 732.3 chooses, but the engine applies it to every loop of choices
  going nowhere - also one whose amounts rise each round (mana floated), where no position is "reached multiple times" - and
  for the rest of the step. What is made to differ is every answer they gave to the same question in the same shape (a
  target, card, player or mode chosen alone, a priority action - any activation of that ability, whatever its targets - or a
  "may" taken): more than "a different game choice", though never anything but choosing less or another of the same.
- A pass or a decline is never refused: a loop whose only choice is a "may" declined each time round (Fecundity's draw) is a
  draw after `ROUNDS`. CR 732.5 would let the player be held to that action, a loop object's; the engine's policy is not to,
  as taking it seldom stops the loop and may only harm them.
- A choice of several targets or cards at once, an order of triggers, a number, and a "may" asked in a staged ability
  (`Game::ask_then`) can't be refused, so a loop going round through them is treated as one of passes: a draw after
  `ROUNDS`. Where the other order or choice would have ended the loop, that is a draw the rules don't make (CR 104.4b's last
  sentence): the price of ending the loop at all.
- A loop whose rounds read the history (a card's "this turn" question each time round) is found only by its shape, after
  `BUSY_STEP` events and `ROUNDS` rounds, not at its second round: the right result, later. `ROUNDS` (256) is taken to be past
  any number a card counts the history or an amount to.
- A loop that never comes back to a shape - its zones growing, a token or an effect more each time round - is ended only by
  `FORCED_LIMIT` (a run of mandatory actions: a draw if no life total, poison, library, the stack or the board moved towards
  an end in its second half), or in a step of choices by `STEP_LIMIT` (players may then only pass and decline, and every
  answer counts as passing) and at last by twice `STEP_LIMIT` events in the step, a draw by the engine's judgement and no
  rule's - each slower as the zones grow. An amount going down from far away (a million life lost one at a time) is played
  out to its end.
- Cost: in a step being watched every question clones and journals the watch (its kept questions and answers), so a loop
  that runs to `ROUNDS` costs seconds, and a minute or more under `CANTRIP_VERIFY` (the growth and Fecundity draw
  scenarios). A fix keeps the questions in smaller journal entries.
- A mana ability whose whole cost is {T} is treated as no choice when nothing more mana could pay for is castable, even when
  its effect does more than add mana (a painland's damage).

**Scenarios.**
- [`scenarios/oblivion-ring-loop-a-player-can-break-with-an-instant-is-not-a-draw.cantrip`](../scenarios/oblivion-ring-loop-a-player-can-break-with-an-instant-is-not-a-draw.cantrip)
  and [`scenarios/worldgorger-dragon-loop-ends-when-animate-dead-chooses-another-creature-card.cantrip`](../scenarios/worldgorger-dragon-loop-ends-when-animate-dead-chooses-another-creature-card.cantrip):
  loops with a choice, played round by hand until the script breaks them.
- [`scenarios/oblivion-ring-loop-a-player-never-breaks-with-the-instant-they-hold-is-a-draw.cantrip`](../scenarios/oblivion-ring-loop-a-player-never-breaks-with-the-instant-they-hold-is-a-draw.cantrip),
  [`scenarios/worldgorger-dragon-loop-whose-player-keeps-choosing-the-dragon-must-choose-the-other-creature-card.cantrip`](../scenarios/worldgorger-dragon-loop-whose-player-keeps-choosing-the-dragon-must-choose-the-other-creature-card.cantrip)
  and [`scenarios/worldgorger-dragon-loop-whose-player-always-declines-fecundity-is-a-draw.cantrip`](../scenarios/worldgorger-dragon-loop-whose-player-always-declines-fecundity-is-a-draw.cantrip):
  the fixed count at work (not marked: they show rules-correct outcomes).

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

### KI-61: A land whose own layer-4 ability and a subtype-setting ability depend on each other is judged by a capped fixpoint

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Pri-1 | Fix under way | engine | 2026-10-17, review of the CR 614.12 change; narrowed 2026-10-21 |

**Notes.** `layers::land_text_lost` finds the lands whose rules text a land-subtype-setting ability takes (CR 305.7) by
judging each land with every layer-4 effect but the setting abilities, then again with the layer-4 abilities of the lands
found so far removed, until the set stops changing - capped at four rounds. That is exact whenever the set settles. It is
not in a dependency loop (CR 613.8b) between a land's own layer-4 ability and a setting ability keyed on what it adds: a
land L with "each land is an X in addition" and an ability "All Xs are Ys" depend on each other (each changes the other's
existence or what it applies to, CR 613.8a), so they apply in timestamp order - L's ability may apply before the setting
ability takes L's text, which the fixpoint never models, and the rounds may alternate until the cap stops on a wrong set.
No printed card in content reaches it. The fix under way decides layer 4's order once, on the pair-based dependency
judgment of KI-41 (`layers::through_others`, the `layer_dependencies` cache), with loops applied in timestamp order, and
has a test card for each loop case; the cap goes with it.

**Scenarios.** None; a Rust test with test cards comes with the fix.

### KI-76: A fused cast is not among the ways an effect can cast a split card from a hand

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-18, review of Buster Sword |

**Notes.** A fuse split card in a hand may be cast with both halves at once (CR 702.102a); its mana value on the stack is the
sum of both halves' (CR 708.4, 702.102c). `Game::cast_ways_during_resolution` offers a split card's halves one at a time and no fused
way, so an effect that casts a spell from a hand with a mana value limit (Buster Sword, Lady Octopus, Omniscience-style free casts)
never offers the fused cast of a fuse card, and cannot judge its combined mana value against the limit. When fuse is built, the
function must offer a fused way for a fuse card in a hand, with a mana value equal to the sum of both halves. No fuse card is in
content yet.

**Scenarios.** None yet: the first fuse card to be added (Turn // Burn) reaches it through Buster Sword.

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
it. The target arena where an item's targets past the first 16 are kept is indexed by a `usize`, with no bound of its own.
