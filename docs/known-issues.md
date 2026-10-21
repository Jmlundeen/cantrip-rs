# Known issues

Everything known to be wrong, suspected to be wrong, or unproven in the engine, the client and the harness, in one
place. A scenario that shows a bug is still committed with a `# Known bug:` line (`scenarios/README.md`, "Known bugs");
this file is where that bug is described, ranked and followed. Add an entry when a bug, a suspicion or an unexercised
path is found and not fixed in the same change; when it is fixed, delete the entry in the fixing commit (the history
keeps it).

## Severity

| Severity | Meaning |
| --- | --- |
| Critical | A crash, a hang, a desync between engine and client, or a game that cannot go on. |
| High | A wrong game result in ordinary play: a rule misapplied on a common card or interaction. |
| Medium | A wrong result in a rare interaction, or a choice the client cannot offer or show. |
| Low | Cosmetic, a harness or tooling gap, a suspected bug not yet confirmed, or a path no scenario runs. |

## Entry template

```

### KI-<n>: <one-line description of what is wrong>

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open / Suspected / Unexercised | engine / mechanics / card / client / harness | YYYY-MM-DD, how |

**Notes.** What happens, what the rules require (CR numbers, rulings), where in the code, and what a fix needs.

**Scenarios.**
- [`scenarios/<name>.cantrip`](../scenarios/<name>.cantrip) - what it shows (marked `# Known bug`), or "none yet" and
  which implemented card could reach it.
