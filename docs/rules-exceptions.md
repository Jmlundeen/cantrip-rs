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

None.
