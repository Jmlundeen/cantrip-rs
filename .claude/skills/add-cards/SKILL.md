---
name: add-cards
description: Add one or more Magic cards to the Cantrip rules engine - each card exact to its Oracle text and the Comprehensive Rules, built from the engine's shared rules verbs and keyword mechanics, with brief scenarios that execute every line of its code and show it observably correct, playable end to end in the web client - and, for many cards, run them through parallel agents and land their work. Use whenever a card is added, implemented or fixed, by anyone.
---

# Adding cards

This is the standard for every card in Cantrip, whoever writes it. A card that does not meet all of it is not added.
`docs/adding-cards.md` is the reference for a card file's fields; `docs/engine-design.md` is the architecture;
CLAUDE.md is the repository's rules. The `write-game-script` skill is the scenario syntax, in full.

## The non-negotiables

1. **The card is exactly what the rules say it is.** Its Oracle text, read with the Comprehensive Rules
   (`docs/rules/MagicCompRules *.txt`) and the card's official rulings, is the whole specification. There is no
   approximation, no "close enough", no simplification, no house rule, no behaviour that is right in the usual case
   and wrong in a rare one, no "the bots never do that", no deviation of any kind - in what it does, in when it does
   it, in what it asks, in whom it asks, in the order it asks, or in what it lets a player do. If the engine cannot do
   what the card needs, the card waits: the missing rule is built in the core, properly and for every card (below),
   or the card is not added and the gap is reported. A card that is wrong is worse than a card that is missing.
2. **Rules belong to the engine, never to the card.** A card file holds only what that one card does alone. It calls
   the engine's verbs for rules procedures and the shared mechanics for keywords and recipes.
3. **Every line of the card's code is executed by its scenarios**, every branch both ways, and each shows the card
   observably doing what its text says. Measured, not claimed (`tools\card-coverage.cmd`). Scenarios are brief;
   corner cases are tested only where an interaction can plausibly go wrong (below).
4. **An independent review against the Oracle text and the rules** before it lands.
5. **The card works end to end, in the client.** A card is done when a person at a table in the web client can play
   it: see everything the card puts into the game (a state on an object - doors, a chosen name or type, counters of a
   new kind, a face, a designation; a revealed card; a new zone or pile), make every choice it asks with wording that
   says what the choice does, and take every action it offers (an activated ability, a special action, an alternative
   cost, an optional cost) with its cost shown. If the engine side works and the client cannot show or offer it, the
   card is not done. The language-model client and the draft bot, where it reads cards, answer its questions
   sensibly, never by a reading of a question meant for something else. The tactical bot
   (`dotnet/src/Cantrip.Client/Tactics`) is not part of card work and is no development tool: no card, core or
   review agent reads, changes, runs, tests through or waits on it, and a review raises no finding about how it plays
   a card - its correctness is proven by hand-authored scenarios alone.

## Where code goes

Three layers, kept apart by the compiler (`cantrip-api` is all a card can see):

| Layer | Crate | What goes there |
| --- | --- | --- |
| Core | `cantrip-state`, `cantrip-rules`, exposed by `cantrip-api` (`Game`, `Query`) | Every procedure the Comprehensive Rules define: destroy, sacrifice, exile, return to hand, put into a library, mill, discard, damage and its prevention, counters, control, targeting legality, protection/hexproof/ward, the stack, priority, layers, replacement and prevention ordering, triggers, state-based actions, combat, mana, costs, zones and new-object identity (CR 400.7), last-known information, simultaneity (CR 101.4, 603.10). Keywords that change a core procedure (trample, first strike, split second). |
| Shared mechanics | `cantrip-mechanics` | **Every keyword action (CR 701) and keyword ability (CR 702), from the first card that has it**: one module named for the keyword, parameterised (`earthbend`, `evoke`, `storm`, `amass`, `flashback`, ...). Any other recipe two or more printed cards use exactly as written: target filters, trigger matchers, tutors, land templates, tokens, one-line effects, delayed end-step effects. |
| Card | `crates/cantrip-cards/src/cards/<card>.rs` | Only this card's own conditions, values, branching and sequencing, over `Game`, `Query` and the shared mechanics. |

