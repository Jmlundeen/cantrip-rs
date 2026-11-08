---
name: card-lane
description: Hand cards to several implementors from one shared queue in the worktree pool - each worker runs `next` with its id and gets a packet in its own slot, no two take the same card, and a shared branch reuses mechanics and engine gaps. Use when more than one implementor works cards at once, or when a solo implementor wants the card packet.
---

# The card lane

`tools/maintenance/card-lane.py` is dispatch and the card's mechanical gates. It does not write a card, review it or
land it, and it has no agent loop and no generated track. The card itself is still the `add-cards` skill; its
scenarios are still `write-game-script`; its review is still someone who did not write it. This file is the queue, the
claim, the packet, the shared branch, `check` and `commit`.

## The model

- **One queue** per lane: `.git/card-lane/<queue>/queue.json`, beside the pool's state, shared by every worktree and
  never committed. The first `queue` builds or extends it; it never removes a card that is already there.
- **An id owns a card.** `next --id w1` claims the next pending card under a lock. Running it again with `w1` prints
  the same card and packet again; `w2` gets the next card. The same worktree holds one open card, so one implementor
  works one pool slot.
- **A packet in the implementor's worktree:** `.card-lane/<queue>/<card>/` - `brief.md`, `reference.md` (everything
  `new-card.py` prints: Oracle text, rulings, CR rules, prior art, test points), `prior-art.md` (the matched cards'
  code and a scenario), `content.json`, `report.json` (the line-by-line evidence `check` reads) and `state.json`.
  Gitignored, regenerable, thrown away with the slot.
- **One shared branch**, `card-lane/<queue>`. `next` merges its tip into the slot before a card is opened; `share`,
  `done`, `block` and `release` update it with a compare-and-swap. It is cut from remote main (`origin/main`), never
  from the checkout that ran `queue`; a worktree carrying the lane's own files is refused, and an existing queue
  branch cut from the lane's work is refused. A `Mechanics:` or `Engine:` commit made by one implementor is therefore
  in the next implementor's worktree: shared mechanics and implemented engine gaps are reused instead of written
  twice.

The lane commands are run from the checkout or worktree they act on. `--queue` defaults to `default`; everyone who
should share a queue passes the same name.

## Pointing an agent at this lane

`PROMPT.md` beside this file is the handoff a composer gives a fresh implementor: the queue, worker id and pool token,
never the slot path or lease, and it points back here for the card and gate rules. It also has a solo form for an agent
running in a checkout without the pool. Use the prompt rather than copying this file into a message; this file stays
the one instruction set.

## Coordinator

1. **Pool first.** `tools\worktree.cmd acquire --for "card: next <queue>"` per implementor, then a token per
   implementor (`coordinator.md` step 0). The lane reads the pool's own state: in a slot it refuses a claim with no
   lease, and one with no joined member.
2. **Build the queue once**, from the main checkout or any worktree:

   ```sh
   python tools/maintenance/card-lane.py queue --queue trc --format pioneer
   python tools/maintenance/card-lane.py queue --queue trc --file docs/cube-lists/MyCube.txt
   python tools/maintenance/card-lane.py queue --queue trc --backlog
   python tools/maintenance/card-lane.py queue --queue trc --backlog --list LSVCube
   python tools/maintenance/card-lane.py queue --queue trc --card "Lightning Bolt" --card "Counterspell"
   ```

   A source's names are kept in their order (the format's, the list's, `missing-cards.py`'s), cards `content/` already
   has are skipped, and cards added earlier (`pending`, `done`, `blocked` and all) are kept. `--backlog` is the real
   cube backlog: it runs `missing-cards.py` over every cube list (or `--list <Cube>`, or `--file <list>`) and queues
   what no card file defines, in the order `missing-cards.py` prints it. It is a source of its own, so narrow it with
   `--list`/`--file`, not with `--format`/`--card`; an empty backlog queues nothing and says so. Use `--queue trc` on
   every later command of that lane.
3. **Start one implementor per slot** with its worker id (`w1`, `w2`, ...), the queue name, and the prompt in
   `PROMPT.md` (it points at this file plus `add-cards` and `write-game-script`). Never name the card: `next` chooses
   it, and never give the agent the slot path or the lease; the token is its only way in.
