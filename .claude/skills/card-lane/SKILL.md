---
name: card-lane
description: Hand cards to several implementors from one shared queue in the worktree pool - each worker runs `next` with its id and gets a packet in its own slot, no two take the same card, and a shared branch reuses mechanics and engine gaps. Use when more than one implementor works cards at once, or when a solo implementor wants the card packet.
---

# The card lane

`tools/maintenance/card-lane.py` is dispatch. It does not write a card, review it or land it, and it has no agent
loop and no generated track. The card itself is still the `add-cards` skill; its scenarios are still
`write-game-script`. This file is the queue, the claim, the packet and the shared branch.

## The model

- **One queue** per lane: `.git/card-lane/<queue>/queue.json`, beside the pool's state, shared by every worktree and
  never committed. The first `queue` builds or extends it; it never removes a card that is already there.
- **An id owns a card.** `next --id w1` claims the next pending card under a lock. Running it again with `w1` prints
  the same card and packet again; `w2` gets the next card. The same worktree holds one open card, so one implementor
  works one pool slot.
- **A packet in the implementor's worktree:** `.card-lane/<queue>/<card>/` - `brief.md`, `reference.md` (everything
  `new-card.py` prints: Oracle text, rulings, CR rules, prior art, test points), `prior-art.md` (the matched cards'
  code and a scenario), `content.json` and `state.json`. Gitignored, regenerable, thrown away with the slot.
- **One shared branch**, `card-lane/<queue>`. `next` merges its tip into the slot before a card is opened; `share`,
  `done`, `block` and `release` update it with a compare-and-swap. A `Mechanics:` or `Engine:` commit made by one
  implementor is therefore in the next implementor's worktree: shared mechanics and implemented engine gaps are
  reused instead of written twice.

The lane commands are run from the checkout or worktree they act on. `--queue` defaults to `default`; everyone who
should share a queue passes the same name.

## Coordinator

1. **Pool first.** `tools\worktree.cmd acquire --for "card: <Name>"` per card, then a token per implementor
   (`coordinator.md` step 0). The lane reads the pool's own state: in a slot it refuses a claim with no lease, and
   one with no joined member.
2. **Build the queue once**, from the main checkout or any worktree:

   ```sh
   python tools/maintenance/card-lane.py queue --queue trc --format pioneer
   python tools/maintenance/card-lane.py queue --queue trc --file docs/cube-lists/MyCube.txt
   python tools/maintenance/card-lane.py queue --queue trc --card "Lightning Bolt" --card "Counterspell"
   ```

   It takes the format list in its own order, skips cards `content/` already has, and keeps cards added earlier
   (`pending`, `done`, `blocked` and all). Use `--queue trc` on every later command of that lane.
3. **Start one implementor per slot** with its worker id (`w1`, `w2`, ...), the queue name, and this file plus
   `add-cards` and `write-game-script`. Never name the card: `next` chooses it.
4. **Watch, do not hand out:** `python tools/maintenance/card-lane.py status --queue trc` lists who holds what and
   whether a claim's lease or member has gone. A dead claim is freed with `release --id <id> --reason "..."` from
   its worktree, or from any worktree when the slot is gone.
5. **Land from the shared branch.** Its commits are local until pushed: `git push origin card-lane/trc` (or land
   them) before `tools\worktree.cmd release <lease>` - the pool refuses a slot whose commits are on no origin branch.

## Implementor

Your first command is `join` (`POOL.md`, worktree-pool-setup), so the slot has its one member. Then:

1. **Take the card:**

   ```sh
   python tools/maintenance/card-lane.py next --id w1 --queue trc
   ```

   It prints the brief and writes the packet into your worktree. If you lose your place, run exactly the same line:
   the card is still yours and nothing new is taken. `--card "<Name>"` picks a named card instead of the next one;
   `--refresh` rebuilds the packet (after `queue` added cards, for example).
2. **Do the card as `add-cards` says** - `new-card.py`, the rules, prior art, code, scenarios, the break loop, citations,
   coverage, client, review, and one `Card:` commit ending `Closes #N`. Claim the card's GitHub issue first
   (`card-issues.py claim "<Name>"`), unless you already hold it. `reference.md` and `prior-art.md` are your starting
   point; read only the sections of the skills the card needs.
3. **Share a mechanic as soon as it is committed.** After a `Mechanics:` or `Engine:` commit, and before the card is
   done:

   ```sh
   python tools/maintenance/card-lane.py share --id w1 --queue trc
   ```

   That merges the queue's branch into your slot if someone else moved it, then moves the branch to your `HEAD`.
4. **Close the claim:**

   ```sh
   python tools/maintenance/card-lane.py done --id w1 --queue trc
   ```

   It requires a clean tree and a commit since the card was opened (or `--no-commit` for a card that needed none), and
   shares your commits. For a card that waits on the engine:

   ```sh
   python tools/maintenance/card-lane.py block --id w1 --queue trc --rule 601.2c --note "..."
   ```

   For work that stops: `release --id w1 --queue trc --reason "..."` puts the card back. `abandon` closes it for good
   with the reason. `block`, `abandon` and `release` save any uncommitted diff as the packet's `work.patch` first.
5. **Leave the slot** with the MEMBER id `join` printed: `tools\worktree.cmd leave <MEMBER>`. Commit before leaving;
   do not push, and do not release the lease.

## Solo, without the pool

The lane works in any checkout: in a worktree outside `.claude/worktrees/pool-<n>` a claim is marked `solo` and no
lease or member is needed. Do the card, commit, `done`, and push or land the branch as `CLAIMS.md` says. The pool is
preferred when more than one implementor works at once; its slot is also what keeps two builds off each other.

## What the lane will not do

- It does not choose a card for a worktree that already has one open, and it refuses a second card in the same
  worktree under another id.
- It does not merge a branch that conflicts: it aborts the merge and tells you to resolve it by hand. Nothing is left
  half-merged.
- It does not write outside the worktree: the queue and the branch are in the repository's git common directory, the
  packet is in the worktree, and nothing is committed.
- It is not a claim on GitHub. Two people in different clones have two queues; the issue claim is still the rule
  across contributors (`claims.md`).
