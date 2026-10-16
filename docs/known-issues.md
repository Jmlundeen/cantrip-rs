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
| [KI-6](#ki-6-tokens-do-not-enter-tapped-under-an-enter-tapped-replacement) | Tokens do not enter tapped under an enter-tapped replacement | Medium | Open |

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
### KI-6: Tokens do not enter tapped under an enter-tapped replacement

| Severity | Status | Area | Found |
| --- | --- | --- | --- |
| Medium | Open | engine | 2026-10-16, Authority of the Consuls review scenarios |

**Notes.** "Creatures your opponents control enter tapped" (Authority of the Consuls, Spider-Woman, Archon of Emeria's
nonbasic lands) is a replacement of the entering event (CR 614.1c) and applies to a token as it is created (CR 111.1),
but `ops::create_tokens_worded` (`crates/cantrip-rules/src/ops.rs`) never runs the `ZoneChange` replacements
(`replace::zone_change`): it takes `tapped` only from the creating effect and `prepare_entry`. A token made under an
opponent of such a permanent enters untapped. The enters trigger is unaffected (Authority's life gain works for tokens).

**Scenarios.**
- [`scenarios/authority-of-the-consuls-taps-a-token-created-under-an-opponent.cantrip`](../scenarios/authority-of-the-consuls-taps-a-token-created-under-an-opponent.cantrip) - two Spirit tokens must enter tapped (marked `# Known bug`).