4. **Watch, do not hand out:** `python tools/maintenance/card-lane.py status --queue trc` lists who holds what and
   whether a claim's lease or member has gone. A dead claim is freed with `release --id <id> --reason "..."` from
   its worktree, or from any worktree when the slot is gone.
5. **Land from the shared branch.** Its commits are local until pushed: `git push origin card-lane/trc` (or land
   them) before `tools\worktree.cmd release <lease>` - the pool refuses a slot whose commits are on no origin branch.

## Implementor

Your first command is `join` (`POOL.md`, worktree-pool-setup), so the slot has its one member. If a composer started
you from `PROMPT.md`, it already gives your queue, id, token and core scope; this section is the long form. Then:

1. **Take the card:**

   ```sh
   python tools/maintenance/card-lane.py next --id w1 --queue trc
   ```

   It prints the brief and writes the packet into your worktree. If you lose your place, run exactly the same line:
   the card is still yours and nothing new is taken. `--card "<Name>"` picks a named card instead of the next one;
   `--refresh` rebuilds the packet (after `queue` added cards, for example). Claim the card's GitHub issue
   (`card-issues.py claim "<Name>"`, unless you already hold it), then run the same line with `--issue <N>` so the
   claim and the `Card:` commit carry it. `--engine` is only for a card the coordinator has put the core in scope
   for; a card opened without it cannot change `cantrip-state`, `cantrip-rules` or `cantrip-api`.
2. **Do the card as `add-cards` says** - the rules, prior art, code, scenarios, the break loop, citations, coverage and
   client. `reference.md` and `prior-art.md` are your starting point; read only the sections of the skills the card
   needs. Fill the packet's `report.json` as you go: every printed Oracle line and the code that implements it, each
   test point with the scenario line that catches it, each ruling (a scenario line, or why the card does not decide
   it), a line answering every lint LOOK, and `equivalent_mutants` for a mutation survivor that means the same thing.
3. **Check while you write:**

   ```sh
   python tools/maintenance/card-lane.py check --id w1 --queue trc --fast
   ```

   It runs the claim, scope, Oracle text, scenario, corpus, lint, citation and report gates. `--fast` skips only
   coverage and mutation; the full `check` runs both. Read the gate's detail, fix it, and run it again.
4. **Commit and close - the gated path:**

   ```sh
   python tools/maintenance/card-lane.py commit --id w1 --queue trc
   ```

   It runs every gate, writes the `Engine:`, `Mechanics:` and `Client:` commits the change needs (each with only its
   own paths), then exactly one `Card: <Name>` commit ending `Closes #N`, and closes the claim. No commit carries
   `Co-Authored-By`. If it refuses, nothing is committed (or only the commits before the failing one): fix the gate
   and run it again. `done` also closes a claim, but it is the manual close for work committed by hand and does not
   run the gates; a card you wrote uses `commit`.
5. **Share a mechanic as soon as it is committed.** `commit` shares when it closes. If you commit an `Engine:` or
   `Mechanics:` change by hand before the card is ready, share it so the next worker starts with it:

   ```sh
   python tools/maintenance/card-lane.py share --id w1 --queue trc
   ```

   That merges the queue's branch into your slot if someone else moved it, then moves the branch to your `HEAD`.
6. **Block or release.** For a card that waits on the engine (never a workaround):

   ```sh
   python tools/maintenance/card-lane.py block --id w1 --queue trc --rule 601.2c --note "..."
   ```

   For work that stops: `release --id w1 --queue trc --reason "..."` puts the card back. `abandon` closes it for good
   with the reason. `block`, `abandon` and `release` save any uncommitted diff as the packet's `work.patch` first.
7. **Leave the slot** with the MEMBER id `join` printed: `tools\worktree.cmd leave <MEMBER>`. Commit before leaving;
   do not push, and do not release the lease.

## The gates

`check` (and `commit`) runs these in order. A full check adds the last two; `--fast` skips only those while the card
is being written.

