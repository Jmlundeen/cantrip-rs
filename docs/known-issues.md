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
| [KI-9](#ki-9-a-copy-of-a-modal-spell-cannot-be-given-new-targets) | A copy of a modal spell cannot be given new targets | Medium | Open |

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
| Ward's life prompt for an amount above 60 (`triggers::ward_life_question`: the static `Question`s name the amount for 0 to 60 only, so a bigger one is asked as "Pay the life", without the number; the payment itself is exact) | Raubahn, Bull of Ala Mhigo's ward at 4 | No implemented creature reaches a power above 60. A `Question` with a runtime amount would remove the table. |

| A copy of a spell paid for with mana that made it uncounterable or gives it haste (Cavern of Souls, Delighted Halfling, Arena of Glory) is counterable and gains nothing (`ops::copy_spell` clears `cant_be_countered` and `gains_haste`: no mana was spent on the copy, CR 707.10, 106.6) | nothing | No implemented card copies a creature spell or a legendary spell, the only spells that mana can pay for. |
| Discover dropping a face whose mana value is over N although the card's is not (`discover::discover`'s `ways` filter; CR 701.57a, "if the resulting spell's mana value is less than or equal to N") | Adventurers and split cards with both faces at or under N (Quintorius Kand's scenarios) | No implemented card has an Adventure or castable back face costing more than a front face of 4 or less. |
| Planetarium of Wan Shi Tong that left the battlefield and came back the same turn after a card was cast with it: "do this only once each turn" is looked for among the controller's casts with the ability of any Planetarium (CR 603.2h: "its source's controller has not yet taken the indicated action"), so the new object (CR 400.7) does not trigger again; keyed on the object it would | nothing | No implemented card blinks or returns an artifact to the battlefield in a turn the Planetarium has already cast with. |

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
### KI-9: A copy of a modal spell cannot be given new targets

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine | 2026-10-16, Restoration Magic copied by Leyline of Resonance |

**Notes.** "You may choose new targets for the copy" (CR 707.10c, 115.7d) is offered by `retarget` in
`crates/cantrip-rules/src/ops.rs` from `cx.card(obj.card).spell_targets()`, the spell's printed target slots. A modal spell's
targets are those of the modes it was cast with (`ModeDef::targets`, chosen by `StackItem::modes`, as `cast_modes`/the target-spec
builder in `crates/cantrip-rules/src/actions.rs` reads them), so the copy keeps its old targets and is never asked.
Affects every modal spell (spree, tiered, escalate, "choose one") that is copied with a chance to retarget (Leyline of
Resonance, Lutri, storm and replicate copies). A fix has `retarget` take the specs of the copy's chosen modes.

**Scenarios.**
- [`scenarios/restoration-magic-copy-may-be-aimed-at-a-new-target.cantrip`](../scenarios/restoration-magic-copy-may-be-aimed-at-a-new-target.cantrip) - the copy of a Cura aimed at a Plains (marked `# Known bug`).

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