**A card file must never:**

- filter on indestructible, regeneration, protection, hexproof, shroud or ward to decide what a rules action does -
  the verb does that (`destroy_all`, `destroy_target`, targeting);
- build a zone destination the rules dictate (`Zone::graveyard(owner)`, `Zone::hand(owner)`, a library position), or
  call `move_together` to perform a rules action that has a verb: use `destroy_all`/`destroy_all_no_regen`,
  `sacrifice`, `exile_all`, `return_to_hand`, `put_into_library(.., LibraryPlace::..)`, `mill`, `discard_together`,
  `discard_hand`, `library::search_to_hand`/`search_to_battlefield`/`search_to_top`;
- shuffle or randomise by hand (`put_into_library(.., LibraryPlace::BottomInRandomOrder)`, `bottom_in_random_order`,
  `library::random_of`);
- loop over players or objects for something the rules make simultaneous (`damage_together`,
  `each_opponent_loses_life`, `each_opponent_sacrifices`, `destroy_all`);
- check whether a player has lost. "Each player" is `apnap()` (or `turn_order_from(p)` for "starting with you") and
  "each opponent" is `opponents(p)`, and both already leave out players who have left the game (CR 800.4a). Targeting
  refuses them, and a player who leaves takes their spells and abilities with them, so a resolving ability's
  controller is always in the game. `seats()` is every seat, for walking zones by owner, never for "each player". Only
  a player named another way (Fecundity's "that creature's controller") is asked `in_game(p)`;
- name a color word, basic land type or creature type its rules text uses without reading it through the words of the
  object whose text it is (`g.words(item.object).subtype(Subtype::Goblin)`, `view.words(source).color(Color::Red)`):
  a text-changing effect (CR 612) has to change what the card does (`docs/adding-cards.md`, "Words of its rules
  text"). Mana symbols are not words, and the type line and the tokens it makes follow by themselves;
- build a choice question by hand that `cantrip_mechanics::choose` asks (`card`, `cards`, `card_from`, `cards_from`,
  `player`, `one_of`), or write a fallback arm for an answer of the wrong shape: the core checks every answer against
  its question before the card sees it;
- read a printed cost where the rules mean mana value (`Query::mana_value`: copies, tokens, face-down permanents,
  X on the stack);
- implement a keyword, even one only this card has today;
- copy something `cantrip-mechanics` or `cantrip-api` already has (target filters and `targets::object`, trigger
  matchers, `effects`, `tokens`, `delayed`, `library`, `lands`), or define a token a shared definition covers;
- keep a counter or a flag for something that happened earlier: the past is asked of the event history,
  `q.history()` (a one-off query in the card; a question a second card asks moves to `cantrip_mechanics::history`);
- hand-write a stale-object guard (`targets::permanent`, `cantrip_api::is_live`), or guard `sacrifice`, which already
  skips objects that changed zones (CR 400.7);
- produce a different event than the rules name: making Clue tokens is not investigating; moving cards to the
  graveyard is not discarding (and skips madness); moving a card out of a library is not searching.

**Adding to the core or to the mechanics.** A core feature mirrors the rule it implements, cites the CR, works for
every card the rule describes, and is guarded on the game state - never on "does any card in the card table have X",
because the table holds every card. A core change is benchmarked: `cargo run --release -p cantrip-bench -- --games
128`, the "engine answering" line, before and after, reported with the change. A mechanics change is not: the
mechanics are card recipes, outside the decision loop the benchmark measures. A non-keyword mechanic names
in its doc comment at least two printed cards that use it exactly as written; if only one does, it stays in that card.
A keyword module cites its CR rule and names the cards that use it. Never widen a central switch to encode a card.

**Look for overlap before writing anything that could be shared.** When a card needs a mechanic that another card
might also have - a keyword, a recipe, a choice, a history question, a target filter - first search the existing card
files, `cantrip-mechanics` and `cantrip-api` for the same thing (grep the Oracle wording in `content/*/cards/*.json`
too, to find the cards that print it). If an existing function does it, use it. If two or more cards already do the
same thing in their own code, move it into one shared function and change those cards to call it, in the same change.
Share what is functionally identical, when sharing makes each card's code simpler. Never stretch a shared function to
cover something that only looks alike: a flag or parameter that only one caller uses, or a branch on which card is
calling, means it was not the same thing. Leave that in its own card.

## Which card, and claiming it

Every card that is worked on is an issue on GitHub, claimed by whoever works on it, so two people never build the same
card. `python tools/maintenance/card-issues.py` does all of it (`--help` for the rest); `gh` must be signed in.

1. **Which cards.** If the request names cards, those are the cards. Otherwise the cards come from a format's list of
   cards that decks actually play (`docs/formats/<format>.json`, best-played first, made from MTGGoldfish and MTGTop8
   by `format-scrape.py`), and the format is the contributor's to choose: **ask which format** unless they said (the
   lists present are the files in `docs/formats/`; today Standard). `card-issues.py next <format>` prints the
   next cards that are not implemented and that nobody holds - a claim older than 24 hours with no pull request
   commit or `card/<slug>` push since counts as nobody's. Take them in that order and never skip one for being
   hard; a card the engine cannot yet do is held while its gap is built in the core (below), not passed over
   silently.
2. **Claim one card at a time, before any work on it:** `card-issues.py claim --format <format>` takes the next free
   card, or `claim "Card Name"` the one named; the next card is claimed only when this one has landed or been
   released - or is waiting on a core gap, which stays claimed while the next card is worked (never more than those
   two) - because a card takes minutes and a pile of claims keeps others off cards nobody is working on. It opens the card's issue
   first when it has none (a card named outside every format list gets one too), then claims it with a comment, and
   prints `#N`. Exit status 3 means the card is held by someone else (or they claimed it a moment earlier): leave it
   and take the next. Never work on a card whose live claim is someone else's.
   **A card that already exists is validated, not rebuilt.** If the card an issue names already has a content file
   (in this checkout, on a local branch, or `claim` says "already implemented"), the issue is almost always a listing
   error: give it a cursory look of a minute or two and move on. The look is only this: if its Oracle text needs code,
   the card file exists and its code reasonably looks like it does what the text reads (and the content JSON's text
   is the card's). If it passes, close the issue as already implemented, saying so in a comment, and take the next
   card. Anything doubtful is logged as a bug in `docs/known-issues.md`, not fixed by a rebuild. Only when the
   cursory look fails - no code where the text needs it, or code that plainly does something else - is the card
   re-implemented through the loop below.
3. **Keep the claim alive** - it lasts 24 hours from the last sign of work. Open a (draft) pull request whose
   description says `Closes #N` as soon as there is a commit, or push work in progress to a branch named
   `card/<card-slug>`; every commit pushed there renews the claim. A card waiting on a core gap keeps its claim, and
   the claim is never renewed by hand: 24 hours is far more than any one card takes, core gaps and all, so a claim
   that lapses means the work stopped. A card that is abandoned - nobody will build its gap, or the work
   stops - is released (`card-issues.py release "Card Name"`) with its gap reported.
4. **Landing it closes the issue:** the commit message (and a pull request description) ends with `Closes #N`.

## The loop for one card

1. **Read the rules before any code.**
   - The Oracle text (the card's `content/<dataset>/cards/<card>.json`, or the current Oracle text if the card is new
     to content) and its official rulings.
   - Every CR section its words touch: each keyword and keyword action, each "can't", "instead", "as ... enters",
     "whenever", "at the beginning of", each cost and each kind of target. Write down, for each sentence of the Oracle
     text, the rule that governs it.
   - What the rules make the card do in the odd cases: no legal target, a target that becomes illegal, a source that
     has left the battlefield, a copy, a token, a face-down permanent, an opponent controlling it, two of it, X of
     zero, an empty library or hand, simultaneous events, a replacement effect in the way.
2. **The card's content file.** If it does not exist, write `content/<dataset>/cards/<card-name>.json` with the
   characteristics only: `name`, `manaCost`, `type`, `power`/`toughness`, `loyalty` or `defense`, `oracleText` exactly
   as current Oracle text, core `keywords` (`Flying`, `FirstStrike`, ... - the list is in
   `crates/cantrip-oracle/build.rs`), `colorIndicator`, `convoke`/`delve`/`improvise`, and faces (`backFace`,
   `adventure`, `omen`, `halves`, `aftermath`). Nothing else: what a card does is Rust. The build generates its constant;
   there is no generator to run and no shared file to edit. A card whose Oracle text is only core keywords needs no
   Rust file.
3. **Start the file.** `python tools\maintenance\new-card.py "Card Name"` prints the JSON and every scenario that holds
   the card, and writes `crates/cantrip-cards/src/cards/<card_name>.rs` over its generated constant.
4. **Look before writing.** Read `crates/cantrip-api/src/game.rs` (the verbs) and grep `crates/cantrip-mechanics/src`
   (the keyword modules, `targets`, `triggers`, `library`, `lands`, `effects`, `tokens`, `delayed`, `copy`, `hand`,
   `chosen_type`, `history`, `combat`, `replacements`), and read two or three implemented cards with similar text. Most
   cards need nothing new.
5. **Write the card.** Its file adds behaviour with struct update syntax over the generated constant
   (`CardDef { spell: Some(..), ..oracle::LIGHTNING_BOLT }`). Ask questions with `g.ask(..)` in the order the rules ask
   them, offering options in the order the engine and the implemented cards do (scenarios answer by name and position).
   Doc comments quote the Oracle sentence each function implements and cite the CR.
6. **Write the scenarios** (next section) and run them until they pass:
   `cargo run --profile cards -p cantrip-script --bin corpus -- --card "Card Name"`.
7. **Measure coverage:** `tools\card-coverage.cmd "Card Name"` must report every line of the card's file executed
   (it runs the card's scenarios under `cargo-llvm-cov`; install it once with `rustup component add
   llvm-tools-preview` and `cargo install cargo-llvm-cov --locked`).
   A line no scenario can reach is wrong code or a missing scenario, never an exception.
8. **The client.** Ask what a player needs from the client to play this card, and check each against what the client
   already presents (`docs/web-client.md`; `crates/cantrip-host` builds the views, prompts and actions the client
   draws; `dotnet/src/Cantrip.Client.Presentation` and `Cantrip.Client.Web` draw them):
   - **State:** anything new on an object, a player or a zone reaches the host view (`crates/cantrip-host/src/view.rs`)
     and is drawn - legibly, at table size, for both the owner and the opponent as the rules allow them to see it.
   - **Questions:** every `g.ask(..)` the card makes has a prompt the client shows with wording true to what the choice
     does. A new `Purpose` or `RequestKind` gets its own view and caption; never map it onto an existing one that means
     something else.
   - **Actions:** every ability, special action and cost the card offers is clickable, with its cost and what it
     does shown, and distinguishable from the card's other actions.
   - **Not the tactical bot:** card and core work does not read, change, run, test or wait on the tactical bot, and
     reviews do not check it.
   A card that only uses what the client already shows and asks needs no client change - the review confirms it.
   **Browser tests are only for a new user interaction**: a prompt, action, cost display or piece of state the client
   did not have before. A card that reuses existing prompts and actions gets no browser scenario and runs no browser
   tests: its headless scenarios prove it. A card that does add an interaction makes the change in the same work, and proves it with a browser
   scenario: `# Browser` at the
   top and `? Client.` checks of what the page shows (`docs/scenarios-in-the-browser.md`), run with
   `tools\test.cmd /filter "ScenarioInBrowser"` (or `tools\scenario-in-browser.cmd name` to watch), after
   `cargo build --profile cards -p cantrip-host`: the table runs the last-built `cantrip-native-host.exe`, and one that
   predates the card publishes nothing. If the browser
   driver cannot click its way through the new prompt or action, extend `ScenarioPlayer`/`TablePage` - the same rule as
   the harness: never drop the check to fit the driver. A protocol change bumps the version and passes the wire-shape
   test, in the same commit as the change.
9. **Verify:** plain `cargo run --profile cards -p cantrip-script --bin corpus` shows `0 fail` and no fewer passing
   than before; `set CANTRIP_VERIFY=1 & cargo test --profile cards --workspace` is green; if the core or mechanics
   changed, `tools\test.cmd`, and if the core changed, the benchmark; if the client, the host or the protocol changed,
   `tools\test.cmd /noweb` and, only if it added a new user interaction, that interaction's browser scenarios; other
   wall-clock tests only when the change touches what they cover (CLAUDE.md). A card agent working for a coordinator
   runs none of the full suites: it runs the scenarios of its card and of every card whose file or shared mechanic it
   changed (`corpus -- --card "Name"` for each) and the unit tests of a crate it changed; the coordinator's full run
   at landing is the gate (Many cards, 5).
10. **Review** (below), then commit: `Card: <Name>` - the content JSON, the card file and its scenarios - one card per
    commit, its message ending with `Closes #N` (the card's issue); core, mechanics and client changes in their own
    commits before it (`Engine: ...`, `Mechanics: ...`, `Client: ...`). Commit and push as CLAUDE.md says; a
    contributor without push access opens a pull request with the coverage report and the review in its description,
    which also says `Closes #N`.

## Scenarios: what a card owes

A scenario (`scenarios/<what-it-proves>.cantrip`; syntax in `scenarios/README.md` and the `write-game-script` skill)
is a real game: both decklists in order, every answer, and the state it must end in. Positions are forced by the
script. **Never sweep RNG seeds**, and never rely on a bot's choices to reach a position.

Keep them brief: a few short scenarios that exercise all of the card's code and show that it is observably correct.
Not every usage of every mechanic needs a test, and a corner case is not tested for its own sake. A card's scenarios,
together, must:

- **Execute every line of the card's code and take every branch both ways**: each `if` true and false, each `match`
  arm, each early return, each mode, each way to cast it, each cost and alternative cost, each ability, each face.
  `tools\card-coverage.cmd` checks the lines; the branches are on you, listed in the review.
- **Show what the card's text does**, and where the card's own code enforces a limit ("only", "up to", "another", a
  mana value or type restriction), a refusal (`! Bot1.Hand.Card.Cast`, `! Bot1.Choose(..)`) beside the allowed case,
  so the refusal cannot pass for the wrong reason. A limit the engine or a shared mechanic enforces (targeting,
  timing, equip, cycling) is already tested there.
- **Test a corner case only where it makes sense**: where the card's own code makes a decision an interaction could
  plausibly get wrong - it reads a value that can change before it is used ("that damage", "the number of lands"), it
  holds last-known information, it combines with another rule in a way its code handles itself. A card complex enough
  to need very specific testing gets it. Corner cases the engine handles for every card (an illegal target fizzling,
  a source gone before resolution, cleanup) are not retested card by card.
- **Verify the rules, not just that the script runs.** Every scenario ends with an `[Expect]` block stating the state
  the rules require - taken from the Oracle text, the rulings and the CR, **never copied from whatever the engine
  produced** - and at least one expectation in it fails if the card is wrong. An `[Expect]` that would also hold if
  the card were broken is not accepted. Put `?` checks where the position matters before the end.
- **Assert the outcome precisely**: the keys that prove the behaviour and no others, so a scenario fails for the
  right reason.

Name each scenario for the behaviour it proves (`wrath-of-god-destroys-a-creature-regenerated-in-response`) and start
it with a `#` comment: what it shows, why its actions are arranged that way, and the rule it proves.

If the harness cannot express something a scenario needs - a question, a check, a dump key - extend the harness
(`crates/cantrip-script`) and its README. Never weaken the scenario to fit the harness.

**A scenario that shows a bug is committed, not dropped.** If the engine or another card gets something wrong and you
cannot fix it in this change, commit the scenario asserting the rules-correct outcome with a `# Known bug: <what, and
the rule>` line at the top (`scenarios/README.md`, "Known bugs"). It is expected to fail; the fix removes the marker.
Give it an entry in `docs/known-issues.md` (severity, notes, the scenario), and a reviewer's suspected bug or a path no
scenario can reach one too; the fix deletes the entry.
Never delete, weaken or rewrite a scenario to pass because the engine is wrong, and never assert behaviour you know is
wrong. "0 fail" means no unexpected failures; the known-bug count is reported beside it.

## Review

Before a card lands, someone other than its author - a person, or a separate agent that had no part in writing it -
reads it with the Oracle text, the rulings and the CR open. **Its primary concerns are the code's correctness and
safety and the card's adherence to the rules**; the tests are checked for sufficiency, not exhaustiveness:

- every sentence of the Oracle text against the code, and every line of the code against the text: nothing missing,
  nothing extra, nothing approximate, nothing deviating from the rules;
- the "never" list above, line by line;
- the scenarios: every line covered (the coverage report), each `[Expect]` correct by the rules and able to fail.
  More tests are asked for only when the scenarios are obviously insufficient - code that no expectation would catch
  being wrong - or when the card is complex enough to need a specific test, or an interaction has a clear potential to
  go wrong. A list of corner cases is not a finding;
- any core or mechanics change against its rule, its second card (a non-keyword recipe) and the benchmark;
- overlap: code in the card that an existing shared function already does, or that does the same as code in another
  card, which should have been shared;
- the client: everything the card puts into the game is shown, every question it asks has a true caption, every
  action it offers is clickable with its cost, and a browser scenario proves
  each new user interaction the client had to learn - or the reviewer confirms the card needs nothing the client did
  not already do (then no browser scenario is owed).

The reviewer changes nothing silently: each finding is fixed by the author or recorded as the reason the card does not
land. Self-reported "exact" is not proof. A finding blocks the card when it is a rules deviation, a code defect, an
item on the "never" list, or a test that is obviously insufficient; the rest (wording, citations, optional sharing)
does not hold the card and is fixed in the same pass only if it is quick.

## Many cards: agents

The coordinator dispatches, lands and keeps the queue; it does not implement cards itself. Card agents run on Sonnet;
core-feature agents and review agents run on Opus.

0. **The worktree pool.** Work happens in a fixed pool of worktrees, `.claude/worktrees/pool-1` .. `pool-N`, each on
   its own branch with its own warm `target/`; N (about 4) is also the cap on build-heavy agents at once, because each
   cargo build takes every core. A slot is made once (`git worktree add .claude/worktrees/pool-<n> -b pool-<n>
   origin/main`) and never removed. Agents are started without the Agent tool's own worktree isolation and work in
   their slot by absolute path: every shell command starts with `cd <slot>` (so cargo uses the slot's workspace and
   `target/`) and every file path is under the slot. (`EnterWorktree` refuses a subagent that was not launched inside
   a worktree, so it cannot be relied on.) Every build stays in the slot's own `target/`: never
   `CARGO_TARGET_DIR` or `--target-dir` pointed at the main checkout's or another slot's, and never a copied
   `target/`, because cargo's freshness records name absolute source paths, and a shared or copied `target/` then
   compiles another tree's files into a test run that claims to test this one.
1. **One card per agent, many agents per slot.** A card agent takes one card in its slot: it starts the slot afresh
   (`git fetch`, then `git checkout -B pool-<n> origin/main`, once the slot's last work has landed), claims the card
   (`card-issues.py claim --format <format>`, or the card named), implements it, commits, reports, and stays alive
   for its review and fixes. When the card has landed, the coordinator ends that agent and starts a fresh one in the
   same slot for the next card: the finished card's history is no help to the next one and only costs tokens. Cards
   go in list order (never pre-sorted by difficulty). The coordinator's job is the format and how many slots are
   busy, not the claims, and it releases nothing an agent has not claimed. A card blocked by the core is never
   released: its agent commits nothing the gap would change, keeps the claim, sends a gap report and ends; the slot
   takes the next card meanwhile, and when the gap lands a fresh agent finishes the blocked card from the gap report.
2. **A card agent's prompt** names its slot and the format, points it at this skill, and adds no requirement the
   skill does not make. What the skill asks of it: this skill's loop for its card, claiming before any work
   (`Closes #N` from the claim's output); the core is closed to it unless the prompt opens
   it, though a missing keyword module may be added in `cantrip-mechanics`; a card blocked by the core is never faked
   and never released: the agent keeps the claim, commits nothing for that card's behaviour that the gap would
   change, sends the coordinator a gap report - the card, its issue, the exact rule (CR), what `cantrip-api` lacks
   and what it learned of the card - and ends (1); the client work a card needs (step 8) is part of the card, not a
   follow-up; one commit for the card, and a short report to the coordinator when it is committed (its commits,
   coverage, what it needed from the client, anything odd in the core); the targeted checks of step 9, rebased on
   `origin/main`; no push. Card agents run no browser tests
   unless a card added a new user interaction; then one agent at a time runs only that interaction's browser
   scenarios.
3. **Gaps.** When a card agent reports a gap, the coordinator starts a core-feature agent (Opus) for it in a free
   slot: the core open to it, the feature built to its rule for every card the rule describes, the benchmark run, its
   own commits (`Engine: ...`), no card work. Gaps several waiting cards share go to one feature agent; feature agents
   work on separate rules areas. The waiting card's claim is not renewed. As soon as the feature passes its review,
   the waiting card is started - a fresh card agent, given the gap report, that cherry-picks the reviewed core
   commits into its slot rather than waiting for them to land; the duplicates drop out at landing. Other running
   agents are told what landed so they rebase. Only a gap the coordinator decides not to build is released, with its
   gap recorded on the issue.
4. **Review agents** (Opus) review inside the implementing agent's slot - by absolute path as in (0), on the
   commit as it stands, using its warm build - never in a worktree of their own, and change no files, check
   out nothing and clean nothing there. The implementing agent waits while it is reviewed. Each card is reviewed as
   soon as it is committed and lands as soon as it passes. Their prompt points
   them at correctness, safety and rules adherence, and does not hand them a list of corner cases to demand tests
   for; the coordinator sends back only the blocking findings.
5. **Land** each reviewed card or core change on `main` (cherry-pick its commits; a conflict stops for a look), then
   one test run for everything ready to land together: `tools\test.cmd /engine` (the Rust workspace - the corpus
   with 0 fail and the verify tests) when the batch touches only cards, content, mechanics and scenarios, since
   nothing on the .NET side can tell a card apart; the whole `tools\test.cmd` when it touches the core
   (`cantrip-state`, `cantrip-rules`, `cantrip-api`), the host, the protocol, the client or anything under `dotnet/`, and otherwise at a much lower cadence (the last
   landing of a session, or every ten or so landings); `git fetch` and `git rebase origin/main` right
   before every push (other sessions land on `main` too). The full run is needed once, on a rebased `main`; if more
   commits arrive while it runs, rebase again, build, and run only the scenarios of the cards and mechanics the
   landing touches (`corpus -- --card "Name"` for each) before pushing - the session that landed those commits ran
   the full suite on them. Push; then end the slot's agent, and the slot takes its next card (1). Slots are never
   removed.
6. **Report** what landed, what waited on which core gap and how it was built, and the corpus line.
7. **Improve the process, always.** The coordinator keeps looking for ways to make this workflow - its own steps
   and the card, core, review and harness agents' - faster, measured against three pillars in this order, never
   traded the other way: (1) the code is correct and safe, (2) the Magic rules are enforced exactly, (3) cards are
   implemented fast. Watch for steps whose cost buys little toward a validated, rules-exact card: work done twice
   (the same tests run by agent and coordinator), waits (an idle blocked agent, a review held for a whole quota,
   cold builds, a rebase loop against a moving `main`), demands beyond what the card owes (exhaustive tests, benchmarks
   that measure nothing the change touched), and prompts that add requirements the skill does not make. Raise each
   with the user as a concrete change - what it costs now, what it saves, and why pillars 1 and 2 are untouched - and
   once approved write it into this skill (and `import-cards`, CLAUDE.md or the tools where it belongs) in the same
   session, so the next run starts from the better process.
