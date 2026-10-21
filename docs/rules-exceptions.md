# Rules exceptions

The engine and every card follow the game's rules - the Comprehensive Rules and the official rulings - exactly. This
file lists the only deviations a human has allowed. Nothing else is an exception: a deviation not listed here is a bug,
and blocks any change that would bring it in (`CLAUDE.md`, "No rules deviations").

An automated agent never allows a deviation and never adds an entry here. It asks a human, and only when the rule is
one of these:

- **Nonsensical for the domain:** a rule that only makes sense for paper play (ante, physical card handling).
- **Extremely niche and extremely complicated to build:** sub-games (Shahrazad), stickers and the like.
- **Only reaching banned cards:** a rule that affects only cards banned in every format Cantrip supports.

A decision recorded here may be applied to a later case of the same kind; the later case is added to its entry.

## Entry template

```
### RX-<n>: <the rule, and what the engine does instead>

| Rule | Category | Decided by | Date |
| --- | --- | --- | --- |
| CR <number> | paper-only / niche and complex / banned cards only | <person> | YYYY-MM-DD |

**What the rules say.** ...

**What the engine does instead, and why.** ...

**Cards and cases it covers.** ...
```

## Exceptions

### RX-1: A game that reaches the engine's maximum number of events ends in a draw

| Rule | Category | Decided by | Date |
| --- | --- | --- | --- |
| CR 104.4b, 732.4-5 | niche and complex / beyond practical scale | jschibel | 2026-10-21 |

**What the rules say.** A loop of mandatory actions is a draw (CR 104.4b); a loop the players keep going by their own
choices goes on as long as they choose, and no player can be made to break it with an action outside the loop
(CR 732.5). A run of play that never repeats exactly - a token, an effect or a card more each time round, while the
other players only pass - is neither recognised as a loop nor ever ended by the rules.

**What the engine does instead, and why.** The engine keeps recognising every loop it can tell exactly, as the rules
say. Beyond that it has a practical maximum on the game's history (`MAX_GAME_EVENTS`, in events recorded), sized to
hold the longest and most complicated multiplayer game with headroom; a game that reaches it ends in a draw. The
engine's earlier judgements of "a run going nowhere" (a step-length limit, a forced-run limit) are replaced by this one
bound.

**Cards and cases it covers.** Any combination that grows without repeating while nobody acts to end it.

### RX-2 (not a deviation, recorded as a decision): a "may" declined every time round a loop is left to the clock

| Rule | Category | Decided by | Date |
| --- | --- | --- | --- |
| CR 732.5 | - | jschibel | 2026-10-21 |

**What the rules say.** No player can be forced to perform an action that would end a loop other than those called for
by objects in the loop (CR 732.5); a player who declines a "may" each time round keeps the loop going by choice.

**What the engine does.** It does not end such a loop as a draw after a number of rounds. Every iteration is played by
hand, and the match clock runs down until the player times out, as for any other loop kept going by choice.

### RX-3 (not a deviation, recorded as an interpretation): a dependency loop with a dependency outside it

| Rule | Category | Decided by | Date |
| --- | --- | --- | --- |
| CR 613.8b | - | jschibel | 2026-10-21 |

**What the rules say.** If several dependent effects form a dependency loop, "this rule is ignored and the effects in
the dependency loop are applied in timestamp order" (CR 613.8b). The rules do not say what happens when a member of
the loop also depends on an effect outside it, and no ruling settles it.

**How the engine reads it.** Only the dependencies that make up the loop are set aside: the cycle is broken by falling
back to timestamp order among its members, and every other dependency - including a loop member's dependency on an
effect outside the loop - stays in place, so the order is a DAG. Dependencies are judged on the working state at each
step (CR 613.8c). Example: Life and Limb, Arcane Adaptation naming Saproling, Blood Moon (in that order), Taiga and
Llanowar Elves: Arcane Adaptation, then Life and Limb, then Blood Moon.