```

## Summary

| ID | Issue | Severity | Status |
| --- | --- | --- | --- |
| [KI-2](#ki-2-a-harmonize-cost-reduction-cannot-be-checked-in-the-browser) | A harmonize cost reduction cannot be checked in the browser | Low | Open |
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
| [KI-75](#ki-75-words-of-winds-return-a-permanent-question-may-be-captioned-as-a-cost) | Words of Wind's "return a permanent" question may be captioned as a cost | Low | Suspected |
| [KI-76](#ki-76-a-fused-cast-is-not-among-the-ways-an-effect-can-cast-a-split-card-from-a-hand) | A fused cast is not among the ways an effect can cast a split card from a hand | Low | Unexercised |
| [KI-78](#ki-78-the-tactical-bot-attacks-a-player-whom-protection-keeps-from-being-dealt-damage) | The tactical bot attacks a player whom protection keeps from being dealt damage | Low | Open |
| [KI-83](#ki-83-teferis-protection-in-the-browser-shows-an-empty-battlefield) | Teferi's Protection in the browser shows an empty battlefield | Low | Open |
| [KI-84](#ki-84-trinisphere-and-force-of-wills-alternative-cost-in-the-browser-show-the-wrong-menu) | Trinisphere and Force of Will's alternative cost in the browser show the wrong menu | Low | Open |
| [KI-85](#ki-85-a-scenario-table-test-sometimes-fails-closing-a-connection) | A scenario table test sometimes fails closing a connection | Low | Open |
| [KI-93](#ki-93-the-browser-shows-no-phased-out-permanents) | The browser shows no phased-out permanents | Medium | Open |
| [KI-94](#ki-94-a-browser-scenarios-clientmenu-check-reads-a-menu-of-a-card-long-gone) | A browser scenario's `Client.Menu` check reads a menu of a card long gone | Low | Open |

## Issues

### KI-2: A harmonize cost reduction cannot be checked in the browser

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | harness | 2026-10-15, Wild Ride's browser scenario |

**Notes.** When the mana pool already covers a spell's cost, no payment prompt appears, so a browser scenario has
nothing on the page to read the reduced total from. The engine side is proven by the headless scenarios; the client
shows the harmonize cost on the graveyard pile's note. A fix is a `Client.` key for the cost a pending cast will pay,
or a scenario arranged so the payment prompt appears.

**Scenarios.**
- [`scenarios/wild-ride-in-the-browser-offers-harmonize-from-the-graveyard-and-the-creature-to-tap.cantrip`](../scenarios/wild-ride-in-the-browser-offers-harmonize-from-the-graveyard-and-the-creature-to-tap.cantrip) - the browser
  scenario that plays it, without the reduced total.

### KI-3: Engine paths no scenario runs

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-15/16, fixes in the U-B-R-G Prowess run and the known-bug fixes |

**Notes.** Code written for every card the rule describes, where no implemented card reaches it yet. Each is a
candidate bug until something runs it; when a card that reaches one is implemented, its scenarios cover the path and
the line goes. So does a line whose path a Rust test drives through the real engine with test cards (the tests under
`crates/cantrip-script/tests`, `real_cards` among them with every card the engine has, and the rules crate's own), where
no printed card can reach it.

| Path | Covered instead by | Why no scenario |
| --- | --- | --- |
| The "isn't a mana ability" guards on activation triggers (`triggers::opponent_activates_ability_of_artifact_creature_or_land`, Burning-Tree Shaman's matcher) | nothing | A mana ability's activation is only recorded, never emitted (`EventMask::of`), so no trigger sees one; the guards stay for when it is emitted for the cards that trigger on mana abilities too (Elrond, Moon-Reader, Ceaseless Searblades). |
| The end of an "until" exile (`DelayedWhen::UntilEvent`, Palace Jailer) is never stopped by `trigger_rules` (CR 610.3: the return is a one-shot effect, not a triggered ability) | nothing | The only "until" event is becoming the monarch, which no rule stops. |
| Damage redirected (CR 614.1a) to a battle: `ops::deal_damage_with` judges defense against the recipient the replacement effects leave | Ancient Adamantoise's scenarios (creatures, players, and a planeswalker the damage is redirected away from, `ancient-adamantoise-takes-a-bolt-meant-for-your-planeswalker-which-keeps-its-loyalty`); a Rust test redirecting to a planeswalker (`crates/cantrip-script/tests/damage_redirect.rs`) | No implemented card redirects damage to a battle: Ancient Adamantoise and Palisade Giant redirect to themselves, creatures. |
| A static ability whose condition reads the history (`LayerQuery::history`) judged while the triggers of the event that would satisfy it are collected: `Cx::emit` collects triggers before it records the event (so a trigger sees the history as it was just before it), so a triggered ability that such a static grants ("as long as you've surveilled this turn, this creature has '...'", Darkblade Agent) is not yet there for the very event that turns the condition on. CR 603.2 looks for triggers once the event has happened; whether the granted ability should see that first event is unconfirmed | nothing | Essence Channeler and Shardmage's Rescue grant only keywords; no implemented card grants a triggered ability on a history condition. |
| "For as long as it has a [kind] counter on it" (`Duration::WhileCounter`) ended by damage removing loyalty counters (`ops::end_counter_durations`) | Rust tests: `crates/cantrip-script/tests/counter_duration.rs` (a counter put back after the last came off, an effect begun without one) and `real_cards/counter_duration_ends.rs` (a stun counter removed in place of an untap ending a control effect, counters removed as a cost); Eluge, the Shoreless Sea's Vampire Hexmage scenario (removal through `add_counters`) | No implemented card makes such an effect on a planeswalker, and one that loses its last loyalty counter is put into its owner's graveyard as a state-based action anyway (CR 704.5i). |
| Eluge, the Shoreless Sea's power and toughness as a spell another player than its owner controls, and in a zone where its owner's Islands are counted (the characteristic-defining ability counts its controller's Islands on the battlefield and the stack, its owner's elsewhere); its cost reduction's "land" for a flooded permanent that has stopped being a land | Eluge stolen on the battlefield by Act of Treason (`eluge-stolen-by-act-of-treason-counts-its-new-controllers-islands`) | No implemented card casts another player's Eluge, and nothing in content makes a land stop being a land while it keeps a flood counter. |
| Demonic Pact with only modes that need a target left and no legal target for them (ruling 2015-06-22: the ability is removed from the stack with no effect, CR 603.3c, 700.2) | Gollum, Riddle Master with every mode spent (`gollum-spends-all-three-modes-and-then-riddles-no-more`: the same removal, for lack of modes) | Its targeted modes take any target and target opponent, which are always legal while an opponent is in the game, so no position lacks a target. |
| A spell no longer allowed once its proposal is complete (CR 601.6: `actions::cast` asks `CardDef::spell_rules` again of the spell on the stack, its X announced, and takes the cast back): a fused split spell's combined mana value, or a back face cast transformed, whose mana value is its front face's (CR 202.3b) where the card it is judged as before the cast reads the back face alone | the check before the cast (every Gaddock Teeg scenario) | No implemented spell's qualities that a rule judges change between the offer and the proposal: Gaddock Teeg's {X} clause already forbids every noncreature spell with {X}, no implemented card has fuse, and the one implemented Siege's back face (Lightshield Array) is mana value 2. |

**Scenarios.** None, by definition.

### KI-25: A Spacecraft's power and toughness box is not drawn before it is a creature

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, review of the control-changing static abilities |

**Notes.** `ops::settle_control` works the controllers out again while any changes, since an Aura's controller is whom it
gives control to (gain control of a Mind Control, and the creature it enchants follows). No card in content can take control
of an Aura that gives control: Opalescence leaves Auras alone, and the stealing cards target creatures. The repeat has no
scenario.

### KI-71: The tactical bot's combat planning ignores bands

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-17, review of banding |

**Notes.** The engine is right (CR 702.22h-k). The tactical bot's blocks and attacks (`TacticalPlay.Combat.cs`, `Combat`) are
judged pair by pair: it does not know that blocking one member of a band blocks all of it, nor that the opponent divides its
blocker's damage among the band (702.22k) or its attacker's damage among banding blockers (702.22j). Its band choice and the
divisions it makes are on purpose (`TacticalPlay.Choices.cs`, the engine's sparing suggestion).

**Scenarios.** none: a bot's judgement is not a rules outcome.

### KI-75: Words of Wind's "return a permanent" question may be captioned as a cost

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Suspected | client | 2026-10-17, draw-replacement batch |

**Notes.** Words of Wind's replacement ("each player returns a permanent they control to its owner's hand instead") asks with
`Purpose::ReturnToHand`, which the server's prompt captioning treats as a cost (the karoo lands' "return a land you control ...
to pay for it"). The caption would then say the return pays for something, which it does not. A fix gives the effect's return its
own purpose or caption. Not checked in the browser.

**Scenarios.** none yet.

### KI-76: A fused cast is not among the ways an effect can cast a split card from a hand

| Severity | Status | Area | Found |
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

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client | 2026-10-18, Absolute Virtue |

**Notes.** The engine is right: damage from an opponent's sources to a player with protection from them, or from everything,
is prevented (CR 702.16e, 702.16j, 702.16k: Absolute Virtue, Teferi's Protection, The One Ring), and the host tells both seats
("They have protection from Bot1."). Targeting is the engine's, so the bot never aims a spell at such a player. But its attack
planning (`TacticalPlay.Combat.cs`, `Combat`) scores the damage an unblocked attacker would deal that player as life lost, so it
may attack into a protected player for nothing and leave itself open. A fix tells the bot, in the view, whom each player is
protected from (a field on `PlayerSummaryView` rather than the text), and has the attack plan count no damage to them.

**Scenarios.** none yet.

### KI-83: Teferi's Protection in the browser shows an empty battlefield

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client, browser scenario | 2026-10-19, a full run of ScenarioInBrowserTests |

**Notes.** The browser scenario fails: the page shows an empty battlefield where the phased-out permanents and the
player's facts should be. It failed before the declare-blockers test-driver change of 2026-10-19 too (it fails with that
change switched off), so it is older. Not investigated: it may be the client's drawing of phased-out permanents or the
scenario driver.

**Scenarios.** `teferis-protection-shows-phased-out-permanents-and-its-player-facts-in-the-browser` (browser).

### KI-84: Trinisphere and Force of Will's alternative cost in the browser show the wrong menu

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | client, browser scenario | 2026-10-19, a full run of ScenarioInBrowserTests |

**Notes.** The browser scenario fails at its line 46 with the wrong menu showing. Like KI-83 it predates the
declare-blockers test-driver change of 2026-10-19. Not investigated: the cost display of an alternative cost under
Trinisphere, or the driver's reading of the menu.

**Scenarios.** `trinisphere-cost-of-force-of-wills-alternative-cost-shown-in-the-browser` (browser).

### KI-85: A scenario table test sometimes fails closing a connection

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | server, tests | 2026-10-19, a `tools	est.cmd /noweb` run during the site overhaul |

**Notes.** `ScenarioTableTests.A_client_answering_otherwise_fails_the_scenario_at_the_line` once threw a
NullReferenceException in `GameServer` while a connection was closing, then passed three reruns and the next full run.
A race in closing a table's connections, in code the change being tested did not touch. Not investigated.

**Scenarios.** none; the .NET test above.

### KI-93: The browser shows no phased-out permanents

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | client | 2026-10-20, review of PR #147 (fails the same on a clean main) |

**Notes.** After Teferi's Protection resolves, Bot1's Plains, Mox Pearl, Mox Jet and Ornithopter are phased out
(`Bot1.PhasedOut` passes in the engine), but Bot1's page draws none of them: `Client.Bot1.Battlefield` reads "" where
"Plains; Mox Pearl; Mox Jet; Ornithopter" is expected. Phased-out permanents are treated as though they don't exist
(CR 702.26b) but are not hidden; the players know of them, and the client is meant to keep them in sight with PHASED OUT
at their foot. Either the projection stopped sending them or the table stopped drawing them; not yet bisected.

**Scenarios.** `teferis-protection-shows-phased-out-permanents-and-its-player-facts-in-the-browser` (browser suite;
fails at line 35).

### KI-94: A browser scenario's `Client.Menu` check reads a menu of a card long gone

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | harness / client | 2026-10-20, review of PR #147 (fails the same on a clean main) |

**Notes.** Under Trinisphere, Bot1 casts Force of Will for its alternative cost from its card menu, and the check after it
expects that menu: "Cast Force of Will; Cast Force of Will — Pay 1 life, exile a blue card from your hand (costs {3})".
The page reports instead the five mana abilities of Black Lotus, which was sacrificed on turn 1 and is no longer on the
battlefield. So `Client.Bot1.Menu` returns a stale menu (the first one opened in the game) rather than the one just used.
Whether the page keeps publishing an old menu in `cantripTest` or `ScenarioPlayer` reads the wrong one is not yet known;
the engine side of the cast passes.

**Scenarios.** `trinisphere-cost-of-force-of-wills-alternative-cost-shown-in-the-browser` (browser suite; fails at
line 46).
