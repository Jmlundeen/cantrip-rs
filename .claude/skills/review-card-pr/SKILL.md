---
name: review-card-pr
description: Review a pull request that adds or fixes a card (or the mechanics it shares) against the add-cards standard, then fix what the review finds and land it as if the card had been built here - the PR's commits in a fresh worktree-pool slot, straight to an Opus review agent, a fresh Sonnet agent for the fixes, then the coordinator's landing. Use whenever a new card implementation from someone else is reviewed - a GitHub PR, a contributor's branch, commits handed over.
---

# Reviewing a card pull request

A card that arrives as a pull request is handled exactly like a card a local card agent has just committed: the
`add-cards` skill is the standard, and its `coordinator.md` is the procedure from step 4 (review) onward. This skill
only says how the PR's commits get there. Nothing here relaxes the standard. You are the coordinator: you dispatch,
check and land; you do not review or fix the card yourself.

Take pull requests **one at a time**, oldest first unless the user names one; report each before starting the next.

## 1. The PR's commits in a fresh pool slot

Take a lease on the worktree pool (`coordinator.md` step 0): `tools\worktree.cmd acquire --for "PR #<N>"` gives a
clean slot on today's `main`, its path and the lease ID, which stay with you. No slot free: wait, or ask the user to
grow the pool. With no member in the slot yet, put the PR on it:

```
cd <the path acquire printed>
git fetch origin pull/<N>/head:pr-<N>
git cherry-pick origin/main..pr-<N>        # the PR's own commits, in order, authorship kept
```

A cherry-pick conflict stops here and goes to the user; never resolve the contributor's code by guesswork. Note the
card(s) the PR adds or changes, every other card or shared function it touches, and the card's issue (`Closes #N` in
the commits, or `gh issue list --search "<Card Name> in:title" --state all`; a card with no issue gets one with
`card-issues.py claim "Card Name"` only if the user agrees - the claim may be the contributor's).

## 2. Straight to the review: an Opus agent

Start a fresh agent (`model: "opus"`, no worktree isolation) exactly as `coordinator.md` step 4 says for a review
agent: let into the lease by a `--read-only` token (`worktree.cmd token <lease> --read-only`, the token in its prompt,
never the path), on the commits as they stand, pointed at `review.md`, the card's issue number,
the commits, the card's Oracle text and the CR rules it touches - and nothing else, no opinion of the card and no list
of corner cases. Add only what a PR makes different:

- the commits are someone else's; review **every** file the PR changes, including any other card a refactor moved onto
  a shared function, since that card's behaviour changes too, and run those cards' scenarios as well;
- the seams are checked line by line: the card file holds only that card's own conditions, values, branching and
  sequencing, and every rules concept it touches (a name and name identity, mana value, a zone destination, a choice,
  "each player", the past, a keyword, a token) goes through the verb, query or shared mechanic that already owns it -
  grep `crates/cantrip-api` and `crates/cantrip-mechanics` for the concept, not for the name the author would have
  given it; a new or widened shared function names a second printed card that uses it exactly as written, is the same
  for every caller, and does real rules or game-state work rather than glue over a call that is already shared.

Read its report before acting on it; a finding that is wrong is dropped, and the user is told which and why.

## 3. Fixes: a fresh Sonnet agent

With blocking findings (or a non-blocking note that `coordinator.md` step 4 says to send back), start a fresh agent
(`model: "sonnet"`, no worktree isolation) in the same lease, with a write token once the reviewer has left, as the
card agent that would have written the card: its
prompt is a card agent's (`coordinator.md` step 2 - `SKILL.md` read with the Read tool, the `write-game-script` skill,
the core closed to it, only the touched cards' scenarios and coverage, no push), plus the findings verbatim and that
it fixes them on top of the PR's commits, logs `fixes-committed` on the issue and reports how each was fixed. A fix
that needs the core is a gap report, handled as `coordinator.md` step 3 says.

Send the fix commits back to the review agent (SendMessage, its context kept, a fresh read-only token in the message
once the fix agent has left) to confirm each finding is resolved and
nothing new is broken; loop until it passes, or stop and tell the user when a finding needs a decision only they can
make. No blocking findings: nothing to fix.

## 4. Land, and close the PR

Land as `coordinator.md` step 5 says (the test run it names, `git fetch` and rebase before the push, push, `log <N>
landed`). Then on the PR: `gh pr comment <N>` saying it landed, with the commit hashes on `main` and a short list of
what the review found and what was changed, and `gh pr close <N>`. Delete `pr-<N>`, then `tools\worktree.cmd release
<lease>`; the next PR takes a new lease. A PR that does not land: `release --discard --reason "PR #<N> not landed:
..."`, since its commits stay on the PR.

## 5. Report

Per PR, to the user: the card(s), the review's blocking and non-blocking findings (and any dropped, with why), what
the fix agent changed, the coverage and corpus lines, and what landed with its hashes - or why it did not land.
