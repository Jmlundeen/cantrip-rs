# Pointing an agent at this lane

`SKILL.md` beside this file is the whole card-lane instruction set: the queue, the claim, the packet, `add-cards`, the
gates, the two shared branches and the review gate. A solo implementor needs no coordinator; a pool implementor gets
its queue, worker id and token from a composer, and this file is the handoff. Any harness that can run a shell and
read a file can follow it.

## One implementor for a composer

Use this when a composer owns the queue and the pool, and each fresh agent takes one card in its own leased slot.

> Your first action, before anything else, is to join the slot; the token is single-use and dies in two minutes, so
> join now:
>
> `cd <CHECKOUT>`
> `python tools/maintenance/worktrees.py join <TOKEN> --agent-id <AGENT_ID>`
>
> (or `tools\worktree.cmd join ...`). `<AGENT_ID>` is the agent id your harness gave you at start (in Claude Code, the
> `SubagentStart` context); if your harness has no such id, use `<WORKER>`. `join` prints the slot path and your member
> id. `cd` into that slot and work only there. Never the main checkout, another slot, or a worktree of your own. If
> `join` refuses, stop and tell the composer; do not work in any worktree meanwhile.
>
> After you join, read `.claude/skills/card-lane/SKILL.md` in the Cantrip checkout at `<CHECKOUT>` (the absolute path
> if your harness starts elsewhere), and follow it together with the `add-cards` skill
> (`.claude/skills/add-cards/SKILL.md`) and the `write-game-script` skill
> (`.claude/skills/write-game-script/SKILL.md`). You are worker `<WORKER>` on queue `<QUEUE>`.
>
> The card-lane script is `<LANE>`: normally `tools/maintenance/card-lane.py` in the slot; while the lane is being
> tested before it lands on main, use the absolute path `<CHECKOUT>/tools/maintenance/card-lane.py` from the lane
> checkout. The queue worktree itself stays at `origin/main`; `next` refuses one carrying the lane's own files.
>
> The composer built the queue; do not run `queue`. Take the next card:
>
> `python <LANE> next --id <WORKER> --queue <QUEUE>`
>
> Do not name or choose a card; `next` chooses. It prints the brief and writes the packet in your slot. If it says the
> queue has no pending card, report that and leave. Claim the card's GitHub issue
> (`python tools/maintenance/card-issues.py claim "<Name>"`) and run the same `next` with `--issue <N>`. The core
> (`cantrip-state`, `cantrip-rules`, `cantrip-api`) is always in scope: a card whose text needs a rule the engine does
> not have changes the core rather than waiting on it. `--engine` is accepted and ignored.
>
> Do the card as `add-cards` says. A missing rule is a gap to fill in the core, proper and for every card, never a
> reason to block and never a workaround; `commit` writes it as the `Engine:` commit before the `Mechanics:` and card
> commits, so `share` puts the core first. Fill the packet's `report.json`, run
> `python <LANE> check --id <WORKER> --queue <QUEUE> --fast` while writing, and
> `python <LANE> commit --id <WORKER> --queue <QUEUE>` when it passes. `commit` runs every
> gate, writes the `Engine:`/`Mechanics:`/`Client:` commits the change needs and one `Card: <Name>` commit, then sends
> the card to the review gate. A card that passes is **not** done and does not land yet: `review --pass`, by someone
> who did not write it, is what clears it. Do not use `done` for a card you wrote.
>
> **The `Engine:` and `Mechanics:` commit messages are yours to write.** They are shared code, not the card's, so the
> subject says what changed in the rules' words - never a function's name - and the body says why; the repository's
> `git log --format=%B --grep="^Engine:"` is the model. Your shell has no terminal, so `commit` refuses the generated
> template rather than prompting. Pass the whole message with it:
>
> `python <LANE> commit --id <WORKER> --queue <QUEUE> --message mechanics="Mechanics: <what the recipe is>" --message mechanics2=@<a file with the body>`
>
> (`--message engine=...` for an `Engine:` commit: `--message engine="Engine: <the subject>"` and, when the body is
> more than one line, `--message engine2=@<file>`.) A `Client:` message is left as written.
>
> If the review sent the card back to you (`review --block` on the issue, `review --list` shows it), fix the
> findings on its own branch: `commit` again - with a written `Engine:`/`Mechanics:` message if the finding changes
> shared code - `share`, then tell the composer; the lane brings it back with `review --fix`. While 20 cards wait at
> the review gate, `next` claims no new card and says so: that is the gate being cleared, not an error.
>
> If the text needs something even a core change cannot express, run
> `python <LANE> block --id <WORKER> --queue <QUEUE> --rule <CR> --note "..."`, report the gap,
> and leave; never fake the core. If you must stop, run
> `python <LANE> release --id <WORKER> --queue <QUEUE> --reason "..."` so the card goes back.
> Work one card per lease: after `commit`, `block` or `release`, leave the slot with
> `python tools/maintenance/worktrees.py leave <MEMBER_ID>` (the member id `join` printed), then report to the
> composer with the card, its commits, the corpus line, coverage, test points, client needs and any core gap. Do not
> push, do not release the lease, and do not take a second card in this lease.
>
> The slot owns its build output: keep `target/` there, never `CARGO_TARGET_DIR` or another tree's target. Run every
> command from the slot. Use the editor's file tools for source, not shell rewriting. Long commands go in the
> foreground with a long timeout; if the harness moves one to the background, wait for it in short foreground checks
> and never end the turn while it runs. Scratch files go in a scratchpad folder named for `<WORKER>`.

