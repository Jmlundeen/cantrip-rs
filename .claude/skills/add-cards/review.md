# Reviewing a card

Before a card lands, someone other than its author - a person, or a separate agent that had no part in writing it -
reviews it against `SKILL.md` beside this file: its non-negotiables, its "Where code goes" with the **"never" list**,
and "Scenarios: what a card owes". Read those, then review with the Oracle text, the rulings and the CR open
(`python tools\maintenance\scryfall.py "Card Name"`, `python tools\maintenance\cr.py <rule>`).

Log it on the card's issue: `python tools\maintenance\card-issues.py log <N> review-started` first, and at the end
`log <N> review-passed`, or `log <N> review-blocked --findings <how many blocking>` - fixed comments, nothing to write.

**The primary concerns are the code's correctness and safety and the card's adherence to the rules**; the tests are
checked for sufficiency, not exhaustiveness:

- every sentence of the Oracle text against the code, and every line of the code against the text: nothing missing,
  nothing extra, nothing approximate, nothing deviating from the rules;
- the content JSON against Scryfall's current card;
- every `CR` citation in the card's file and scenarios: `cr.py check --show <files>` prints each beside the rule it
  names (and flags a number the current rules don't have) - each rule says what the comment says it does - citations copied from
  older code can name a rule that was renumbered (CR 712.4 is meld cards, not modal double-faced cards);
- the "never" list, line by line;
- no parallel system (`CLAUDE.md`, "Use the systems that exist"): every new store, cache, flag, record, lookup or helper
  is checked against what the engine already has for that job - the event history for the past, the layers for
  characteristics, the journal for undoable state, the shared mechanics and `cantrip_state` helpers - and a second
  system for the same kind of thing blocks;
- the test points (`new-card.py "Card Name" --no-write` prints them): the author's report pairs each with the
  scenario line that catches it - check each pair, and look past the list for anything the text decides that it missed;
- the scenarios: every line covered (the coverage report), each `[Expect]` correct by the rules and able to fail - a
  limit the card's own code enforces met exactly at the limit, and both players' objects where the text reads "each",
  "target" or "you control". More tests are asked for only when the scenarios are obviously insufficient (code no
  expectation would catch being wrong), when the card is complex enough to need a specific test, or when an
  interaction has a clear potential to go wrong. A list of corner cases is not a finding;
- any core or mechanics change against its rule, its second printed card (a non-keyword recipe) and the benchmark (a
  core change);
- overlap: code in the card that an existing shared function already does, or that does the same as code in another
  card, which should have been shared;
- the client: everything the card puts into the game is shown, every question it asks has a true caption, every action
  it offers is clickable with its cost, and a browser scenario proves each new user interaction (`client.md`) - or the
  card needs nothing the client did not already do, and no browser scenario is owed. The tactical bot is never checked.

A reviewer runs only the card's scenarios and coverage (`corpus -- --card "Name"`, `python tools/maintenance/card-coverage.py "Name"`), and those of a
card whose shared code the change touched - nothing wider - and, in another's worktree, changes no files, checks out
nothing and cleans nothing.

The reviewer changes nothing silently: each finding is fixed by the author or recorded as the reason the card does not
land. Self-reported "exact" is not proof. A finding **blocks** the card when it is a rules deviation, a code defect
(an overflow included), an item on the "never" list, or a test that is obviously insufficient - and a decision of the
card's own code (a number, a limit, a narrowing, whose objects) that no expectation would catch being wrong is
obviously insufficient, however small. A wrong citation is fixed in the same pass; the rest (wording, optional
sharing) does not hold the card.

**A reviewer never accepts a rules deviation, or anything that risks one** (`CLAUDE.md`, "No rules deviations"): a
clamp, a floor, a fixed-size list that cuts down or drops, an approximation that holds only for today's cards, a path
left untested "until a card needs it". Each blocks. "Record it as a known issue" is never an answer to a rules finding.
A suspected rules bug elsewhere is reported as a finding to be fixed, not filed away; a path no scenario can reach gets
a Rust test with test-only cards. Only a human may allow a deviation, in `docs/rules-exceptions.md`.

Report briefly: PASS, or the blocking findings - each with file:line, what is wrong and the rule - then any
non-blocking notes apart.
