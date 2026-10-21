# Known issues

Problems being tracked in the client, the tooling and the harness, and engine paths no scenario runs yet, in one place.
A scenario that shows a bug is still committed with a `# Known bug:` line (`scenarios/README.md`, "Known bugs"); this
file is where that bug is described, ranked and followed. When it is fixed, delete the entry in the fixing commit (the
history keeps it).

**This file never accepts a deviation from the game's rules** (`CLAUDE.md`, "No rules deviations"). A rules deviation
found in review blocks the change that has it. One found in existing code is fixed, or reported to a human at once; it
appears here only while its fix is under way, marked as such, and is never left to wait for "a card that needs it". A
path no scenario runs gets a test - a Rust test with test-only cards when no printed card reaches it - rather than an
entry. The only allowed deviations are the human decisions in `docs/rules-exceptions.md`.

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
| [KI-25](#ki-25-a-spacecrafts-power-and-toughness-box-is-not-drawn-before-it-is-a-creature) | A Spacecraft's power and toughness box is not drawn before it is a creature | Low | Open |
| [KI-26](#ki-26-an-exile-that-a-replacement-sends-elsewhere-is-not-exercised-by-hex-magic-fireglass-mentor-or-esper-origins) | An exile that a replacement sends elsewhere is not exercised by Hex Magic, Fireglass Mentor or Esper Origins | Low | Unexercised |
| [KI-34](#ki-34-a-skip-effects-source-that-has-left-may-go-unnamed-in-the-turn-would-begin-prompt) | A skip effect's source that has left may go unnamed in the "Your turn would begin" prompt | Low | Open |
| [KI-35](#ki-35-the-tactical-bot-counts-no-summoning-sick-creature-as-a-mana-source-under-shang-chi) | The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi | Low | Open |
| [KI-39](#ki-39-a-state-that-is-true-only-in-the-middle-of-a-resolution-with-no-event-misses-its-state-trigger) | A state that is true only in the middle of a resolution, with no event, misses its state trigger | Low | Unexercised |
| [KI-10](#ki-10-a-lone-action-whose-cost-depends-on-its-target-shows-only-the-printed-cost-until-the-payment) | A lone action whose cost depends on its target shows only the printed cost until the payment | Low | Open |
| [KI-41](#ki-41-a-dependency-through-another-object-is-judged-on-the-object-being-worked-out) | A dependency through another object is judged on the object being worked out | Low | Open |
| [KI-49](#ki-49-the-tactical-bot-does-not-know-infect-or-poison) | The tactical bot does not know infect or poison | Low | Open |
| [KI-51](#ki-51-a-land-whose-subtype-is-set-keeps-the-copiable-values-other-than-abilities-a-copy-effect-gave-it) | A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it | Low | Exercised; reading unconfirmed |
| [KI-54](#ki-54-a-loop-a-player-could-stop-is-shortcut-after-a-fixed-number-of-rounds-not-one-the-players-name) | A loop a player could stop is shortcut after a fixed number of rounds, not one the players name | Low | Open |
| [KI-56](#ki-56-a-counter-ban-judges-an-entering-permanent-as-the-object-it-was-before-it-moved) | A counter ban judges an entering permanent as the object it was before it moved | Low | Unexercised |
| [KI-61](#ki-61-limits-of-judging-a-permanent-as-it-would-exist-and-a-land-as-layer-4-leaves-it) | Limits of judging a permanent as it would exist, and a land as layer 4 leaves it | Low | Unexercised |
| [KI-63](#ki-63-control-from-a-static-ability-or-a-lapsed-condition-is-settled-only-as-state-based-actions-are-checked) | Control from a static ability, or a lapsed condition, is settled only as state-based actions are checked | Low | Unexercised |
| [KI-64](#ki-64-no-card-in-content-lets-one-control-changing-effect-decide-another) | No card in content lets one control-changing effect decide another | Low | Unexercised |
| [KI-71](#ki-71-the-tactical-bots-combat-planning-ignores-bands) | The tactical bot's combat planning ignores bands | Low | Open |
| [KI-76](#ki-76-a-fused-cast-is-not-among-the-ways-an-effect-can-cast-a-split-card-from-a-hand) | A fused cast is not among the ways an effect can cast a split card from a hand | Low | Unexercised |
| [KI-78](#ki-78-the-tactical-bot-attacks-a-player-whom-protection-keeps-from-being-dealt-damage) | The tactical bot attacks a player whom protection keeps from being dealt damage | Low | Open |

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

### KI-25: A Spacecraft's power and toughness box is not drawn before it is a creature

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-16, station (Wurmwall Sweeper) |

**Notes.** A station card's [P/T] box belongs to its striation (CR 721.2b): the permanent has that base power and
toughness only while it has N or more charge counters, and a station card has none in any other zone (CR 721.2c). So its
content file prints no `power`/`toughness`, and the striation's static gives them (`station::base_pt`). The page draws the
2/2 once the Sweeper is a creature, but a player looking at it in hand, or on the battlefield below the threshold, reads only
"It's an artifact creature at 4+" and "4+ | Flying" - not what size it will be. A fix carries the box to the client as
presentation (a `stationPt` beside the Oracle text, drawn in the striation), never as a characteristic.

**Scenarios.**
- [`scenarios/wurmwall-sweeper-is-a-flying-artifact-creature-only-while-it-has-four-charge-counters.cantrip`](../scenarios/wurmwall-sweeper-is-a-flying-artifact-creature-only-while-it-has-four-charge-counters.cantrip) - the engine side (no power below four, 2/2 at four).

### KI-10: A lone action whose cost depends on its target shows only the printed cost until the payment

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-16, review of target-aware costs |

**Notes.** The total cost of a spell or ability is determined once its target is announced (CR 601.2c, 601.2f, 602.2b).
The host words what an activation or a cast costs with each candidate into the action's description ("Equip {2} (costs
{0} targeting Dwarven Mauler, {2} targeting Ornithopter)", `prompt.rs` `costs_by_target`), which the card menu shows when a
card has two or more actions. A card with a single action (Dragonfire Blade's equip, Ajani's Response in hand) goes
straight to its targets when clicked, with no menu row: the targeting caption ("Target for Dragonfire Blade") does not say
what each target would cost, and the real total first shows at the payment ("{1} of {1} still to pay"), where the player
may still cancel. Targets the cost could not be paid for are not offered. A fix shows the description in the targeting
caption, or the cost on each candidate as it is pointed at.

**Scenarios.**
- [`scenarios/dragonfire-blade-equip-shows-its-reduced-cost-and-the-granted-hexproof-in-the-browser.cantrip`](../scenarios/dragonfire-blade-equip-shows-its-reduced-cost-and-the-granted-hexproof-in-the-browser.cantrip) - the menu row of a two-action card (Shadowspear) and the payment of a one-action card (the Blade).

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

### KI-34: A skip effect's source that has left may go unnamed in the "Your turn would begin" prompt

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-16, review of Ral Zarek, Guest Lecturer |

**Notes.** When an effect that makes a player skip a turn (`EffectKind::SkipNextTurn`) and a "you may skip that turn" (Time
Vault) both apply, the player chooses which applies (CR 616.1, `turn.rs::skip_turn`), and the prompt names each effect by its
source. The skip effect's source is usually gone by then (Ral Zarek's −7 from 7 loyalty puts Ral into the graveyard,
CR 704.5i), and `prompt.rs::source_handle` names a gone object only through a handle the seat's view already has. A seat
that never saw the source (a reconnect with a fresh handle space) gets no handle, and its button reads "An effect: you
skip this turn". The choice is still right; only the name is missing. A fix carries the source's name with the prompt.

**Scenarios.**
- [`scenarios/ral-zarek-guest-lecturer-skipped-turn-and-time-vault-in-the-browser.cantrip`](../scenarios/ral-zarek-guest-lecturer-skipped-turn-and-time-vault-in-the-browser.cantrip) -
  the named case, the page having seen Ral; the unnamed one needs a reconnect, which no scenario does.

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

### KI-56: A counter ban judges an entering permanent as the object it was before it moved

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, Melira, Sylvok Outcast |

**Notes.** `CardDef::counter_bans` (Melira's "creatures you control can't have -1/-1 counters put on them") is asked of a
permanent entering with counters (CR 122.6) with the player it enters under. Its characteristics should be those of the
permanent as it would exist on the battlefield (CR 614.12): `ops::judged_entering` now answers `Query::chars` of the entering
object so while its counters are judged, for Solemnity's ban (artifacts, creatures, enchantments, lands) and the replacements
of +1/+1 counters keyed to the permanent's types (`cantrip_mechanics::replacements::plus_one_counters_applies`: Hardened
Scales, Corpsejack Menace, Ozolith, Michelangelo) as well, which should cover a card entering as a copy of a creature or a
noncreature card entering as a creature; but no scenario reaches either yet: no implemented effect returns a noncreature card
with a -1/-1 counter, and a persisting Clone is a creature card. Moonshadow, a persisting Kitchen Finks, a Walking Ballista, a
Saga, Dark Depths and a planeswalker entering as themselves are judged rightly.

**Scenarios.** none yet.

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

### KI-61: Limits of judging a permanent as it would exist, and a land as layer 4 leaves it

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, review of the CR 614.12 change |

**Notes.** `ops::as_it_would_exist` (CR 614.12) and `layers::land_text_lost` (CR 305.7) leave these cases, none of which an
implemented card reaches:
- `ops::judged_entering` works the entering permanent out once, before its zone-change replacements apply, so one that changes
  how it enters ("enters under your control instead") is not taken into account by those after it (614.12: "taking into
  account replacement effects that have already modified how it enters"), and one that sends it elsewhere (Containment Priest)
  leaves the rest still judging a permanent.
- A permanent leaving at the same moment as one enters (`sacrifice_and_return`, Goblin Welder) is still on the battlefield
  while the entering one is judged, so its static abilities count among those that would apply to it.
- The trial object does not carry its `tapped`, a Room's cast door (CR 709.5d) or a battle's protector, and a token copy of a
  permanent spell is judged without the text change `carry_effects` gives it.
- `prepare_entry`'s guard (`layers::abilities_may_be_lost`) looks at the battlefield only: a permanent whose own static
  ability would take its own abilities or set its own land subtype is not asked.
- A replacement of the entering object's own that functions from every zone (`own_everywhere`) is read off its card where it
  is, while `Query::lost_all_abilities` of it answers as it would exist.
- `land_text_lost` judges a land-subtype-setting ability on the object as layer 4 leaves it. For an ability keyed on the subtype
  it overwrites ("All Mountains are Plains": Conversion, Glaciers) that misses the lands it applied to; in a dependency loop
  (CR 613.8b) the ability may have applied before an effect that changed the object's land-ness; and while the question is
  being answered (`finding`), the layer-4 abilities of lands that lose their text still apply. A fix records which setting
  abilities applied to the object as its layer 4 is worked out.

**Scenarios.** none: no implemented card reaches them.
### KI-63: Control from a static ability, or a lapsed condition, is settled only as state-based actions are checked

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, review of the control-changing static abilities (Mind Control, Old Man of the Sea) |

**Notes.** A resolved control effect changes the controller as it is created (`Game::add_effect`), and the untap step settles
control after untapping (an Old Man of the Sea untapped there gives its creature back before the upkeep's triggers). But a
static control-changing ability that starts or stops applying (`control_changing`: an Aura attached, moved or gone) and a
`Duration::AsLongAs` condition that turns false partway through a resolution are worked out by `ops::settle_control` and
`ops::end_lapsed_conditions` only when the state-based actions are next checked (CR 613.1b and 611.2b say at once). Anything
read in between - later in the same resolution, or a trigger looking at the controller at the event - sees the old
controller. No card in content reads a controller in that window that a scenario reaches.

### KI-64: No card in content lets one control-changing effect decide another

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, review of the control-changing static abilities |

**Notes.** `ops::settle_control` works the controllers out again while any changes, since an Aura's controller is whom it
gives control to (gain control of a Mind Control, and the creature it enchants follows). No card in content can take control
of an Aura that gives control: Opalescence leaves Auras alone, and the stealing cards target creatures. The repeat has no
scenario.

### KI-71: The tactical bot's combat planning ignores bands

| Priority | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-17, review of banding |

**Notes.** The engine is right (CR 702.22h-k). The tactical bot's blocks and attacks (`TacticalPlay.Combat.cs`, `Combat`) are
judged pair by pair: it does not know that blocking one member of a band blocks all of it, nor that the opponent divides its
blocker's damage among the band (702.22k) or its attacker's damage among banding blockers (702.22j). Its band choice and the
divisions it makes are on purpose (`TacticalPlay.Choices.cs`, the engine's sparing suggestion).

**Scenarios.** none: a bot's judgement is not a rules outcome.

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
