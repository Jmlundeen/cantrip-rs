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
| [KI-26](#ki-26-an-exile-that-a-replacement-sends-elsewhere-is-not-exercised-by-hex-magic-fireglass-mentor-or-esper-origins) | An exile that a replacement sends elsewhere is not exercised by Hex Magic, Fireglass Mentor or Esper Origins | Low | Unexercised |
| [KI-28](#ki-28-noncombat-damage-plus-n-reads-the-controller-of-a-damage-source-that-is-not-on-the-battlefield-or-the-stack) | Noncombat damage "plus N" reads the controller of a damage source that is not on the battlefield or the stack | Low | Suspected |
| [KI-30](#ki-30-a-permanents-name-is-not-part-of-its-last-known-information) | A permanent's name is not part of its last-known information | Low | Open |
| [KI-31](#ki-31-an-adventure-is-never-cast-from-the-top-of-a-library) | An Adventure is never cast from the top of a library | Medium | Open |
| [KI-33](#ki-33-chain-lightnings-payer-copies-without-being-asked-whether-to) | Chain Lightning's payer copies without being asked whether to | Low | Open |
| [KI-34](#ki-34-a-skip-effects-source-that-has-left-may-go-unnamed-in-the-turn-would-begin-prompt) | A skip effect's source that has left may go unnamed in the "Your turn would begin" prompt | Low | Open |
| [KI-35](#ki-35-the-tactical-bot-counts-no-summoning-sick-creature-as-a-mana-source-under-shang-chi) | The tactical bot counts no summoning-sick creature as a mana source under Shang-Chi | Low | Open |
| [KI-37](#ki-37-a-sideboard-card-the-engine-does-not-have-is-left-out-of-the-game) | A sideboard card the engine does not have is left out of the game | Low | Open |
| [KI-38](#ki-38-an-activated-ability-whose-cost-removes-counters-from-among-permanents-is-unexercised) | An activated ability whose cost removes counters from among permanents is unexercised | Low | Unexercised |
| [KI-39](#ki-39-a-state-that-is-true-only-in-the-middle-of-a-resolution-with-no-event-misses-its-state-trigger) | A state that is true only in the middle of a resolution, with no event, misses its state trigger | Low | Unexercised |
| [KI-10](#ki-10-a-lone-action-whose-cost-depends-on-its-target-shows-only-the-printed-cost-until-the-payment) | A lone action whose cost depends on its target shows only the printed cost until the payment | Low | Open |
| [KI-23](#ki-23-a-creature-fighting-itself-cr-70114c-is-implemented-but-no-scenario-can-reach-it) | A creature fighting itself (CR 701.14c) is implemented but no scenario can reach it | Low | Unexercised |
| [KI-40](#ki-40-a-role-that-could-not-enchant-its-creature-and-a-for-as-long-as-duration-false-from-the-start-are-not-exercised) | A Role that could not enchant its creature, and a "for as long as" duration false from the start, are not exercised | Low | Unexercised |
| [KI-41](#ki-41-a-dependency-through-another-object-is-judged-on-the-object-being-worked-out) | A dependency through another object is judged on the object being worked out | Low | Open |
| [KI-42](#ki-42-a-permissions-enters-tapped-is-judged-from-where-the-land-is-not-from-the-way-it-was-played) | A permission's "enters tapped" is judged from where the land is, not from the way it was played | Low | Unexercised |
| [KI-43](#ki-43-a-modal-double-faced-cards-back-face-cast-by-a-permission-is-not-exercised) | A modal double-faced card's back face cast by a permission is not exercised | Low | Unexercised |
| [KI-44](#ki-44-a-hand-way-is-not-offered-to-a-cast-made-while-an-effect-resolves-and-no-emblem-gives-a-way) | A hand way is not offered to a cast made while an effect resolves, and no emblem gives a way | Low | Open |
| [KI-45](#ki-45-the-number-a-cost-sacrifices-is-kept-as-the-spells-x) | The number a cost sacrifices is kept as the spell's X | Low | Suspected |
| [KI-46](#ki-46-the-lore-counter-a-player-puts-on-a-saga-as-their-main-phase-begins-has-no-player-putting-it) | The lore counter a player puts on a Saga as their main phase begins has no player putting it | Medium | Open |
| [KI-47](#ki-47-containment-priest-judges-a-card-entering-transformed-by-its-front-face) | Containment Priest judges a card entering transformed by its front face | Medium | Open |
| [KI-48](#ki-48-emperor-of-bones-puts-its-finality-counter-on-after-the-creature-enters) | Emperor of Bones puts its finality counter on after the creature enters | Low | Unexercised |
| [KI-49](#ki-49-the-tactical-bot-does-not-know-infect-or-poison) | The tactical bot does not know infect or poison | Low | Open |
| [KI-50](#ki-50-an-animation-that-replaces-creature-types-is-unexercised) | An animation that replaces creature types is unexercised | Low | Unexercised |
| [KI-51](#ki-51-a-land-whose-subtype-is-set-keeps-the-copiable-values-other-than-abilities-a-copy-effect-gave-it) | A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it | Low | Unexercised |
| [KI-52](#ki-52-which-lands-lost-their-rules-text-to-a-subtype-setting-effect-is-judged-on-layers-1-to-3) | Which lands lost their rules text to a subtype-setting effect is judged on layers 1 to 3 | Low | Unexercised |
| [KI-53](#ki-53-a-copy-holds-at-most-four-abilities-added-by-exceptions) | A copy holds at most four abilities added by exceptions | Low | Unexercised |

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
| A granted static ability that affects other objects taken by a later "loses all abilities" (`layers::granted_to_others`: the `LoseAllAbilities` clear and the `has.abilities` filter) | The Irencrag's own "loses all other abilities" before its grants | No implemented card makes a noncreature artifact lose all abilities while it stays an Equipment (Oko's +1 makes Everflame an Elk creature, which unattaches it). |
| A granted cast of a back face (`GRANTED_FACE_BACK`, `faces_to_cast`) | nothing | No implemented double-faced card has a castable back face. |
| A flash grant judged for an MDFC back face or a bestowed Aura through `actions::Speed::face`, beyond Sigarda's Aid's bestow | nothing | No implemented double-faced card has a castable noncreature back face; no flash grant names creatures. |
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
Traveler, Innocent Blood.

**Scenarios.**
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
The same holds for a name an effect set (`EffectKind::SetName`, CR 612.8): The Irencrag that left as Everflame, Heroes'
Legacy reads as The Irencrag afterwards.

**Scenarios.** None yet.

### KI-31: An Adventure is never cast from the top of a library

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine | 2026-10-16, review of the land-with-an-Adventure change |

**Notes.** A card may be cast as its Adventure wherever the card itself may be cast (CR 715.3), the top of a library
included: Bolas's Citadel could cast Stomp or Mage Siege off the top for life, and Mystic Forge an Adventure that is an
artifact or colorless spell. `actions::castable`'s two library-top passes (`PlayRule::LibraryTopSpells`, and
`top_for_life`) offer only the card itself, never `inset_faces` (also noted in `docs/engine-design.md`, "Spells in an
inset frame"). Traveling Chocobo's ruling (Lindblum on top: the land may be played, the Adventure not cast) holds today
only because no face is offered; a fix judges each inset face with the filter, as `Speed::face` does for flash.

**Scenarios.**
- [`scenarios/traveling-chocobo-plays-lindblum-off-the-top-but-not-its-adventure.cantrip`](../scenarios/traveling-chocobo-plays-lindblum-off-the-top-but-not-its-adventure.cantrip) - the case that must stay refused; none yet for one that must be offered (Bolas's Citadel with Bonecrusher Giant on top).

### KI-33: Chain Lightning's payer copies without being asked whether to

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | card | 2026-10-16, review of the payment prompt (`Game::may_pay`) |

**Notes.** Chain Lightning: "that player or that permanent's controller may pay {R}{R}. If the player does, they may copy this
spell and may choose a new target for that copy." Paying is one choice and copying a second (CR 118.12, 707.10c), but
`chain_lightning.rs` copies as soon as the payment is made. Only a player who pays {R}{R} and then would rather not copy is
affected. A fix asks a "may copy" (`effects::ask_may`) after a paid `may_pay`, and a scenario pays and declines the copy.

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

### KI-37: A sideboard card the engine does not have is left out of the game

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | engine | 2026-10-16, review of cards outside the game |

**Notes.** A seat's sideboard is the cards it owns outside the game (CR 400.11a), which North Wind Avatar chooses among. The
host refuses a deck naming a card the engine has not ported, but leaves such a card out of a sideboard instead
(`Session::new`, with a line on stderr), so that a constructed decklist whose sideboard names one (`decks/standard/jeskai-lessons.txt`'s
Riverchurn Monument, `mono-black-combo.txt`'s Rottenmouth Viper) is still played. In such a game a card that brings one in from
outside the game cannot choose it. Limited pools are the cube's ported cards, so an event's games are whole. A fix ports the
card, or has the deck loader refuse the list.

**Scenarios.** `cantrip-host`'s `a_sideboard_card_the_engine_does_not_have_is_left_out`.

### KI-38: An activated ability whose cost removes counters from among permanents is unexercised

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, building Dawnhand Dissident's casts |

**Notes.** `Cost::remove_counters_among` ("remove N counters from among creatures you control", CR 118.3) is chosen and paid by
`actions::activate` as well as by `actions::cast`, but only a cast pays it in content (Dawnhand Dissident's permission). Tayam,
Luminous Enigma's "{3}, Remove three counters from among creatures you control" is the printed activated ability it is built for;
the activation path chooses the counters after the mana, as blight's, and removes them with the rest of the cost. When a card with
it is added, its scenarios exercise this path.

**Scenarios.** None yet: no card in content has the cost on an activated ability.

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

### KI-40: A Role that could not enchant its creature, and a "for as long as" duration false from the start, are not exercised

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, review of Royal Treatment and Eluge, the Shoreless Sea |

**Notes.** `create_tokens_onto` in `crates/cantrip-rules/src/ops.rs` does not create an Aura token "attached to" a
permanent it could not legally enchant (CR 303.4i), and attaches an Equipment token only to what it could equip (CR
301.5e); `add_effect` in `crates/cantrip-rules/src/lib.rs` refuses a `Duration::WhileCounter`/`WhileTapped` effect whose
condition is already false (CR 611.2b). No card in content gives protection from enchantments or keeps a counter from
being put on a land, and none creates an Equipment token attached to something, so those arms run in no scenario. The day
such a card exists each needs a scenario (Royal Treatment on a creature with protection from enchantments; Eluge's flood
counter on a land that can't have counters put on it), and this entry is deleted.

**Scenarios.** None yet.

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

### KI-42: A permission's "enters tapped" is judged from where the land is, not from the way it was played

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, review of Lightstall Inquisitor |

**Notes.** "Each land played this way enters tapped" (CR 305.1, 614.1c; `PlayGrant::enters_tapped`) is the core's replacement
`replace::Origin::PlayedTapped`, which works out from the land's zone and the permissions on it (`actions::land_play_permission`)
whether the land play was made by the permission: the action does not carry the way it was played. Two paths would get it wrong,
and no printed card in the engine reaches either: a land with such a permission played by a resolving effect instead
(`actions::play_land_during_resolution`, which would enter tapped though it was not "played this way"), and a land in a graveyard
with such a permission played by a way a permanent grants (`granted_slot`), which `played_by_permission_from` does not know of.
Of several ways that could play the same land, the engine also takes the one that lets it enter untapped rather than asking. A fix
carries the permission used on the PlayLand move into the entry's `ZoneChange`.

**Scenarios.** None yet: no card in the engine both plays a land by an effect and could face a Lightstall Inquisitor's card.

### KI-43: A modal double-faced card's back face cast by a permission is not exercised

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-16, review of Inside Information |

**Notes.** A permission to cast a card from where it is (CR 601.3) now lets a modal double-faced card be cast as its back face
(CR 712.11b) and a split card as either half (CR 709.3, an aftermath half only from a graveyard), for what the permission says
to pay (`actions::castable`, `face_cast`). The split halves are proven (Inside Information with Life // Death); no card in
content is a modal double-faced card whose back face is a spell rather than a land, so the back-face cast from exile runs in no
scenario. (Two transforming cards, Jennifer Walters and Norman Osborn, give their back faces a mana cost in content, which makes
`back.has_cost` true for them; whether that offers a back-face cast it should not is worth a look.) A scenario is owed when a
spell-backed modal double-faced card is added.

**Scenarios.** None yet.

### KI-44: A hand way is not offered to a cast made while an effect resolves, and no emblem gives a way

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Open | engine | 2026-10-16, review of Omniscience |

**Notes.** Omniscience's "you may cast spells from your hand without paying their mana costs" is a `GrantedWay` from the hand
(CR 118.9), offered with priority (`actions::castable`). A cast an effect makes as it resolves from its controller's hand
(CR 608.2g) is offered only for the card's mana cost and faces (`cast_ways_during_resolution`), so Omniscience's free cast is
not among its ways there, though the rules allow any alternative cost. Ways are found on the battlefield's permanents only
(`granted_ways_of`), so an emblem that gives one - Tamiyo, Field Researcher's "You may cast spells from your hand without paying
their mana costs" (CR 114.4) - could not be built. No card in content casts from a hand as it resolves under such a way.

**Scenarios.** none yet.

### KI-45: The number a cost sacrifices is kept as the spell's X

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Suspected | engine | 2026-10-16, review of Rottenmouth Viper |

**Notes.** `VariableCost::SacrificeAnyNumber` (Rottenmouth Viper) and `SacrificeX` (Immoral Bargain) keep the number announced
as the spell's `x` (`StackItem::x`, `Obj::x`), though Viper's mana cost has no {X}. Its mana value is right (`Query::mana_value`
and the `Cast` record count X once per {X} in the mana cost, CR 202.3e), but anything that reads `x` as "the value of X" -
"X is the value of X paid" on a permanent, a copy's X (CR 707.10), the client's X shown on a spell - would see the number
sacrificed for a spell that has no X. No card in content reads X of a Viper. A fix keeps the number apart from X.

**Scenarios.** none: no card reaches it.

### KI-46: The lore counter a player puts on a Saga as their main phase begins has no player putting it

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine | 2026-10-17, scenarios of Esper Origins |

**Notes.** CR 714.3c: "As a player's precombat main phase begins, that player puts a lore counter on each Saga they control."
`ops::add_lore_counters` runs as a turn-based action whose cause is the rules, so `replace::counters` finds no putter
(`Cx::actor` is `None` for `Cause::Rules`), and "If you would put one or more counters on a permanent or player" (Innkeeper's
Talent at level 3, `replacements::double_counters_applies`, keyed to the putter) does not double it. The lore counter a Saga
enters with is doubled, as its controller puts it (CR 122.6a). A fix names the active player as the putter of the turn's lore
counters.

**Scenarios.**
- [`scenarios/innkeepers-talent-doubles-the-lore-counter-a-player-puts-on-summon-esper-maduin-as-their-main-phase-begins.cantrip`](../scenarios/innkeepers-talent-doubles-the-lore-counter-a-player-puts-on-summon-esper-maduin-as-their-main-phase-begins.cantrip)
  (marked `# Known bug`): one lore counter becomes two, so chapters II and III trigger together.

### KI-47: Containment Priest judges a card entering transformed by its front face

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine / card | 2026-10-17, scenarios of Esper Origins |

**Notes.** "If a nontoken creature would enter and it wasn't cast, exile it instead" looks at the permanent as it would exist on
the battlefield (CR 614.12; Containment Priest's ruling: "If a noncreature card wasn't cast and is entering the battlefield as a
creature ..., it will be exiled"). `containment_priest.rs` reads `q.chars(z.object)`, the card where it is, and a card put onto
the battlefield transformed (`put_onto_battlefield_transformed`, CR 712.14a) has only its front face's characteristics there
(CR 712.8a): Esper Origins (a sorcery) entering as Summon: Esper Maduin (a creature), or The Legend of Roku (an enchantment)
returning as Avatar Roku, is not exiled. `Proposal::ZoneChange` does not say the object will enter transformed (the core's
`Entering::transformed` does), so the card cannot ask; a fix gives replacements the entering permanent's characteristics (the
same is owed to a face-down permanent, CR 708).

**Scenarios.**
- [`scenarios/esper-origins-entering-transformed-under-containment-priest-is-exiled-instead.cantrip`](../scenarios/esper-origins-entering-transformed-under-containment-priest-is-exiled-instead.cantrip)
  (marked `# Known bug`): Summon: Esper Maduin enters under Containment Priest.

### KI-48: Emperor of Bones puts its finality counter on after the creature enters

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | card | 2026-10-17, review of Esper Origins |

**Notes.** "Put a creature card exiled with this creature onto the battlefield under your control with a finality counter on it"
is one entry with the counter on it (CR 122.6, 614.1c). `emperor_of_bones.rs` calls `put_onto_battlefield` and then
`add_counters`, though `put_onto_battlefield_with_counters` does both, as Scavenger's Talent and Zoraline use (and
`put_onto_battlefield_transformed_with_counters` for a card entering transformed, as Esper Origins uses). Nothing can happen
between the two inside the resolution; the difference is to an ability that triggers on a permanent entering with a counter on
it (no card in content has one) and to who puts the counter (CR 122.6a: the permanent's controller, which for the Emperor is
also the ability's controller). A fix calls `put_onto_battlefield_with_counters`.

**Scenarios.** none: no card reaches it.

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

### KI-50: An animation that replaces creature types is unexercised

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | mechanics | 2026-10-17, review of Inkmoth Nexus |

**Notes.** An effect that makes an object "a [creature types] artifact creature" without saying it is still what it was
keeps its card types and other subtypes but replaces its creature types with those named (CR 205.1b). `Animation` in
`cantrip-mechanics` has `replaces_creature_types` for it (a `LoseAllCreatureTypes` effect before the named types), set by
Iron Man Armor ("a 0/0 Construct Hero artifact creature"). The line runs in the Armor's scenarios, but the Armor has no
creature types of its own to lose, so no scenario shows a creature type being replaced. It would need the Armor to have a
creature type before it animates (an effect that gives every permanent a creature type), which no card in content does.

**Scenarios.** none yet: Iron Man Armor could reach it with such an effect.

### KI-51: A land whose subtype is set keeps the copiable values other than abilities a copy effect gave it

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, Magus of the Moon |

**Notes.** CR 305.7: a land whose subtype an effect sets "loses all abilities generated from its rules text, its old land types,
and any copiable effects affecting that land". `Chars::set_land_subtypes` takes, in layer 4, every ability the land has by then:
its card's and those a copy effect gave it (the copied card's abilities and keywords, a copy's "except it has ..."), which is the
part of a copy effect that is abilities. What else a copy effect made of it - the name, mana cost, colors, card types and power and
toughness of what it copies (CR 707.2) - stays, as layer 1 applied it. Whether "loses any copiable effects" takes those too (a
land copying an animated Mutavault, or Vesuva copying a land on the battlefield before Blood Moon arrives) is not settled by any
ruling of a card the engine has; a copy entering under the effect does not copy at all (the "as enters" choice is lost, CR
614.12), which is the case the rulings name. A fix, if a ruling says the copy effect goes entirely, starts the land's
characteristics from its own card when `layers::land_text_lost` holds it.

**Scenarios.** none yet. Vesuva and Thespian's Stage (added 2026-10-17) now make it reachable: a Vesuva or Stage copying a
nonbasic land, then Magus of the Moon entering.

### KI-52: Which lands lost their rules text to a subtype-setting effect is judged on layers 1 to 3

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, Magus of the Moon |

**Notes.** `layers::land_text_lost` finds the lands a "nonbasic lands are Mountains" effect takes the rules text of by asking the
effect's `affects` of each permanent as layers 1 to 3 left it, as `lost_to_statics` does for layer 6, so that the question never
asks for the layer it is answering. The layer-4 effect itself is applied to each land in its place in layer 4 (CR 613.7, 613.8),
and takes the land's abilities there. The two disagree only for an object a layer-4 effect makes a land or stops being one (Hydro-
Man, Fluid Felon becoming a land; a land animated into a non-land by an effect that sets its types): such an object under Magus of
the Moon is a Mountain and loses the abilities among its abilities, but `Query::lost_all_abilities` does not say so, so the rules
its card prints apart from its abilities (replacement effects, cost changes, static abilities of other objects) would still apply.
Hydro-Man prints none. Likewise a land about to enter (`layers::loses_rules_text_entering`, CR 614.12) is judged by its own card or
the copy it enters as, without the layer 1-3 effects of other objects: none in content changes a land's supertypes or types as it
enters. A fix applies the object's own layer-4 effects that change its card types before asking.

**Scenarios.** none: no card in content reaches it.

### KI-53: A copy holds at most four abilities added by exceptions

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Low | Unexercised | engine | 2026-10-17, Thespian's Stage |

**Notes.** `Copiable::extra_abilities` (`crates/cantrip-state/src/object.rs`) is a fixed array of four; adding a fifth
"except it has [ability]" (CR 707.9b) is silently dropped. Each Thespian's Stage copying a Stage-copy adds one more instance of the
Stage ability (CR 113.2c), so a chain of five Stages copying one another loses one. No other card in content adds more than one.
A fix grows the array or refuses the copy loudly; instances beyond the second of the same ability change no play.

**Scenarios.** none: no scenario builds the chain.