| gate | what it refuses |
|---|---|
| `scope` | a path that is not the card's content, its Rust file, a scenario, a card note it deletes, a shared mechanic, the client, the harness, `docs/known-issues.md`, or the core without `--engine`; no card path or no changed scenario that holds the card |
| `oracle` | content whose Oracle text is not Scryfall's current text (an unreachable Scryfall is shown as WARN; check it by hand) |
| `scenarios` | a changed scenario with `# TODO`, `# Known bug`, no `[Expect]` or no assertion in it; no scenario holding the card; a card with a mana cost and no `? Bot1.Mana =` |
| `corpus` | any fail, known bug or need-cards count from `corpus --card`, a build warning from a changed file, or no passing scenario |
| `lint` | a `card-lint.py` NEVER finding; every LOOK and every `card-dupes.py` shared body must be answered in `report.json` |
| `citations` | a `CR` citation in the card file or a scenario that is not a rule of the current Comprehensive Rules |
| `report` | `decision` not `"done"`, no issue, a printed line with no code or marked disagreeing, a test point with no `?`/`[Expect]` line in a scenario holding the card, a ruling with no scenario or reason, an unanswered LOOK, or an unexcused survivor |
| `coverage` | any line of the card's file its scenarios never ran (`card-coverage.py`; full check only) |
| `mutation` | a `card-mutate.py` survivor that `report.json` does not declare equivalent (full check only) |

`report.json` starts in the packet from `new-card.py`'s printed text, test points and rulings, with the issue if one
was given. Its shape:

```json
{
  "issue": 123,
  "decision": "done",
  "reason": "",
  "validation": [{"line": "Flying", "code": "the generated Flying keyword", "agrees": true}],
  "test_points": [{"point": "...", "scenario": "scenarios/foo-shows-it.cantrip", "line": 31}],
  "rulings": [{"ruling": "...", "scenario": "scenarios/foo-shows-it.cantrip", "line": 31,
               "no_decision": ""}],
  "look_answers": ["crates/cantrip-cards/src/cards/foo.rs:12: the one object is the chosen target"],
  "equivalent_mutants": [{"mutant": "crates/cantrip-cards/src/cards/foo.rs:12: 4 to 5",
                          "why": "both faces print four"}],
  "notes": ""
}
```

A ruling the card does not decide needs no scenario line, only a `no_decision` of a sentence saying why. The file is
never committed; the `check`/`commit` output and `check.json` in the packet are the evidence.

## Testing the lane before it lands

A queue branch starts at `origin/main`, so the lane's own unmerged commits never enter a queue branch. Build the
queue from the lane checkout, but work cards in a queue worktree based on `origin/main` (a pool slot, or a worktree
made with `git worktree add -b queue-test ../queue-test origin/main`). Run the lane script from the lane checkout
against that worktree:

```sh
# in the lane checkout
python tools/maintenance/card-lane.py queue --queue test --backlog

# in the queue worktree
python <lane-checkout>/tools/maintenance/card-lane.py next --id w1 --queue test
python <lane-checkout>/tools/maintenance/card-lane.py check --id w1 --queue test --fast
python <lane-checkout>/tools/maintenance/card-lane.py commit --id w1 --queue test
```

The worktree's ordinary tools come from `origin/main`; the lane script adds the queue, gates and publication. Do
not take a card in the lane checkout: its lane commits are not on `origin/main`, and `next` refuses it. The queue
branch then contains only the card commits, which is what you can test before the lane itself is shared.

## Solo, without the pool

The lane works in any queue worktree: in a worktree outside `.claude/worktrees/pool-<n>` a claim is marked `solo`
and no lease or member is needed. A solo worktree starts at `origin/main` too; while the lane is not on main, create
one at `origin/main` and run the lane script from the lane checkout. Do the card, `commit`, and push or land the
branch as `claims.md` says. The pool is preferred when more than one implementor works at once; its slot is also
what keeps two builds off each other.

## What the lane will not do

- It does not choose a card for a worktree that already has one open, and it refuses a second card in the same
  worktree under another id.
- It does not merge a branch that conflicts: it aborts the merge and tells you to resolve it by hand. Nothing is left
  half-merged.
- It does not write outside the worktree: the queue and the branch are in the repository's git common directory, the
  packet is in the worktree, and nothing is committed.
- It is not a claim on GitHub. Two people in different clones have two queues; the issue claim is still the rule
  across contributors (`claims.md`).
- It is not a review. `check` finds the mechanical items; the card is still sent to someone who did not write it
  (`review.md`), and that review, not a green `check`, is the verdict.
