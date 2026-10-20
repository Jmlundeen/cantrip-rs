# Many cards: the coordinator and its agents

The coordinator dispatches, lands and keeps the queue; it does not implement cards itself. Card agents run on Sonnet;
core-feature agents and review agents run on Opus. Agents read only what their work needs: a card agent `SKILL.md`
(and the `write-game-script` skill), a reviewer `review.md` and the parts of `SKILL.md` it names, a core-feature agent
`SKILL.md`'s "Where code goes" and the rule it builds.

0. **The worktree pool.** Work happens in a fixed pool of worktrees, `.claude/worktrees/pool-1` .. `pool-N`, each on
   its own branch with its own warm `target/`; N (about 4) is also the cap on build-heavy agents at once, because each
   cargo build takes every core. A slot is made once (`git worktree add .claude/worktrees/pool-<n> -b pool-<n>
   origin/main`) and never removed. Agents are started without the Agent tool's own worktree isolation and work in
   their slot by absolute path: every shell command starts with `cd <slot>` (so cargo uses the slot's workspace and
   `target/`) and every file path is under the slot. (`EnterWorktree` refuses a subagent that was not launched inside a
   worktree.) Every build stays in the slot's own `target/`: never `CARGO_TARGET_DIR` or `--target-dir` pointed at
   another tree's, and never a copied `target/` - cargo's freshness records name absolute source paths, so a shared or
   copied `target/` compiles another tree's files into a test run that claims to test this one.
1. **One card per agent, many agents per slot.** Before starting an agent, `card-issues.py next <format>`: an agent
   started on an empty format spends minutes finding that out. A card agent takes one card in its slot: it starts the
   slot afresh (`git fetch`, then `git checkout -B pool-<n> origin/main`, once the slot's last work has landed), claims
   the card (`claim --format <format>`, or the card named), implements it, commits, reports, and stays alive for its
   review and fixes. When the card has landed, the coordinator ends that agent and starts a fresh one in the same slot
   for the next card: the finished card's history is no help to the next one and only costs tokens. Cards go in list
   order, never pre-sorted by difficulty. The coordinator's job is the format and how many slots are busy, not the
   claims, and it releases nothing an agent has not claimed.
2. **A card agent's prompt** names its slot and the format, points it at `SKILL.md` (read with the Read tool - it is
   longer than a shell result shows) and the `write-game-script` skill, and adds no requirement the skill does not
   make. It says: claim before any work; `new-card.py` first; the core is closed to it (a missing keyword module may go
   in `cantrip-mechanics`); run only its card's scenarios and coverage and those of a card whose file or shared
   mechanic it changed; one commit per card, `Closes #N`, mechanics and client commits before it; no push; and a short
   report when committed - its commits, coverage, each test point with the line that catches it and what it
   broke, client needs, anything odd in the core, and what in `new-card.py`'s
   output it used, what was noise, what it searched for by hand and what cost it failed runs (the coordinator's
   material for step 7). A card blocked by the core is never faked and never released: the agent keeps the claim,
   commits nothing the gap would change, sends a gap report - the card, its issue, the exact rule (CR), what
   `cantrip-api` lacks and what it learned - and ends; the slot takes the next card meanwhile. The agent logs its
   points on the card's issue (`claims.md`, "The progress log": `committed`, `fixes-committed`, `blocked`); a fresh
   agent finishing a card that waited logs `unblocked`.
3. **Gaps.** For a reported gap the coordinator starts a core-feature agent (Opus) in a free slot: the core open to it,
   the feature built to its rule for every card the rule describes, the benchmark run, its own commits
   (`Engine: ...`), no card work. Gaps several waiting cards share go to one feature agent; feature agents work on
   separate rules areas. As soon as the feature passes its review, a fresh card agent, given the gap report, finishes
   the waiting card, cherry-picking the reviewed core commits into its slot rather than waiting for them to land. Other
   running agents are told what landed so they rebase. Only a gap the coordinator decides not to build is released,
   with the gap recorded on the issue.
4. **Review agents** (Opus) review inside the implementing agent's slot - by absolute path as in (0), on the commit as
   it stands, using its warm build - never in a worktree of their own, and change no files, check out nothing and clean
   nothing there; they log `review-started` and their verdict on the card's issue. Their prompt points them at
   `review.md`, the card's issue number, the commit, the card's Oracle text and the CR rules it
   touches, and at correctness, safety and rules adherence; it hands them no list of corner cases to demand tests for.
   The implementing agent waits while it is reviewed; each card is reviewed as soon as it is committed. The
   coordinator sends back the blocking findings, and a non-blocking note that is a real test gap (a limit not met
   exactly, one player's side untested) when the fix is a line of a scenario.
5. **Land** each reviewed card or core change on `main` (cherry-pick its commits; a conflict stops for a look), then
   one test run for everything ready to land together: `tools\test.cmd /engine` (the Rust workspace, about a minute and
   a half: the corpus with 0 fail - known bugs reported beside it - and the verify tests) when the batch touches only
   cards, content, mechanics and scenarios; the whole `tools\test.cmd` when it touches the core (`cantrip-state`,
   `cantrip-rules`, `cantrip-api`), the host, the protocol, the client or anything under `dotnet/`, and otherwise at a
   much lower cadence (the last landing of a session, or every ten or so). `git fetch` and `git rebase origin/main`
   right before every push. If more commits arrive while the run goes, rebase, build, and run only the scenarios of
   the cards and mechanics this landing touches before pushing. Push, then `card-issues.py log <N> landed --commit
   <sha>` for each card (and `review-passed` first where the coordinator verified review fixes itself); end the slot's
   agent; the slot takes its next card (1).
6. **Report** what landed, what waited on which core gap and how it was built, the corpus line, and
   `card-issues.py times` for the cards of the run.
7. **Improve the process, always.** After each card, read the agent's report and the review for friction - a search by
   hand the tools could have answered, a failed run the skills could have prevented, a finding the agent could have
   caught itself - and fix the tool or the skill before the next card. Weigh every change against three pillars in
   this order, never traded the other way: (1) the code is correct and safe, (2) the Magic rules are enforced exactly,
   (3) cards are implemented fast. Watch for steps whose cost buys little toward a validated, rules-exact card: work
   done twice, waits (an idle blocked agent, a review held for a batch, cold builds, a rebase loop against a moving
   `main`), demands beyond what a card owes, and prompts that add requirements the skill does not make. Raise a change
   with the user when it trades anything; write approved changes into these files (and `import-cards`, CLAUDE.md or
   the tools) in the same session, so the next run starts from the better process.
