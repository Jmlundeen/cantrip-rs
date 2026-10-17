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
| [KI-1](#ki-1-suspend-ignores-a-spell-limit-already-reached) | Suspend ignores a spell limit already reached | Low | Suspected |
| [KI-2](#ki-2-a-harmonize-cost-reduction-cannot-be-checked-in-the-browser) | A harmonize cost reduction cannot be checked in the browser | Low | Open |
| [KI-3](#ki-3-engine-paths-no-scenario-runs) | Engine paths no scenario runs | Low | Unexercised |
| [KI-4](#ki-4-scenarios-waiting-on-cards) | Scenarios waiting on cards the engine does not have | Low | Open |
| [KI-5](#ki-5-a-spell-that-has-left-the-stack-is-judged-by-its-printed-keywords) | A spell that has left the stack is judged by its printed characteristics | Low | Suspected |
| [KI-24](#ki-24-under-bolass-citadel-the-top-of-a-library-is-cast-only-for-life) | Under Bolas's Citadel the top of a library is cast only for life | Medium | Open |
| [KI-25](#ki-25-a-spacecrafts-power-and-toughness-box-is-not-drawn-before-it-is-a-creature) | A Spacecraft's power and toughness box is not drawn before it is a creature | Low | Open |
| [KI-26](#ki-26-an-exile-that-a-replacement-sends-elsewhere-is-not-exercised-by-hex-magic-or-fireglass-mentor) | An exile that a replacement sends elsewhere is not exercised by Hex Magic or Fireglass Mentor | Low | Unexercised |
| [KI-28](#ki-28-noncombat-damage-plus-n-reads-the-controller-of-a-damage-source-that-is-not-on-the-battlefield-or-the-stack) | Noncombat damage "plus N" reads the controller of a damage source that is not on the battlefield or the stack | Low | Suspected |

## Issues

### KI-1: Suspend ignores a spell limit already reached

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Suspected | engine | 2026-10-16, review of the cast-limit fix (`d9509b633`) |

**Notes.** Suspend may be used "if you could begin to cast this card by putting it onto the stack from your hand"
(CR 702.62a). The suspend action's `Timing::AsCast` gate in `crates/cantrip-rules/src/actions.rs` checks
`static_rules::may_cast`, but not the spell limit of Archon of Emeria ("each player can't cast more than one spell each
turn", CR 101.2) or the card's own "cast this spell only" condition, which casting checks through `may_begin_cast` and
`cast_only_if_met`. So a player who has cast a spell this turn under Archon may still suspend Lotus Bloom. Confirm the
reading against the suspend rulings first; if it holds, the gate calls `may_begin_cast` like every cast.

**Scenarios.** None yet. Lotus Bloom (suspend 3, {0}) and Archon of Emeria are both implemented and can reach it,
with a positive control that suspends it as the turn's first action.

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
candidate bug until a scenario runs it; when a card that reaches one is implemented, its scenarios cover the path and
the line goes.

| Path | Covered instead by | Why no scenario |
| --- | --- | --- |
| Retargeting a copy (`ops::retarget`): "up to" slots of a modal spell, each-opponent slots, spliced text (CR 702.47c) and kicked-spell targets | nothing | No modal spell in content has an "up to" mode target, no copyable spell has an each-opponent target, and no scenario copies a spliced or kicked spell. |
| A granted cast of a back face (`GRANTED_FACE_BACK`, `faces_to_cast`) | nothing | No implemented double-faced card has a castable back face. |
| The transformed cast's `cast_only_if_met` check (`could_cast_transformed_during_resolution`) | nothing | Only Berserk has a "cast only" condition, and it has no back face. |
| A "loses all abilities" effect older than an ability granted later (CR 613.1f, 613.7) | Rust test | No implemented card does both. |
| Gift of an extra turn; a copied permanent spell keeping its gift; "whenever you give a gift" | Rust tests, `crates/cantrip-script/tests/gift.rs` | Perch Protection needs phasing; no implemented card copies a gift permanent or has the trigger. |
| Lost abilities at the flash-rules, instant-activation, damage-by-toughness and ascend sites | nothing | No implemented card loses its abilities while using one. |
| The late-targets / divide and MDFC back-face sites of the minimum-targets offering check (CR 601.2c) | nothing | No implemented card reaches them. |
| The "isn't a mana ability" guards on activation triggers (`triggers::opponent_activates_ability_of_artifact_creature_or_land`, Burning-Tree Shaman's matcher) | nothing | A mana ability's activation is only recorded, never emitted (`EventMask::of`), so no trigger sees one; the guards stay for when it is emitted for the cards that trigger on mana abilities too (Elrond, Moon-Reader, Ceaseless Searblades). |
| A rule that stops abilities triggering (`trigger_rules`) on a permanent entering at the same moment as other permanents: "If Torpor Orb and a creature enter at the same time" (Torpor Orb, Doorkeeper Thrull, Hushbringer, Tocatli Honor Guard, Hushwing Gryff rulings) | Tocatli Honor Guard's own entering stopping Enduring Innocence (the newcomer is on the board the rule is read from); Hushbringer dying beside Worldspine Wurm | No implemented effect puts a chosen suppressor onto the battlefield together with another permanent (several creatures put onto the battlefield by one effect still enter one at a time, `engine-design.md`). |
| The end of an "until" exile (`DelayedWhen::UntilEvent`, Palace Jailer) is never stopped by `trigger_rules` (CR 610.3: the return is a one-shot effect, not a triggered ability) | nothing | The only "until" event is becoming the monarch, which no rule stops. |
| More than `MAX_TARGETS` (16) targets for one spell or ability: a stack item holds 16, so a target question's count is cut to the room left (`choose_slots`, `choose_trigger_targets`), where the rules set no limit - a deviation, in place of a panic. Reachable by Miasma Demon (17+ cards discarded and 17+ creatures), Pest Infestation or Batroc with X or kicks of 17+ | nothing | No scenario holds that many cards and targets. |
| Floodpits Drowner targeting itself (`shuffle_away` shuffles one object, `objects.dedup()`) | nothing | Unreachable in today's pool only by accident: its ability needs it untapped ({T}) and carrying a stun counter, but every stun source (its own trigger, Kaito, Baloth Prime, Cryogen Relic) leaves the permanent tapped, and an untap with a stun counter removes the counter instead (CR 122.1g). A card that puts a stun counter on an untapped permanent would reach it. |
| A creature's own "leaves the battlefield" ability under Hushbringer, not triggering when it dies (the rulings) | earthbend's "When this land dies or is put into exile" (stopped as it dies, not as it is exiled); Daretti's emblem | No implemented creature has a leaves-the-battlefield ability of its own. |
| Dedicated Commons entering at the same time as a planeswalker: that planeswalker isn't considered (ruling; `enters` reads the battlefield as it is, so it is not counted) | nothing | No implemented effect puts a land and a planeswalker onto the battlefield together (several permanents put onto the battlefield by one effect still enter one at a time, `engine-design.md`). |
| Torch the Tower's replacement not made when its damage is prevented (`g.damage(..) > 0` false: a permanent "dealt damage by" the spell only if damage was dealt) | nothing | No implemented effect prevents damage and leaves the target legal: protection from red gained in response makes the target illegal, so the spell does not resolve (a different path), and Impractical Joke and Stomp stop prevention rather than cause it. A damage-prevention card (a Fog, a Circle of Protection) would reach it. |
| A face-down permanent turned face up by an effect (`Game::turn_face_up` with the effect's controller as the player recorded in `Event::TurnedFaceUp`), and a manifested instant or sorcery card that would turn face up, revealed and left face down (`ops::turn_face_up`, CR 701.40g, 701.58g) | the special action (Oblivious Bookworm's and Abhorrent Oculus's scenarios), whose own condition refuses a noncreature card | No implemented card turns a permanent face up as an effect. |
| Ward's life prompt for an amount above 60 (`triggers::ward_life_question`: the static `Question`s name the amount for 0 to 60 only, so a bigger one is asked as "Pay the life", without the number; the payment itself is exact) | Raubahn, Bull of Ala Mhigo's ward at 4 | No implemented creature reaches a power above 60. A `Question` with a runtime amount would remove the table. |

| A copy of a spell paid for with mana that made it uncounterable or gives it haste (Cavern of Souls, Delighted Halfling, Arena of Glory) is counterable and gains nothing (`ops::copy_spell` clears `cant_be_countered` and `gains_haste`: no mana was spent on the copy, CR 707.10, 106.6) | nothing | No implemented card copies a creature spell or a legendary spell, the only spells that mana can pay for. |
| Discover dropping a face whose mana value is over N although the card's is not (`discover::discover`'s `ways` filter; CR 701.57a, "if the resulting spell's mana value is less than or equal to N") | Adventurers and split cards with both faces at or under N (Quintorius Kand's scenarios) | No implemented card has an Adventure or castable back face costing more than a front face of 4 or less. |
| A Case's "To solve" resolving for a Case that became solved while it waited (`solved::become_solved` through `Game::solve`, which leaves a solved permanent as it is; CR 603.4's "this Case is not solved" asked again) | nothing | Only a Case's own "To solve" solves it, once per end step: no implemented card copies a triggered ability or solves a Case another way. |
| A Case that loses all its abilities: it stays solved (CR 719.3b) and its "Solved" ability is gone, a play rule under `PlayRule::While` with it (`each_play_rule` skips a permanent that lost its abilities) | nothing | No implemented card takes the abilities of an enchantment (Oko, The Wondrous Wasp and Tishana's Tidebinder reach only artifacts, creatures and planeswalkers). |
| A "Solved" static ability of the layers (`solved::solved_in_layers`, CR 702.169b) | nothing | No implemented Case has one: Case of the Locked Hothouse's is a play rule. |
| The host showing a player the top card of their library while a permanent says they may look at it any time (`PlayRule::LookAtLibraryTop`, `observable`), whether or not it can be played | nothing | No `Client.` key reads a library pile, and the browser driver plays nothing from a library. Bolas's Citadel, Mystic Forge and a solved Case of the Locked Hothouse have it. |
| Damage redirected (CR 614.1a) to a planeswalker or battle, or away from one: `ops::deal_damage_with` judges loyalty and defense against the recipient the replacement effects leave, and a player's protection from everything is ordered with a redirection (`replace::Origin::Protection`, CR 616.1) | Ancient Adamantoise's scenarios (creatures and players) | Ancient Adamantoise only redirects to itself, a creature, and no implemented card gives its controller protection from everything while it is out; a planeswalker of Bot1's hit while it is out would reach the first half. |
| Planetarium of Wan Shi Tong that left the battlefield and came back the same turn after a card was cast with it: "do this only once each turn" is looked for among the controller's casts with the ability of any Planetarium (CR 603.2h: "its source's controller has not yet taken the indicated action"), so the new object (CR 400.7) does not trigger again; keyed on the object it would | nothing | No implemented card blinks or returns an artifact to the battlefield in a turn the Planetarium has already cast with. |
| Damage prevention ordered against damage replacement (CR 616.1: when replacement and prevention effects both apply to one damage event, the affected player or the affected object's controller chooses which applies first). `ops::deal_damage_with` always runs the replacement chain first (`replace::damage`: Artist's Talent's and Torbran-style "deals that much plus 2 instead") and protection's prevention (CR 702.16e) after it; the player is never asked. For replacements that only change the amount, the order cannot change the result - protection prevents all of that source's damage, whatever the amount. It can for a replacement that redirects the damage to another player or permanent, or an "instead" that changes what is dealt to whom: applied first, protection would have prevented the damage before any redirection, so the player's choice decides it. No content card alters `d.target` (or the source) of a damage proposal today. Damage of 0 never reaches the chain, which is CR 120.8 ("replacement effects that would increase the damage ... have no event to replace"), so Artist's Talent's `d.amount > 0` guard is reached only if an earlier replacement in the chain reduces the damage to 0 (CR 615.6) | nothing | No implemented card prevents a fixed amount of damage, redirects damage by a replacement, or replaces damage down to 0. |
| Beyond the Quiet's "Spacecraft" half (`chars.has_subtype(Subtype::Spacecraft)`): exiling a Spacecraft that is not a creature, which the Oracle text requires and the code does | The creature half of the same filter (a crewed Vehicle exiled, an uncrewed one left) | No Spacecraft card exists in content yet, so no scenario can put one on the battlefield. |
| A static "can attack as though it didn't have defender" of a permanent that has lost all its abilities (`static_rules::may_attack_despite_defender` skips it, CR 613.1f), or of one permanent about another creature | Demon Wall's own rule, asked of itself and refusing Bot2's defender (`demon-wall-lets-only-itself-attack-and-an-opponents-wall-attacks-on-its-own-turn`) | Demon Wall is the only card with the rule and it is its own source: losing its abilities loses its defender too. Rolling Stones, High Alert or Ghalta, the Immovable would reach it. |
| A static ability whose condition reads the history (`LayerQuery::history`) judged while the triggers of the event that would satisfy it are collected: `Cx::emit` collects triggers before it records the event (so a trigger sees the history as it was just before it), so a triggered ability that such a static grants ("as long as you've surveilled this turn, this creature has '...'", Darkblade Agent) is not yet there for the very event that turns the condition on. CR 603.2 looks for triggers once the event has happened; whether the granted ability should see that first event is unconfirmed | nothing | Essence Channeler and Shardmage's Rescue grant only keywords; no implemented card grants a triggered ability on a history condition. |
| Ba Sing Se entering at the same time as basic lands: those lands are not counted for "enters tapped unless you control a basic land" (ruling 2025-10-02; `lands::enters_tapped_unless_you_control_basic_land` reads the battlefield as it is) | Abandoned Air Temple-style scenarios with a basic land already in play | No implemented effect puts a land and a basic land onto the battlefield together (several permanents put onto the battlefield by one effect still enter one at a time, `engine-design.md`). |
| Demonic Pact under another controller: the modes it chose before count whoever chose them (ruling 2015-06-22; `history::mode_not_chosen_before` is keyed on the Pact object, not the player) | Two Pacts choosing independently (`demonic-pact-two-pacts-each-have-all-four-modes-to-choose`) | No scenario steals a Demonic Pact between upkeeps. |
| Demonic Pact with only modes that need a target left and no legal target for them (ruling 2015-06-22: the ability is removed from the stack with no effect, CR 603.3c, 700.2) | Gollum, Riddle Master with every mode spent (`gollum-spends-all-three-modes-and-then-riddles-no-more`: the same removal, for lack of modes) | Its targeted modes take any target and target opponent, which are always legal while an opponent is in the game, so no position lacks a target. |

**Scenarios.** None, by definition.

### KI-4: Scenarios waiting on cards

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | cards | `corpus` reports them as "need cards" |

**Notes.** These scenarios name a card no file defines, so they cannot run. Each tests an interaction already
written into the engine that is unproven until the card exists: Diabolic Edict, Quest for Ancient Secrets, Doomed
Traveler, Innocent Blood, Planar Void.

**Scenarios.**
- [`scenarios/planar-void-exiles-a-spell-that-paused-for-its-opponents-choice-as-it-lands-in-the-graveyard.cantrip`](../scenarios/planar-void-exiles-a-spell-that-paused-for-its-opponents-choice-as-it-lands-in-the-graveyard.cantrip)
- [`scenarios/quest-for-ancient-secrets-sees-a-spell-that-paused-for-its-opponents-choice-land-in-the-graveyard.cantrip`](../scenarios/quest-for-ancient-secrets-sees-a-spell-that-paused-for-its-opponents-choice-land-in-the-graveyard.cantrip)
- [`scenarios/quest-for-ancient-secrets-counts-both-an-innocent-blood-sacrifice-and-the-spell-itself.cantrip`](../scenarios/quest-for-ancient-secrets-counts-both-an-innocent-blood-sacrifice-and-the-spell-itself.cantrip)
- [`scenarios/dying-pest-tokens-hand-over-their-trigger-order.cantrip`](../scenarios/dying-pest-tokens-hand-over-their-trigger-order.cantrip)

### KI-5: A spell that has left the stack is judged by its printed keywords

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Suspected | engine | 2026-10-16, review of Momo, Friendly Flier |

**Notes.** `layers::chars_before` computes `spell` from the object's zone being the stack, and applies continuous effects only to
a spell or a permanent on the battlefield. An earlier cast, read through `q.chars(c.spell)` after the spell resolved, was countered or
left the stack (its old object is a card row in another zone or `Gone`), therefore has its printed characteristics, not the ones it
had as it was cast (CR 601.2i, 400.7). Momo, Friendly Flier asks "was an earlier spell this turn a non-Lemur creature spell with
flying" this way: a creature spell given flying (or stripped of it) while it was on the stack is misjudged afterwards. The history's
`Cast` record keeps types, colors and mana value but not subtypes or keywords; recording them there, or keeping last-known
information for a spell that leaves the stack, fixes it. No implemented card grants or removes a keyword on a spell on the stack, so
no scenario can reach it yet.

**Scenarios.** None yet; the first card that gives a creature spell flying on the stack (then a second flier cast under Momo) reaches it.

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


### KI-23: A creature fighting itself (CR 701.14c) is implemented but no scenario can reach it

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, review of Bushwhack |

**Notes.** `ops::fight` makes a creature that fights itself deal damage to itself equal to twice its power (CR 701.14c), and
the two damages of an ordinary fight are one simultaneous `damage_together`. No card in the engine can name one creature for
both fighters (Bushwhack's targets are a creature you control and a creature you don't; Mawloc and Wolverine say "other"), so
only the simultaneous damage is covered, by the fight scenarios. The day a card can fight itself it needs a scenario, and this
entry is deleted.

**Scenarios.** none yet.

### KI-24: Under Bolas's Citadel the top of a library is cast only for life

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine | 2026-10-16, review of the Cases |

**Notes.** A player who controls Bolas's Citadel (`PlayRule::LibraryTop`) and also Mystic Forge or a solved Case of the Locked
Hothouse (`PlayRule::LibraryTopSpells`) may cast the top card either way: for life by the Citadel, or for its mana cost by the
other permission (CR 601.3, 118.9; the Citadel's life payment is only for a spell cast "this way"). `castable` in
`crates/cantrip-rules/src/actions.rs` offers the mana-cost cast only when no permanent gives `LibraryTop`, and the cast decides
life or mana from `land_rules(..).top_for_life` alone, so only the life cast is offered. A fix offers both, each as its own way,
and has the cast pay what that way says.

**Scenarios.** None yet: Bolas's Citadel with Mystic Forge, or with a solved Case of the Locked Hothouse, and a creature on top.

### KI-26: An exile that a replacement sends elsewhere is not exercised by Hex Magic or Fireglass Mentor

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | card | 2026-10-16, review of Hex Magic |

**Notes.** Hex Magic exiles the whole hand, then draws "that many" cards and lets the player play "cards exiled this way"; Fireglass
Mentor exiles two cards and lets the player choose one "of them". Both keep only the cards that really reached exile
(`g.obj(card).zone.is(ZoneKind::Exile)`), as a replacement effect could send one elsewhere (CR 614.6, 406.2), and draw or grant
play for those alone. No card in the engine replaces an exile from a hand or a library today, so the false arm of that guard
(hex_magic.rs, fireglass_mentor.rs) runs in no scenario. The day such a card exists each card needs a scenario, and this entry is
deleted.

**Scenarios.** None yet.

### KI-28: Noncombat damage "plus N" reads the controller of a damage source that is not on the battlefield or the stack

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Suspected | mechanics | 2026-10-16, review of Tomik, Izzet Sparkmage |

**Notes.** `replacements::noncombat_damage_you_deal_to_an_opponent_or_their_permanent` (Tomik, Izzet Sparkmage; Artist's Talent
at level 3) asks `q.controller(d.source)` for "a source you control". A damage source that is a card outside the battlefield and the
stack has no controller (CR 108.4a); for an ability of such a card the controller is the player who controls the ability
(CR 110.2, 113.8), and for the damage of a spell it is the spell's controller. `q.controller` of such a card answers its owner, so
damage dealt by an ability a card has in a graveyard or exile (an activated ability that works from there, a Cycling-style
trigger) is judged by its owner rather than by the player who activated or controls the ability. No card in the engine deals
noncombat damage from such an ability while an opponent controls Tomik or a level 3 Artist's Talent, so nothing shows it. A fix
reads the controller of the stack item that is dealing the damage (the `DamageEvent` would have to carry it) and keeps
`q.controller` for permanents.

**Scenarios.** none yet.

### KI-30: A permanent's name is not part of its last-known information

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | engine | 2026-10-16, review of The End |

**Notes.** `LastKnown` (`crates/cantrip-state/src/object.rs`) records types, subtypes, colors, power, toughness, keywords,
abilities and words but no name, so "the same name as that permanent" asked after it has left the battlefield (CR 608.2h) cannot
use last-known information: a card, copy effect, face-down or transformed state is gone with the old object (CR 400.7). Cards
read `library::object_names` before moving the object (The End, Deadly Cover-Up), which is right for them; a card that must ask
after the move needs the name kept in `LastKnown`.

**Scenarios.** None yet.