The composer:

1. Build the queue once, from the lane checkout while the lane is not on main (otherwise the main checkout or any
   worktree): `python <LANE> queue --queue <QUEUE> --backlog` (or `--format` / `--file`).
2. For each implementor: `tools\worktree.cmd acquire --for "card: next <QUEUE>"`, then
   `tools\worktree.cmd token <LEASE> --name "card: <WORKER>"`. Start the agent immediately; the token dies in two
   minutes.
3. Start one fresh agent with the prompt above; fill `<CHECKOUT>`, `<QUEUE>`, `<WORKER>`, `<TOKEN>`, `<LANE>`. The
   core is in scope for every card, so there is no scope flag to set. `<LANE>` is the card-lane.py path: the lane
   checkout's copy while the lane is not on
   main, otherwise `tools/maintenance/card-lane.py` in the slot. Leave `<AGENT_ID>` for the agent to fill from its
   own context (or `<WORKER>` when its harness has none). Give each agent a unique `<WORKER>`; never give it the
   slot path or lease ID, and never a card name.
4. Watch `python <LANE> status --queue <QUEUE>`; when the agent leaves, its card waits at the review gate. Land a
   card only after `review --pass` has cherry-picked it onto `card-lane/<QUEUE>`, then `tools\worktree.cmd release
   <LEASE>`. A new card gets a new lease and a fresh agent.

## A solo implementor

Use this in a queue worktree with no pool.

> Read `.claude/skills/card-lane/SKILL.md` in the Cantrip checkout (absolute path if your harness starts elsewhere),
> and the `add-cards` (`.claude/skills/add-cards/SKILL.md`) and `write-game-script`
> (`.claude/skills/write-game-script/SKILL.md`) skills. Your queue is `<QUEUE>`; your id is `<WORKER>`. Work in a
> queue worktree based on `origin/main`; the lane checkout itself carries the lane's own commits and `next` refuses
> it. While the lane is not on main, build the queue from the lane checkout with
> `python <LANE> queue --queue <QUEUE> --backlog` (with `<LANE>` the lane checkout's
> `tools/maintenance/card-lane.py`), then work cards in the queue worktree with
> `python <LANE> next --id <WORKER> --queue <QUEUE>`,
> `python <LANE> check --id <WORKER> --queue <QUEUE> --fast` and
> `python <LANE> commit --id <WORKER> --queue <QUEUE>` until the queue is empty, writing every `Engine:` and
> `Mechanics:` message yourself (`--message engine=...`/`mechanics=...`). Do the card as `add-cards` says; one `Card:`
> commit per card, no `Co-Authored-By`. Each card that passes waits at the review gate
> (`review --list`, then `review --pass`/`--block` once you have reviewed it as `review.md` says); the queue branch
> carries only reviewed cards, and it is what you push or land as `claims.md` says.

## Several implementors at once

The composer section above is the parallel form: one queue, one lease and token per implementor, a unique `<WORKER>` for
each, and no card named in advance. The queue is atomic, so two `next` calls never take the same card. Two branches
carry the work, in that order: `card-lane/<QUEUE>-review` holds every card in progress - so a shared `Engine:` or
`Mechanics:` commit reaches the next implementor's `next` - and `card-lane/<QUEUE>` holds only cards a reviewer passed.
Both are built by cherry-pick, one commit per card commit, so a worktree's own history never lands. 20 cards at the
gate stop new claims, so the composer keeps reviewers working: a gate nobody clears stops every implementor.

What the agent needs on the machine: `python` (3.10+), `git`, `cargo` with the MSVC toolchain on Windows, and the
Cantrip checkout. It needs no Claude-specific tool: the prompts use the shell and file reads.
